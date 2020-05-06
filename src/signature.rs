use std::io::{sink, Sink, Write};

use cfg_if::cfg_if;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureType {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    // TODO: support openssl signatures
}

impl SignatureType {
    pub fn from_save_id(flag: u32) -> Option<Self> {
        Some(match flag {
            0x0001 => Self::Md5,
            0x0002 => Self::Sha1,
            0x0004 => Self::Sha256,
            0x0008 => Self::Sha512,
            _ => return None,
        })
    }

    pub fn to_save_id(self) -> u32 {
        match self {
            Self::Md5 => 0x0001,
            Self::Sha1 => 0x0002,
            Self::Sha256 => 0x0004,
            Self::Sha512 => 0x0008,
        }
    }

    pub fn to_save_length(self) -> u64 {
        match self {
            Self::Md5 => 16,
            Self::Sha1 => 20,
            Self::Sha256 => 32,
            Self::Sha512 => 64,
        }
    }

    pub fn to_verifier(self) -> Result<Box<dyn SignatureVerifier>, UnsupportedSignature> {
        match self {
            Self::Md5 => {
                cfg_if! {
                    if #[cfg(feature = "sig-md5")] {
                        Ok(Box::new(Md5Verifier { write: md5::Context::new() }))
                    } else {
                        Err(UnsupportedSignature::Md5)
                    }
                }
            }
            Self::Sha1 => {
                cfg_if! {
                    if #[cfg(feature = "sig-sha1")] {
                        use sha1::Digest;
                        Ok(Box::new(DigestVerifier { write: sha1::Sha1::new() }))
                    } else {
                        Err(UnsupportedSignature::Sha1)
                    }
                }
            }
            Self::Sha256 | Self::Sha512 => {
                cfg_if! {
                    if #[cfg(feature = "sig-sha2")] {
                        use sha1::Digest;
                        match self {
                            Self::Sha256 => Ok(Box::new(DigestVerifier { write: sha2::Sha256::new() })),
                            Self::Sha512 => Ok(Box::new(DigestVerifier { write: sha2::Sha512::new() })),
                            _ => unreachable!(),
                        }
                    } else {
                        Err(UnsupportedSignature::Sha2)
                    }
                }
            }
        }
    }
}

#[derive(Debug, snafu::Snafu)]
pub enum UnsupportedSignature {
    #[snafu(display("Support for signature type md5 is disabled"))]
    Md5,
    #[snafu(display("Support for signature type sha1 is disabled"))]
    Sha1,
    #[snafu(display("Support for signature types sha256/sha512 is disabled"))]
    Sha2,
}

pub trait SignatureVerifier {
    fn write(&mut self) -> &mut dyn Write;
    fn verify(self: Box<Self>, sig: &[u8]) -> bool;
}

pub struct DummyVerifier {
    sink: Sink,
}

impl Default for DummyVerifier {
    fn default() -> Self {
        Self { sink: sink() }
    }
}

impl SignatureVerifier for DummyVerifier {
    fn write(&mut self) -> &mut dyn Write {
        &mut self.sink
    }

    fn verify(self: Box<Self>, _: &[u8]) -> bool {
        true
    }
}

cfg_if! {
    if #[cfg(feature = "md5")] {
        pub struct Md5Verifier {
            write: md5::Context,
        }

        impl SignatureVerifier for Md5Verifier {
            fn write(&mut self) -> &mut dyn Write {
                &mut self.write
            }

            fn verify(self: Box<Self>, sig: &[u8]) -> bool {
                <[u8; 16]>::from(self.write.compute()) == sig
            }
        }
    }
}

cfg_if! {
    if #[cfg(any(feature = "sig-sha1", feature = "sig-sha2"))] {
        pub struct DigestVerifier<W: Write + digest::Digest> {
            write: W,
        }

        impl<W: Write + sha2::Digest> SignatureVerifier for DigestVerifier<W> {
            fn write(&mut self) -> &mut dyn Write {
                &mut self.write
            }

            fn verify(self: Box<Self>, sig: &[u8]) -> bool {
                &self.write.result()[..] == sig
            }
        }
    }
}
