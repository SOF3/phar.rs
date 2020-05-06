use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{
    entry::{self, FileEntry},
    error, util,
};

pub struct Parsed {
    pub(crate) num_files: u32,
    pub(crate) api: u16,
    pub(crate) flags: PharFlags,
    pub(crate) alias: Vec<u8>,
    pub(crate) metadata: Vec<u8>,
    pub(crate) entries: Vec<FileEntry>,
}

bitflags::bitflags! {
    pub struct PharFlags : u32 {
        const HAS_VERIFICATION = 0x0001_0000;
        const HAS_ZLIB = 0x0000_1000;
        const HAS_BZIP = 0x0000_2000;
    }
}

pub fn read(mut file: impl Read) -> Result<Parsed, error::Open> {
    let num_files = file.read_u32::<LittleEndian>()?;
    let api = file.read_u16::<LittleEndian>()?;
    let flags = PharFlags::from_bits(file.read_u32::<LittleEndian>()? & PharFlags::all().bits())
        .expect("Filtered by `& PharFlags::all().bits()`");
    let alias = util::read_bstr(&mut file)?;
    let metadata = util::read_bstr(&mut file)?;
    let entries = (0..num_files)
        .map(|_| entry::read(&mut file))
        .collect::<Result<_, _>>()?;
    let manifest = Parsed {
        num_files,
        api,
        flags,
        alias,
        metadata,
        entries,
    };
    Ok(manifest)
}
