use std::io::{Result, Write};

use auto_enums::auto_enum;
use digest::Digest;
use smallvec::SmallVec;

/// A possible phar signature
#[derive(Debug)]
pub enum Signature {
    /// Signature corresponding to `Phar::MD5`
    #[cfg(feature = "sig-md5")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "sig-md5")))]
    Md5(md5::Md5),
    /// Signature corresponding to `Phar::SHA1`
    #[cfg(feature = "sig-sha1")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "sig-sha1")))]
    Sha1(sha1::Sha1),
    /// Signature corresponding to `Phar::SHA256`
    #[cfg(feature = "sig-sha2")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "sig-sha2")))]
    Sha256(sha2::Sha256),
    /// Signature corresponding to `Phar::SHA512`
    #[cfg(feature = "sig-sha2")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "sig-sha2")))]
    Sha512(sha2::Sha512),
}

impl Signature {
    /// Creates an md5 signature
    #[cfg(feature = "sig-md5")]
    pub fn md5() -> Self {
        Self::Md5(Digest::new())
    }

    /// Creates a sha1 signature
    #[cfg(feature = "sig-sha1")]
    pub fn sha1() -> Self {
        Self::Sha1(Digest::new())
    }

    /// Creates a sha256 signature
    #[cfg(feature = "sig-sha2")]
    pub fn sha256() -> Self {
        Self::Sha256(Digest::new())
    }

    /// Creates a sha512 signature
    #[cfg(feature = "sig-sha2")]
    pub fn sha512() -> Self {
        Self::Sha512(Digest::new())
    }

    /// Creates a signature from the phar format flag
    pub fn from_u32(discrim: u32) -> Option<Self> {
        Some(match discrim {
            #[cfg(feature = "sig-md5")]
            1 => Self::Md5(Digest::new()),
            #[cfg(feature = "sig-sha1")]
            2 => Self::Sha1(Digest::new()),
            #[cfg(feature = "sig-sha2")]
            3 => Self::Sha256(Digest::new()),
            #[cfg(feature = "sig-sha2")]
            4 => Self::Sha512(Digest::new()),
            _ => return None,
        })
    }

    /// Returns the phar format flag of the signature type
    pub fn to_u32(&self) -> u32 {
        match self {
            #[cfg(feature = "sig-md5")]
            Self::Md5(_) => 1,
            #[cfg(feature = "sig-sha1")]
            Self::Sha1(_) => 2,
            #[cfg(feature = "sig-sha2")]
            Self::Sha256(_) => 3,
            #[cfg(feature = "sig-sha2")]
            Self::Sha512(_) => 4,
        }
    }

    /// The number of bytes used for this signature
    pub fn size(&self) -> u8 {
        match self {
            #[cfg(feature = "sig-md5")]
            Self::Md5(_) => 16,
            #[cfg(feature = "sig-sha1")]
            Self::Sha1(_) => 20,
            #[cfg(feature = "sig-sha2")]
            Self::Sha256(_) => 32,
            #[cfg(feature = "sig-sha2")]
            Self::Sha512(_) => 64,
        }
    }

    /// Returns a `Write` that writes to the underlying digest
    pub fn write(&mut self) -> impl Write + '_ {
        #[allow(clippy::as_conversions)]
        match self {
            #[cfg(feature = "sig-md5")]
            Self::Md5(digest) => digest as &mut dyn Write,
            #[cfg(feature = "sig-sha1")]
            Self::Sha1(digest) => digest as &mut dyn Write,
            #[cfg(feature = "sig-sha2")]
            Self::Sha256(digest) => digest as &mut dyn Write,
            #[cfg(feature = "sig-sha2")]
            Self::Sha512(digest) => digest as &mut dyn Write,
        }
    }

    /// Returns the memory allocated by the underlying signature implementation
    pub fn finalize(self) -> SmallVec<[u8; 64]> {
        let mut ret = SmallVec::new();
        match self {
            #[cfg(feature = "sig-md5")]
            Self::Md5(digest) => ret.extend(digest.finalize()[..].iter().copied()),
            #[cfg(feature = "sig-sha1")]
            Self::Sha1(digest) => ret.extend(digest.finalize()[..].iter().copied()),
            #[cfg(feature = "sig-sha2")]
            Self::Sha256(digest) => ret.extend(digest.finalize()[..].iter().copied()),
            #[cfg(feature = "sig-sha2")]
            Self::Sha512(digest) => ret.extend(digest.finalize()[..].iter().copied()),
        };
        ret
    }
}

pub enum MaybeDummy {
    Real(Signature),
    Dummy(NullDevice),
}

impl MaybeDummy {
    #[auto_enum(io::Write)]
    pub fn write(&mut self) -> impl Write + '_ {
        match self {
            Self::Real(sig) => sig.write(),
            Self::Dummy(dev) => dev,
        }
    }
}

pub struct NullDevice;

impl Write for NullDevice {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
