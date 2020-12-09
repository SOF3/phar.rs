use std::convert::TryInto;
use std::io::{self, Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::{LittleEndian, WriteBytesExt};

use super::util::{write_bstr, Crc32Writer, MultiWrite};
use crate::signature::Signature;
use crate::util::{tell, PHAR_TERMINATOR, STUB_TERMINATOR};
use crate::Compression;

/// Writes a phar file
///
/// For performance reasons, the name and metadata of _all_ entries
/// must be known at the beginning before writing any file content.
/// Editing previous writes is _not_ supported.
pub fn write<W: Read + Write + Seek>(stream: W, signature: Signature) -> WriterNeedStub<W> {
    WriterNeedStub { stream, signature }
}

/// Intermediate builder type for `Writer`.
///
/// Call `stub` to progress to the next builder step.
pub struct WriterNeedStub<W: Read + Write + Seek> {
    stream: W,
    signature: Signature,
}

impl<W: Read + Write + Seek> WriterNeedStub<W> {
    /// Sets the stub for the phar archive.
    ///
    /// It is not necessary to append the `__HALT_COMPILER();`,
    /// as the `phar` library will insert it automatically.
    ///
    /// It is not strictly necessary for the stub to start with `<?php`,
    /// but lack of `<?php` prevents the `php` command from running the file directly.
    ///
    /// Consider adding a line `#!/usr/bin/env php\n` before the `<?php` tag
    /// to allow direct shebang execution of the output file.
    pub fn stub(mut self, mut stub: impl Read) -> Result<WriterNeedAlias<W>> {
        let _ = io::copy(&mut stub, &mut self.stream)?;
        self.stream.write_all(STUB_TERMINATOR)?;
        let manifest_size_offset = tell(&mut self.stream)?;

        let _ = self.stream.seek(SeekFrom::Current(8))?; // manifest size, num_files
        self.stream.write_all(&[0x11, 0])?; // api
        self.stream.write_u32::<LittleEndian>(0x00010000)?; // flag

        Ok(WriterNeedAlias {
            manifest_size_offset,
            stream: self.stream,
            signature: self.signature,
        })
    }
}

/// Intermediate builder type for `Writer`.
///
/// Call `stub` to progress to the next builder step.
pub struct WriterNeedAlias<W: Read + Write + Seek> {
    manifest_size_offset: u64,
    stream: W,
    signature: Signature,
}

impl<W: Read + Write + Seek> WriterNeedAlias<W> {
    pub fn alias(mut self, alias: impl Read) -> Result<WriterNeedGlobMeta<W>> {
        write_bstr(&mut self.stream, alias, "alias is too long")?;
        Ok(WriterNeedGlobMeta {
            manifest_size_offset: self.manifest_size_offset,
            stream: self.stream,
            signature: self.signature,
        })
    }

    pub fn metadata(self, metadata: impl Read) -> Result<WriterNeedEntries<W>> {
        self.alias(io::empty())?.metadata(metadata)
    }
}

pub struct WriterNeedGlobMeta<W: Read + Write + Seek> {
    manifest_size_offset: u64,
    stream: W,
    signature: Signature,
}

impl<W: Read + Write + Seek> WriterNeedGlobMeta<W> {
    pub fn metadata(mut self, metadata: impl Read) -> Result<WriterNeedEntries<W>> {
        write_bstr(&mut self.stream, metadata, "metadata is too long")?;
        Ok(WriterNeedEntries {
            manifest_size_offset: self.manifest_size_offset,
            stream: self.stream,
            signature: self.signature,
            entries: Vec::new(),
        })
    }
}

pub struct WriterNeedEntries<W: Read + Write + Seek> {
    manifest_size_offset: u64,
    stream: W,
    signature: Signature,
    entries: Vec<WriteEntry>,
}

struct WriteEntry {
    uncompressed_offset: u64,
    compression: Compression,
}

impl<W: Read + Write + Seek> WriterNeedEntries<W> {
    /// Adds an entry to the phar.
    pub fn entry(
        &mut self,
        name: impl Read,
        metadata: impl Read,
        timestamp: SystemTime,
        mode: u32,
        compression: Compression,
    ) -> Result<()> {
        write_bstr(&mut self.stream, name, "file name is too long")?;
        let uncompressed_offset = tell(&mut self.stream)?;

        let _ = self.stream.seek(SeekFrom::Current(4))?; // uncompressed filesize

        self.stream.write_u32::<LittleEndian>(
            #[allow(clippy::as_conversions)]
            // explicit truncation to u32, since we have no better solution
            match timestamp.duration_since(UNIX_EPOCH) {
                Ok(duration) => duration.as_secs() as u32,
                Err(err) => {
                    let secs = err.duration().as_secs() as u32;
                    secs.wrapping_neg()
                }
            },
        )?;

        let _ = self.stream.seek(SeekFrom::Current(8))?; // compressed filesize, crc32
        self.stream.write_u32::<LittleEndian>({
            let mut out = mode & 0x1FF; // should we panic if mode >= 0x200?
            out |= compression.bit();
            out
        })?;

        write_bstr(&mut self.stream, metadata, "file metadata is too large")?;

        self.entries.push(WriteEntry {
            uncompressed_offset,
            compression,
        });

        Ok(())
    }

    pub fn contents(mut self) -> Result<Writer<W>> {
        let content_offset = tell(&mut self.stream)?;
        let manifest_size = content_offset - (self.manifest_size_offset + 4);

        let _ = self
            .stream
            .seek(SeekFrom::Start(self.manifest_size_offset))?;
        self.stream.write_u32::<LittleEndian>(
            manifest_size
                .try_into()
                .map_err(|_| Error::new(ErrorKind::Other, "manifest too large"))?,
        )?;

        Ok(Writer {
            stream: self.stream,
            entries: self.entries,
            ptr: Some(0),
            signature: Some(self.signature),
            end_offset: content_offset,
        })
    }
}

pub struct Writer<W: Read + Write + Seek> {
    stream: W,
    entries: Vec<WriteEntry>,
    ptr: Option<usize>,
    signature: Option<Signature>,
    end_offset: u64,
}

impl<W: Read + Write + Seek> Writer<W> {
    pub fn feed(&mut self, read: impl Read) -> Result<()> {
        fn try_feed(
            entry: &WriteEntry,
            mut read: impl Read,
            mut write: impl Write + Seek,
            start_offset: u64,
        ) -> Result<u64> {
            let start = write.seek(SeekFrom::Start(start_offset))?;

            let mut comp_write = entry.compression.make_write(&mut write)?;

            let mut cksum = Crc32Writer::default();

            #[allow(clippy::as_conversions)]
            let uncompressed_size = io::copy(
                &mut read,
                &mut MultiWrite([
                    &mut comp_write as &mut dyn Write,
                    &mut cksum as &mut dyn Write,
                ]),
            )?;
            drop(comp_write);

            let end = tell(&mut write)?;
            let compressed_size = end - start;

            let _ = write.seek(SeekFrom::Start(entry.uncompressed_offset))?;
            write.write_u32::<LittleEndian>(
                uncompressed_size
                    .try_into()
                    .map_err(|_| Error::new(ErrorKind::Other, "content is too large"))?,
            )?;
            let _ = write.seek(SeekFrom::Current(4))?; // unix timestamp already written
            write.write_u32::<LittleEndian>(
                compressed_size
                    .try_into()
                    .map_err(|_| Error::new(ErrorKind::Other, "content is too large"))?,
            )?;
            write.write_u32::<LittleEndian>(cksum.finish())?;

            Ok(end)
        }

        fn write_signature(
            mut stream: impl Read + Write + Seek,
            end_offset: u64,
            mut signature: Signature,
        ) -> Result<()> {
            let _ = stream.seek(SeekFrom::Start(0))?;
            let _ = io::copy(&mut (&mut stream).take(end_offset), &mut signature.write())?;
            let sig_id = signature.to_u32();
            let bytes = signature.finalize();
            let _ = stream.seek(SeekFrom::Start(end_offset))?;
            stream.write_all(&bytes[..])?;
            stream.write_u32::<LittleEndian>(sig_id)?;
            stream.write_all(PHAR_TERMINATOR)?;
            Ok(())
        }

        let ptr = match self.ptr {
            Some(ptr) => ptr,
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "feed() called again after returning Err",
                ))
            }
        };
        let Writer {
            stream: write,
            entries,
            end_offset,
            signature,
            ..
        } = self;
        let entry = match entries.get(ptr) {
            Some(entry) => entry,
            None => return Err(Error::new(ErrorKind::Other, "feed() called too many times")),
        };
        let ret = try_feed(entry, read, &mut *write, *end_offset);
        match &ret {
            Ok(new_end_offset) => {
                self.ptr = Some(ptr + 1);
                if entries.get(ptr + 1).is_none() {
                    write_signature(write, *new_end_offset, signature.take().expect("last call"))?;
                }
                self.end_offset = *new_end_offset;
            }
            Err(err) => self.ptr = None,
        }
        ret.map(|_| ())
    }
}
