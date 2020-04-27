use std::convert::TryInto;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{error, manifest, stub};

pub struct Phar<F> {
    file: F,
    stub: Vec<u8>,
    manifest: manifest::Manifest,
    offsets: Vec<u64>,
}

impl Phar<File> {
    pub fn open(path: &impl AsRef<Path>, options: fs::OpenOptions) -> Result<Self, error::Open> {
        Ok(Self::open_stream(options.open(path)?)?)
    }
}

impl<F: Read + Seek> Phar<F> {
    pub fn open_stream(mut file: F) -> Result<Self, error::Open> {
        let stub = stub::read(&mut file)?;

        let manifest_len = file.read_u32::<LittleEndian>()?;
        if manifest_len > (1 << 20) {
            return Err(error::Open::ManifestTooLong);
        }
        let manifest = manifest::read((&mut file).take(manifest_len.into()))?;

        let content_offset = file.seek(SeekFrom::Current(0))?;
        let mut offsets = vec![
            content_offset;
            (manifest.num_files + 1)
                .try_into()
                .expect("Architecture not supported")
        ];
        for (i, entry) in manifest.entries.iter().enumerate() {
            let offset = offsets
                .get(i)
                .expect("manifest.entries.len() == manifest.num_files")
                + u64::from(entry.compressed_size);
            *offsets
                .get_mut(i + 1)
                .expect("manifest.entries.len() + 1 == manifest.num_files + 1") = offset;
        }

        let sig_offset = file.seek(SeekFrom::End(-8))?;
        let sig_len = sig_offset
            - offsets
                .last()
                .expect("offsets.len() == manifest.num_files + 1 >= 1");

        let sig_type = SignatureType::from_save_id(file.read_u32::<LittleEndian>()?)
            .ok_or(error::Open::UnknownSignatureType)?;
        let mut gbmb = [0u8; 4];
        file.read_exact(&mut gbmb)?;
        if &gbmb != b"GBMB" {
            return Err(error::Open::BrokenSignature);
        }

        // TODO verify signature

        Ok(Phar {
            file,
            stub,
            manifest,
            offsets,
        })
    }
}

impl<F: Write> Phar<F> {
    pub fn create(file: F) -> io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureType {
    Md5,
    Sha1,
    Sha256,
    Sha512,
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
}
