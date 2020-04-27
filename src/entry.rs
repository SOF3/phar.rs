use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{error, util};

pub struct FileEntry {
    pub(crate) name: Vec<u8>,
    pub(crate) original_size: u32,
    pub(crate) timestamp: u32,
    pub(crate) compressed_size: u32,
    pub(crate) crc32: u32,
    pub(crate) flags: FileFlags,
    pub(crate) metadata: Vec<u8>,
}

pub struct FileFlags(pub u32);

const FLAG_HAS_ZLIB: u32 = 0x0000_1000;
const FLAG_HAS_BZIP: u32 = 0x0000_2000;

impl FileFlags {
    pub fn has_zlib(&self) -> bool {
        self.0 & FLAG_HAS_ZLIB > 0
    }

    pub fn set_has_zlib(&mut self, value: bool) {
        if value {
            self.0 |= FLAG_HAS_ZLIB;
        } else {
            self.0 &= !FLAG_HAS_ZLIB;
        }
    }

    pub fn has_bzip(&self) -> bool {
        self.0 & FLAG_HAS_BZIP > 0
    }

    pub fn set_has_bzip(&mut self, value: bool) {
        if value {
            self.0 |= FLAG_HAS_BZIP;
        } else {
            self.0 &= !FLAG_HAS_BZIP;
        }
    }

    pub fn mode(&self) -> u32 {
        self.0 & 0b111_111_111
    }

    pub fn set_mode(&mut self, mode: u32) {
        self.0 &= !0b111_111_111;
        self.0 |= mode & 0b111_111_111;
    }
}

pub fn read(mut file: impl Read) -> Result<FileEntry, error::Open> {
    let entry = FileEntry {
        name: util::read_bstr(&mut file)?,
        original_size: file.read_u32::<LittleEndian>()?,
        timestamp: file.read_u32::<LittleEndian>()?,
        compressed_size: file.read_u32::<LittleEndian>()?,
        crc32: file.read_u32::<LittleEndian>()?,
        flags: FileFlags(file.read_u32::<LittleEndian>()?),
        metadata: util::read_bstr(&mut file)?,
    };

    Ok(entry)
}
