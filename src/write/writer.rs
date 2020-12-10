use std::borrow::Cow;
use std::convert::TryInto;
use std::ffi::OsStr;
use std::io::{self, Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::{LittleEndian, WriteBytesExt};
use walkdir::WalkDir;

use super::util::{write_bstr, Crc32Writer, MultiWrite};
use crate::signature::Signature;
use crate::util::{tell, PHAR_TERMINATOR, STUB_TERMINATOR};
use crate::Compression;

/// Creates a phar file.
///
/// The `stream` must support *random-access read AND write*.
/// When passing `fs::File`, it should be created with
/// `fs::OpenOptions::new().read(true).write(true).create(true)`,
/// optionally with `truncate(true)` as well.
///
/// For performance reasons, the name and metadata of _all_ entries
/// must be known at the beginning before writing any file content.
/// Editing previous writes is _not_ supported,
/// because that would require repacking all subsequent contents.
pub fn create<W: Read + Write + Seek>(stream: W, signature: Signature) -> NeedStub<W> {
    NeedStub { stream, signature }
}

/// Intermediate type for writing phar.
///
/// Call `stub` to progress to the next builder step.
pub struct NeedStub<W: Read + Write + Seek> {
    stream: W,
    signature: Signature,
}

impl<W: Read + Write + Seek> NeedStub<W> {
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
    pub fn stub(mut self, mut stub: impl Read) -> Result<NeedAlias<W>> {
        let _ = io::copy(&mut stub, &mut self.stream)?;
        self.stream.write_all(STUB_TERMINATOR)?;
        let manifest_size_offset = tell(&mut self.stream)?;

        let _ = self.stream.seek(SeekFrom::Current(8))?; // manifest size, num_files
        self.stream.write_all(&[0x11, 0])?; // api
        self.stream.write_u32::<LittleEndian>(0x00010000)?; // flag

        Ok(NeedAlias {
            manifest_size_offset,
            stream: self.stream,
            signature: self.signature,
        })
    }
}

/// Intermediate type for writing phar.
///
/// Call `alias` or `metadata` to progress to the next builder step.
pub struct NeedAlias<W: Read + Write + Seek> {
    manifest_size_offset: u64,
    stream: W,
    signature: Signature,
}

impl<W: Read + Write + Seek> NeedAlias<W> {
    /// Sets the alias for the phar archive.
    pub fn alias(mut self, alias: impl Read) -> Result<NeedGlobMeta<W>> {
        write_bstr(&mut self.stream, alias, "alias is too long")?;
        Ok(NeedGlobMeta {
            manifest_size_offset: self.manifest_size_offset,
            stream: self.stream,
            signature: self.signature,
        })
    }

    /// Sets the metadata for the phar archive.
    ///
    /// The `phar` crate does not validate the contents,
    /// but they should either be empty string or comply to PHP serialization format.
    pub fn metadata(self, metadata: impl Read) -> Result<NeedEntries<W>> {
        self.alias(io::empty())?.metadata(metadata)
    }
}

/// Intermediate type for writing phar.
pub struct NeedGlobMeta<W: Read + Write + Seek> {
    manifest_size_offset: u64,
    stream: W,
    signature: Signature,
}

impl<W: Read + Write + Seek> NeedGlobMeta<W> {
    /// Sets the metadata for the phar archive.
    ///
    /// The `phar` crate does not validate the contents,
    /// but they should either be empty string or comply to PHP serialization format.
    pub fn metadata(mut self, metadata: impl Read) -> Result<NeedEntries<W>> {
        write_bstr(&mut self.stream, metadata, "metadata is too long")?;
        Ok(NeedEntries {
            manifest_size_offset: self.manifest_size_offset,
            stream: self.stream,
            signature: self.signature,
            entries: Vec::new(),
            global_flags: 0x00010000,
        })
    }
}

/// Preparation step for writing phar entries.
///
/// For performance reasons, users need to first provide all file metadata
/// before providing all file contents.
/// Consider using `build_from_*`
/// if the data source is located on the filesystem.
pub struct NeedEntries<W: Read + Write + Seek> {
    manifest_size_offset: u64,
    stream: W,
    signature: Signature,
    entries: Vec<WriteEntry>,
    global_flags: u32,
}

impl<W: Read + Write + Seek> NeedEntries<W> {
    /// Adds an entry to the phar.
    ///
    /// The file contents shall be later passed with the `Contents::feed` method in the same order.
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

        self.global_flags |= compression.bit();

        write_bstr(&mut self.stream, metadata, "file metadata is too large")?;

        self.entries.push(WriteEntry {
            uncompressed_offset,
            compression,
        });

        Ok(())
    }

    /// Starts writing the contents section of the phar.
    ///
    /// Users should call `feed` on the returned `Contents` value with the file contents
    /// in the exact same order as entries declared with `entry`.
    pub fn contents(mut self) -> Result<Contents<W>> {
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
        self.stream.write_u32::<LittleEndian>(
            self.entries
                .len()
                .try_into()
                .map_err(|_| Error::new(ErrorKind::Other, "too many file entries"))?,
        )?;
        let _ = self.stream.seek(SeekFrom::Current(2))?; // phar api version
        self.stream.write_u32::<LittleEndian>(self.global_flags)?;

        Ok(Contents {
            stream: self.stream,
            entries: self.entries,
            ptr: Some(0),
            signature: Some(self.signature),
            end_offset: content_offset,
        })
    }

    /// Builds the phar from a directory on the filesystem.
    pub fn build_from_directory(self, path: &Path, compression: Compression) -> Result<()> {
        let vec: Result<Vec<(_, _)>> = WalkDir::new(path)
            .into_iter()
            .map(|entry| {
                let entry = entry?;
                Ok((
                    entry
                        .path()
                        .strip_prefix(path)
                        .map_err(|_| {
                            Error::new(ErrorKind::Other, "path is not a prefix of walked entry")
                        })?
                        .as_os_str()
                        .to_owned(),
                    entry.path().to_owned(),
                ))
            })
            .collect();
        let vec = vec?;
        self.build_from_path_iter(|| vec.iter().map(|(a, b)| Ok((a, b))), compression)
    }

    /// Builds the phar from an iterator of file paths.
    ///
    /// The iterator parameter yields `(S, P)` pairs,
    /// where each `S` is an `OsStr` representing the path inside the archive
    /// and each `P` is a `Path` that resolves to the actual file to include
    /// (at least relative to the current working directory).
    pub fn build_from_path_iter<S, P, I>(
        mut self,
        iter: impl Fn() -> I,
        compression: Compression,
    ) -> Result<()>
    where
        I: Iterator<Item = Result<(S, P)>>,
        S: AsRef<OsStr>,
        P: AsRef<Path>,
    {
        use std::fs;

        #[cfg(unix)]
        fn os_str_to_bytes(name: &OsStr) -> impl AsRef<[u8]> + '_ {
            use std::os::unix::ffi::OsStrExt;
            Cow::Borrowed(name.as_bytes())
        }

        #[cfg(not(unix))]
        fn os_str_to_bytes(name: &OsStr) -> impl AsRef<[u8]> {
            match name.to_string_lossy() {
                Cow::Borrowed(name) => Cow::Borrowed(name.as_bytes()),
                Cow::Owned(name) => Cow::Owned(name.into_bytes()),
            }
        }

        #[cfg(unix)]
        fn stat_to_mode(permissions: fs::Permissions) -> u32 {
            use std::os::unix::fs::PermissionsExt;
            permissions.mode()
        }
        #[cfg(not(unix))]
        fn stat_to_mode(permissions: fs::Permissions) -> u32 {
            if permissions.readonly() {
                0o444
            } else {
                0o664
            }
        }

        for pair in iter() {
            let (name, file) = pair?;
            let stat = file.as_ref().metadata()?;
            if stat.is_file() {
                self.entry(
                    os_str_to_bytes(name.as_ref()).as_ref(),
                    &b""[..],
                    stat.modified()?,
                    stat_to_mode(stat.permissions()),
                    compression,
                )?;
            }
        }
        let mut contents = self.contents()?;
        for pair in iter() {
            let (_, file) = pair?;
            contents.feed(fs::File::open(file)?)?;
        }
        Ok(())
    }
}

struct WriteEntry {
    uncompressed_offset: u64,
    compression: Compression,
}

/// Step for writing phar file contents.
///
/// See also the documentation of `NeedEntries`.
///
/// The file signature is automatically appended when all files have been written.
/// File state is _undefined_ before the last entry is written.
pub struct Contents<W: Read + Write + Seek> {
    stream: W,
    entries: Vec<WriteEntry>,
    ptr: Option<usize>,
    signature: Option<Signature>,
    end_offset: u64,
}

impl<W: Read + Write + Seek> Contents<W> {
    /// Passes the content source for the next file entry.
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
        let Contents {
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
            Err(_) => self.ptr = None,
        }
        ret.map(|_| ())
    }
}
