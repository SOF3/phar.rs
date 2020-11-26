use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use std::io::{self, Read, Result, Seek, SeekFrom, Write};

#[derive(Clone)]
pub enum Section {
    Cached(Vec<u8>),
    Offset(u64, u64),
}

impl Section {
    pub fn create(cache: bool, start: u64) -> Self {
        if cache {
            Self::Cached(Vec::new())
        } else {
            Self::Offset(start, start)
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        match self {
            Self::Cached(vec) => vec.extend(bytes),
            Self::Offset(_, end) => {
                *end += u64::try_from(bytes.len()).expect("usize <= u64");
            }
        }
    }

    pub fn from_read(&mut self, read: &mut (impl Read + Seek), len: u32) -> Result<()> {
        match self {
            Self::Cached(vec) => {
                let _ = read
                    .take(len.try_into().expect("u32 <= usize"))
                    .read_to_end(vec)?;
            }
            Self::Offset(_, end) => {
                let _ = read.seek(SeekFrom::Current(len.into()))?;
                let len: u64 = len.into();
                *end += len;
            }
        }
        Ok(())
    }

    pub fn len(&self) -> u64 {
        match self {
            Self::Cached(vec) => vec.len().try_into().expect("usize <= u64"),
            Self::Offset(start, end) => end - start,
        }
    }

    pub fn copy_value(&self, read: &mut (impl Read + Seek), mut write: impl Write) -> Result<()> {
        match self {
            Self::Cached(vec) => {
                write.write_all(&vec[..])?;
            }
            Self::Offset(start, end) => {
                let _ = read.seek(SeekFrom::Start(*start))?;
                let _ = io::copy(&mut read.take(end - start), &mut write)?;
            }
        }
        Ok(())
    }

    pub fn as_memory<'t>(&'t self, read: &mut (impl Read + Seek)) -> Result<impl AsRef<[u8]> + 't> {
        Ok(match self {
            Self::Cached(vec) => Cow::Borrowed(&vec[..]),
            Self::Offset(start, end) => {
                // overflow is impossible for filenames because they are u32
                let size = (end - start)
                    .try_into()
                    .expect("section is too large to fit in memory");
                let mut vec = Vec::with_capacity(size);
                self.copy_value(read, &mut vec)?;
                Cow::Owned(vec)
            }
        })
    }
}
