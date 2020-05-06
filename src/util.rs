use std::convert::TryInto;
use std::io::{Error, ErrorKind, Read, Result};

use byteorder::{LittleEndian, ReadBytesExt};
use itertools::multipeek;
use itertools::structs::MultiPeek;

/// Read the BufRead until the first occurrence of bstr.
///
/// Returns error if any IO error occurred or if string is not found.
pub fn read_until_bstr(file: &mut impl Read, buf: &mut Vec<u8>, bstr: &[u8]) -> Result<()> {
    fn mp_starts_with(
        iter: &mut MultiPeek<impl Iterator<Item = Result<u8>>>,
        bstr: &[u8],
    ) -> Result<bool> {
        iter.reset_peek();
        for &expected in bstr {
            let actual: Option<&Result<u8>> = iter.peek();
            match actual {
                None => return Err(ErrorKind::UnexpectedEof.into()),
                Some(Err(err)) => return Err(Error::new(err.kind(), err.to_string())),
                Some(&Ok(actual)) if actual == expected => continue,
                Some(&Ok(actual)) => return Ok(false),
            }
        }
        Ok(true)
    }

    debug_assert_ne!(bstr.len(), 0, "SSearching for empty string is nonsense");

    let mut iter = multipeek(file.bytes());
    loop {
        if mp_starts_with(&mut iter, bstr)? {
            for _ in 0..bstr.len() {
                let next = iter
                    .next()
                    .expect("EOF was thrown in mp_starts_with()")
                    .expect("Err values were checked in mp_starts_with()");
                buf.push(next);
            }
            return Ok(());
        }

        let next = iter
            .next()
            .expect("EOF was thrown in mp_starts_with()")
            .expect("Err values were checked in mp_starts_with()");
        buf.push(next);
    }
}

pub fn read_bstr(file: &mut impl Read) -> Result<Vec<u8>> {
    let len = file
        .read_u32::<LittleEndian>()?
        .try_into()
        .expect("usize is smaller than 32 bits");
    let mut vec = vec![0u8; len];
    file.read_exact(&mut vec[..])?;
    Ok(vec)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    #[test]
    fn read_until_bstr() {
        let haystack = b"mississippi";
        let mut buf = Vec::with_capacity(haystack.len());

        for needle in (1..=7).flat_map(|len| haystack.windows(len)) {
            let offset = haystack
                .windows(needle.len())
                .position(|substr| substr == needle)
                .expect("needle was extracted from haystack");

            buf.clear();
            super::read_until_bstr(&mut Cursor::new(haystack.iter()), &mut buf, needle).expect(
                &format!("Failed to find needle {}", String::from_utf8_lossy(needle)),
            );

            assert_eq!(&haystack[0..offset + needle.len()], &buf[..]);
        }
    }
}
