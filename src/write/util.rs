use std::convert::TryFrom;
use std::io::{self, Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::util::tell;

pub fn write_bstr(
    mut stream: impl Write + Seek,
    mut bstr: impl Read,
    error_msg: &str,
) -> Result<()> {
    let start = tell(&mut stream)?;
    stream.write_all(&[0u8; 4])?; // alias size
    let size = io::copy(&mut bstr, &mut stream)?;
    let size = u32::try_from(size).map_err(|_| Error::new(ErrorKind::Other, error_msg))?;
    let _ = stream.seek(SeekFrom::Start(start))?;
    stream.write_u32::<LittleEndian>(size)?;
    let _ = stream.seek(SeekFrom::Current(size.into()))?;
    Ok(())
}

pub struct MultiWrite<I>(pub I);

impl<I, W: Write> Write for MultiWrite<I>
where
    for<'t> &'t mut I: IntoIterator<Item = &'t mut W>,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        for write in &mut self.0 {
            write.write_all(buf)?;
        }
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        for write in &mut self.0 {
            write.write_all(buf)?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        for write in &mut self.0 {
            write.flush()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Crc32Writer(crc32fast::Hasher);

impl Crc32Writer {
    pub fn finish(self) -> u32 {
        self.0.finalize()
    }
}

impl Write for Crc32Writer {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.0.update(buf);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
