use std::io::{Error, ErrorKind, Read, Result, Write};

/// A file compression method.
///
/// `Zlib` and `Bzip` are available even without their corresponding features,
/// because this is used in file encoding flags.
#[derive(Debug, Clone, Copy)]
pub enum Compression {
    /// No compression
    None,
    /// zlib (gzip) compression
    ///
    /// The inner `u32` is the deflate compression level.
    /// This value is *always* zero when passed from the library;
    /// it is only used in the `write` module.
    ///
    /// See [`flate2::Compression`](https://docs.rs/flate2/1/flate2/struct.Compression.html) for
    /// details.
    Zlib(u32),
    /// bzip compression
    ///
    /// The inner `u32` is the bzip2 compression level.
    /// This value is *always* zero when passed from the library;
    /// it is only used in the `write` module.
    ///
    /// See [`bzip2::Compression`](https://docs.rs/bzip2/0.4/bzip2/struct.Compression.html) for
    /// details.
    Bzip(u32),
}

impl Compression {
    pub(crate) fn bit(self) -> u32 {
        match self {
            Self::None => 0,
            Self::Zlib(_) => 0x00001000,
            Self::Bzip(_) => 0x00002000,
        }
    }

    pub(crate) fn from_bit(bit: u32) -> Self {
        if bit & 0x00001000 > 0 {
            return Self::Zlib(0);
        }
        if bit & 0x00002000 > 0 {
            return Self::Bzip(0);
        }
        Self::None
    }

    #[cfg(feature = "writer")]
    pub(crate) fn make_write<'t>(self, write: impl Write + 't) -> Result<Box<dyn Write + 't>> {
        match self {
            Self::None => Ok(Box::new(write)),
            #[cfg(feature = "comp-zlib")]
            Self::Zlib(level) => Ok(Box::new(flate2::write::ZlibEncoder::new(
                write,
                flate2::Compression::new(level),
            ))),
            #[cfg(feature = "comp-bzip")]
            Self::Bzip(level) => Ok(Box::new(bzip2::write::BzEncoder::new(
                write,
                bzip2::Compression::new(level),
            ))),
            #[allow(unreachable_patterns)] // unreachable when all features enabled
            _ => Err(Error::new(
                ErrorKind::Other,
                "unsupported compression algorithm (not compiled with comp-zlib/comp-bzip feature)",
            )),
        }
    }

    #[cfg(feature = "reader")]
    pub(crate) fn make_read<'t>(self, read: impl Read + 't) -> Result<Box<dyn Read + 't>> {
        match self {
            Self::None => Ok(Box::new(read)),
            #[cfg(feature = "comp-zlib")]
            Self::Zlib(_) => Ok(Box::new(flate2::read::ZlibDecoder::new(read))),
            #[cfg(feature = "comp-zlib")]
            Self::Bzip(_) => Ok(Box::new(bzip2::read::BzDecoder::new(read))),
            #[allow(unreachable_patterns)] // unreachable when all features enabled
            _ => Err(Error::new(
                ErrorKind::Other,
                "unsupported compression algorithm (not compiled with comp-zlib/comp-bzip feature)",
            )),
        }
    }
}
