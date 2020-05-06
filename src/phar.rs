use std::cmp;
use std::convert::{TryFrom, TryInto};
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};
use getset::{Getters, MutGetters, Setters};

use crate::signature::{DummyVerifier, SignatureType, SignatureVerifier};
use crate::{error, manifest, stub, AbstractEntry};

#[derive(Getters, Setters, MutGetters)]
pub struct Phar<F> {
    #[getset(get = "pub(crate)", get_mut = "pub(crate)")]
    file: F,
    #[getset(get, get_mut, set)]
    stub: Vec<u8>,
    #[getset(get = "pub(crate)", get_mut = "pub(crate)")]
    manifest: manifest::Parsed,
    #[getset(get = "pub(crate)", get_mut = "pub(crate)")]
    aents: Vec<AbstractEntry>,
}

impl Phar<File> {
    pub fn open(path: &impl AsRef<Path>, options: fs::OpenOptions) -> Result<Self, error::Open> {
        Ok(Self::open_stream(options.open(path)?)?)
    }
}

impl<F: Read + Seek> Phar<F> {
    pub fn open_stream(mut file: F) -> Result<Self, error::Open> {
        let mut file_sig_pos = file.seek(SeekFrom::End(-8))?;
        let sig_type = file.read_u32::<LittleEndian>()?;
        let mut gbmb = [0u8; 4];
        file.read_exact(&mut gbmb)?;

        let (sig_len, mut verifier) = if &gbmb == b"GBMB" {
            // signature is present
            let sig_type = SignatureType::from_save_id(file.read_u32::<LittleEndian>()?)
                .ok_or(error::Open::UnknownSignatureType)?;

            if file_sig_pos < sig_type.to_save_length() {
                return Err(error::Open::BrokenSignature);
            }
            file_sig_pos -= sig_type.to_save_length();
            // 8 bytes for type+gbmb
            (
                sig_type.to_save_length(),
                sig_type
                    .to_verifier()
                    .map_err(|err| error::Open::UnsupportedSignatureType { inner: err })?,
            )
        } else {
            // dummy signature verifier
            let verifier: Box<dyn SignatureVerifier> = Box::new(DummyVerifier::default());
            (0, verifier)
        };

        let stub;
        let manifest;
        let mut aents;
        {
            // mask file with reaves in this block
            let mut file = reaves::Reaves::new(&mut file, verifier.write());

            stub = stub::read(&mut file)?;

            let manifest_len = file.read_u32::<LittleEndian>()?;
            if manifest_len > (1 << 20) {
                return Err(error::Open::ManifestTooLong);
            }
            manifest = manifest::read((&mut file).take(manifest_len.into()))?;

            let content_offset = file.seek(SeekFrom::Current(0))?;
            aents = Vec::<AbstractEntry>::with_capacity(
                (manifest.num_files + 1)
                    .try_into()
                    .expect("usize is smaller than 32 bits"),
            );
            let mut offset = content_offset;
            for entry in manifest.entries.iter() {
                let length = u64::from(entry.compressed_size);
                aents.push(AbstractEntry::Entry { offset, length });
                offset += length;
            }

            if offset != file_sig_pos {
                return Err(error::Open::ContentTooLong);
            }

            // fast forward verify signature
            let mut current_offset = content_offset;
            let mut buf = vec![0u8; 8192];
            while current_offset < file_sig_pos {
                current_offset += u64::try_from(
                    #[allow(clippy::indexing_slicing)] // capped by 8192
                    file.read(
                        &mut buf[..cmp::min(
                            (file_sig_pos - current_offset)
                                .try_into()
                                .expect("usize::MAX_VALUE is smaller than 8192"),
                            8192,
                        )],
                    )?,
                )
                .expect("Value is bounded by 8192 < u64::MAX_VALUE");
            }
        }

        let mut sig = vec![
            0u8;
            sig_len
                .try_into()
                .expect("Signature length is a hardcoded small constant")
        ];
        file.read_exact(&mut sig[..])?;
        if !verifier.verify(&sig[..]) {
            return Err(error::Open::BrokenSignature);
        }

        Ok(Phar {
            file,
            stub,
            manifest,
            aents,
        })
    }
}

impl<F: Write> Phar<F> {
    pub fn create(file: F) -> io::Result<Self> {
        unimplemented!()
    }
}
