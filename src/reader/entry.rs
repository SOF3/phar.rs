use std::io::{Read, Result, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use super::Section;

pub struct Entry {
    pub name: Section,
    pub original_file_size: u32,
    pub time: i32,
    pub compressed_file_size: u32,
    pub original_crc32: u32,
    pub flags: u32,
    pub metadata: Section,
}

impl Entry {
    pub fn parse(
        read: &mut (impl Read + Seek),
        cache_name: bool,
        cache_metadata: bool,
    ) -> Result<Self> {
        let name_len = read.read_u32::<LittleEndian>()?;
        let mut name = Section::create(cache_name, read.seek(SeekFrom::Current(0))?);
        name.from_read(read, name_len)?;

        let original_file_size = read.read_u32::<LittleEndian>()?;
        let time = read.read_i32::<LittleEndian>()?;
        let compressed_file_size = read.read_u32::<LittleEndian>()?;
        let original_crc32 = read.read_u32::<LittleEndian>()?;
        let flags = read.read_u32::<LittleEndian>()?;

        let metadata_len = read.read_u32::<LittleEndian>()?;
        let mut metadata = Section::create(cache_metadata, read.seek(SeekFrom::Current(0))?);
        metadata.from_read(read, metadata_len)?;

        Ok(Entry {
            name,
            original_file_size,
            time,
            compressed_file_size,
            original_crc32,
            flags,
            metadata,
        })
    }
}
