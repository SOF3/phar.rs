use std::fs::File;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::path::PathBuf;

use crate::Phar;

pub enum AbstractEntry {
    Entry { offset: u64, length: u64 },
    Memory { buffer: Vec<u8> },
    Static { buffer: &'static [u8] },
    File { path: PathBuf },
}

impl AbstractEntry {
    pub fn read<'t, R: Read + Seek>(
        &'t self,
        phar: &'t mut Phar<R>,
    ) -> io::Result<Box<dyn Read + 't>> {
        match self {
            Self::Entry { offset, length } => {
                let file = phar.file_mut();
                file.seek(SeekFrom::Start(*offset))?;
                Ok(Box::new(file.take(*length)))
            }
            Self::Memory { buffer } => Ok(Box::new(io::Cursor::new(buffer))),
            Self::Static { buffer } => Ok(Box::new(io::Cursor::new(buffer))),
            Self::File { path } => Ok(Box::new(File::open(path)?)),
        }
    }
}
