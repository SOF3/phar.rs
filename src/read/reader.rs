use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};
use shallow_tees::ShallowTees;
use takes::Ext;
use typed_builder::TypedBuilder;

use super::util::read_find_bstr;
use super::{Entry, FileIndex, Section};
use crate::signature::{self, Signature};
use crate::util::{PHAR_TERMINATOR, STUB_TERMINATOR};

/// The metadata of a phar file.
#[derive(Debug)]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "reader")))]
pub struct Reader<R: Read + Seek, FileIndexT: FileIndex> {
    stream: R,
    stub: Section,
    num_files: u32,
    api: u16,
    flags: u32,
    alias: Section,
    metadata: Section,
    file_index: FileIndexT,
}

impl<R: Read + Seek, FileIndexT: FileIndex> Reader<R, FileIndexT> {
    /// Parses the phar file.
    ///
    /// This optionally validates the signature.
    /// Stub, metadata and file metadata are not fully validated,
    /// and may not be saved in memory depending on `options`.
    pub fn read(mut read: R, options: Options) -> Result<Self> {
        let mut expected_sig = None;
        let mut sig_offset = None;

        let mut sig = if options.verify_signature {
            let _ = read.seek(SeekFrom::End(-4))?;
            let mut gbmb = [0u8; 4];
            read.read_exact(&mut gbmb[..])?;
            if gbmb != PHAR_TERMINATOR {
                return Err(Error::new(ErrorKind::Other, "corrupted file"));
            }

            let _ = read.seek(SeekFrom::End(-8))?;
            let discrim = read.read_u32::<LittleEndian>()?;
            let sig = Signature::from_u32(discrim).ok_or_else(|| {
                Error::new(
                    ErrorKind::Other,
                    format!("unsupported signature type {:x}", discrim),
                )
            })?;

            let mut expect = vec![0u8; sig.size().into()];
            let offset = read.seek(SeekFrom::End(-8i64 - i64::from(sig.size())))?;
            sig_offset = Some(offset);
            read.read_exact(&mut expect[..])?;
            expected_sig = Some(expect);

            signature::MaybeDummy::Real(sig)
        } else {
            signature::MaybeDummy::Dummy(signature::NullDevice)
        };

        let _ = read.seek(SeekFrom::Start(0))?;

        let mut tee = ShallowTees::new(&mut read, sig.write());

        let mut stub = Section::create(options.cache_stub, 0);
        read_find_bstr(&mut tee, &mut stub, STUB_TERMINATOR)?;

        let manifest_size = tee.read_u32::<LittleEndian>()?;
        let mut manifest = (&mut tee).takes(manifest_size.into())?;

        let num_files = manifest.read_u32::<LittleEndian>()?;
        let api = manifest.read_u16::<LittleEndian>()?;
        let flags = manifest.read_u32::<LittleEndian>()?;

        let alias_len = manifest.read_u32::<LittleEndian>()?;
        let mut alias = Section::create(options.cache_alias, manifest.seek(SeekFrom::Current(0))?);
        alias.from_read(&mut manifest, alias_len)?;

        let metadata_len = manifest.read_u32::<LittleEndian>()?;
        let mut metadata =
            Section::create(options.cache_metadata, manifest.seek(SeekFrom::Current(0))?);
        metadata.from_read(&mut manifest, metadata_len)?;

        let mut file_index = FileIndexT::default();
        if FileIndexT::scan_files() {
            for _ in 0..num_files {
                let start = manifest.seek(SeekFrom::Current(0))?;
                let entry = Entry::parse(
                    &mut manifest,
                    FileIndexT::requires_name(),
                    FileIndexT::requires_metadata(),
                )?;
                file_index.feed_entry(start, entry)?;
            }
        }

        if let (Some(expected_sig), Some(sig_offset)) = (expected_sig, sig_offset) {
            let _ = tee.seek(SeekFrom::Start(sig_offset))?;
            drop(tee);
            let sig = match sig {
                signature::MaybeDummy::Real(sig) => sig,
                signature::MaybeDummy::Dummy(_) => {
                    unreachable!("expected_sig, sig_offset should be None")
                }
            };
            let ret = sig.finalize();
            if ret[..] != expected_sig {
                return Err(Error::new(ErrorKind::Other, "signature mismatch"));
            }
        }

        Ok(Reader {
            stream: read,
            stub,
            num_files,
            api,
            flags,
            alias,
            metadata,
            file_index,
        })
    }

    /// Returns the stub as a slice.
    ///
    /// If the stub was previously not stored in memory, it is stored in a new Vec.
    /// Consider using `stub_read()` instead if `cache_stub` is false
    /// and storing the stub in memory is not intended.
    pub fn stub_bytes(&mut self) -> Result<impl AsRef<[u8]> + '_> {
        self.stub.as_memory(&mut self.stream)
    }

    /// Returns the stub as an `io::Read`.
    pub fn stub_read(&mut self) -> Result<impl Read + '_> {
        self.stub.as_read(&mut self.stream)
    }

    /// Returns the metadata as a slice.
    ///
    /// If the metadata was previously not stored in memory, it is stored in a new Vec.
    /// Consider using `metadata_read()` instead if `cache_metadata` is false
    /// and storing the metadata in memory is not intended.
    pub fn metadata_bytes(&mut self) -> Result<impl AsRef<[u8]> + '_> {
        self.metadata.as_memory(&mut self.stream)
    }

    /// Returns the metadata as an `io::Read`.
    pub fn metadata_read(&mut self) -> Result<impl Read + '_> {
        self.metadata.as_read(&mut self.stream)
    }
}

#[derive(Default, TypedBuilder)]
pub struct Options {
    /// Whether to cache the phar stub in memory
    ///
    /// Default true.
    /// If set to false, stub is read from the input `R` again
    /// when it is queried by the user.
    /// False is only recommended when stub is not going to be used.
    #[builder(default = true)]
    cache_stub: bool,
    /// Whether to cache the phar alias in memory
    ///
    /// Default true.
    /// If set to false, alias is read from the input `R` again
    /// when it is queried by the user.
    /// False is only recommended when stub is not going to be used.
    #[builder(default = true)]
    cache_alias: bool,
    /// Whether to cache the phar metadata string in memory
    ///
    /// Default true.
    /// If set to false, metadata is read from the input `R` again
    /// when it is queried by the user.
    /// False is only recommended when stub is not going to be used.
    #[builder(default = true)]
    cache_metadata: bool,

    /// Whether to verify the phar signature.
    ///
    /// Default true.
    /// When true, the whole file is scanned at least once
    /// when the file is first parsed.
    /// When false, unused bytes would be skipped (with `fseek(3)`)
    /// instead of being read into buffer.
    #[builder(default = true)]
    verify_signature: bool,
}
