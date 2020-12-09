use std::io::{self, Seek, SeekFrom};

/// Stub terminator
pub const STUB_TERMINATOR: &[u8] = b"__HALT_COMPILER(); ?>\r\n";

/// Stub terminator
pub const PHAR_TERMINATOR: &[u8] = b"GBMB";

pub fn tell(mut seek: impl Seek) -> io::Result<u64> {
    seek.seek(SeekFrom::Current(0))
}
