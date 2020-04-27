use std::io::{self, ErrorKind, Read};

use crate::error;
use crate::util;

/// Reads the stub and manifest length of a phar file
///
/// The read stream should start at the beginning of a phar file.
/// When the method returns successfully,
/// the file pointer should be located at offset 4 of phar manifest.
pub fn read(file: &mut impl Read) -> Result<Vec<u8>, error::Open> {
    let mut stub = vec![0u8; 5];
    file.read_exact(&mut stub[..])?;
    if &stub != b"<?php" {
        return Err(error::Open::IncorrectStubStart);
    }

    match util::read_until_bstr(file, &mut stub, b"__HALT_COMPILER(); ?>\r\n") {
        Ok(()) => (),
        Err(err) if err.kind() == ErrorKind::UnexpectedEof => {
            return Err(error::Open::NoHaltCompiler)
        }
        Err(err) => return Err(error::Open::Io { inner: err }),
    }

    Ok(stub)
}
