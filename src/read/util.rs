use std::io::{Error, ErrorKind, Read, Result};

use itertools::{multipeek, MultiPeek};

use super::Section;

pub fn read_find_bstr(file: &mut impl Read, buf: &mut Section, bstr: &[u8]) -> Result<()> {
    fn multi_peek_starts_with(
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
                Some(&Ok(_)) => return Ok(false),
            }
        }
        Ok(true)
    }

    debug_assert_ne!(bstr.len(), 0, "SSearching for empty string is nonsense");

    let mut iter = multipeek(file.bytes());
    loop {
        if multi_peek_starts_with(&mut iter, bstr)? {
            for _ in 0..bstr.len() {
                let next = iter
                    .next()
                    .expect("EOF was thrown in multi_peek_starts_with()")
                    .expect("Err values were checked in multi_peek_starts_with()");
                buf.feed(&[next]);
            }
            return Ok(());
        }

        let next = iter
            .next()
            .expect("EOF was thrown in multi_peek_starts_with()")
            .expect("Err values were checked in multi_peek_starts_with()");
        buf.feed(&[next]);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::super::Section;

    #[test]
    fn read_until_bstr() {
        let haystack = b"mississippi";

        for needle in (1..=7).flat_map(|len| haystack.windows(len)) {
            let offset = haystack
                .windows(needle.len())
                .position(|substr| substr == needle)
                .expect("needle was extracted from haystack");

            let buf = Vec::with_capacity(haystack.len());
            let mut section = Section::Cached(buf);
            super::read_find_bstr(&mut Cursor::new(haystack.iter()), &mut section, needle)
                .unwrap_or_else(|_| {
                    panic!("Failed to find needle {}", String::from_utf8_lossy(needle))
                });

            let buf = match section {
                Section::Cached(buf) => buf,
                _ => unreachable!(),
            };

            assert_eq!(haystack.get(0..offset + needle.len()), Some(&buf[..]));
        }
    }
}
