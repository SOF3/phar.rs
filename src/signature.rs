use std::io::{Result, Write};

use digest::Digest;
use smallvec::SmallVec;

/// A possible phar signature
pub enum Signature {
    #[cfg(feature = "sig-md5")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "sig-md5")))]
    Md5(md5::Md5),
    #[cfg(feature = "sig-sha1")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "sig-sha1")))]
    Sha1(sha1::Sha1),
    #[cfg(feature = "sig-sha2")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "sig-sha2")))]
    Sha256(sha2::Sha256),
    #[cfg(feature = "sig-sha2")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "sig-sha2")))]
    Sha512(sha2::Sha512),
}

impl Signature {
    /// Creates a signature from the phar format flag
    pub fn from_u32(discrim: u32) -> Option<Signature> {
        Some(match discrim {
            #[cfg(feature = "sig-md5")]
            1 => Self::Md5(Digest::new()),
            #[cfg(feature = "sig-sha1")]
            2 => Self::Sha1(Digest::new()),
            #[cfg(feature = "sig-sha2")]
            4 => Self::Sha256(Digest::new()),
            #[cfg(feature = "sig-sha2")]
            8 => Self::Sha512(Digest::new()),
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
            Self::Sha256(_) => 4,
            #[cfg(feature = "sig-sha2")]
            Self::Sha512(_) => 8,
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
    pub fn write(&mut self) -> &mut dyn Write {
        match self {
            #[cfg(feature = "sig-md5")]
            Self::Md5(digest) => digest,
            #[cfg(feature = "sig-sha1")]
            Self::Sha1(digest) => digest,
            #[cfg(feature = "sig-sha2")]
            Self::Sha256(digest) => digest,
            #[cfg(feature = "sig-sha2")]
            Self::Sha512(digest) => digest,
        }
    }

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
    pub fn write(&mut self) -> &mut dyn Write {
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
