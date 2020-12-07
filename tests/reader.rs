use std::env;
use std::fs;
use std::io::{self, Result};
use std::path::PathBuf;

use phar::{read, Reader};

#[test]
fn test_plain_no_index() -> Result<()> {
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dir.push("tests/data");

    let mut reader = Reader::<_, read::index::NoIndex>::read(
        fs::File::open(dir.join("plain.phar"))?,
        read::Options::builder().build(),
    )?;
    verify_std_case(&mut reader);

    Ok(())
}

#[test]
fn test_zip_no_index() -> Result<()> {
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dir.push("tests/data");

    let mut reader = Reader::<_, read::index::NoIndex>::read(
        fs::File::open(dir.join("zip.phar"))?,
        read::Options::builder().build(),
    )?;
    verify_std_case(&mut reader);

    Ok(())
}

#[test]
fn test_zip_offset_only() -> Result<()> {
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dir.push("tests/data");

    let mut reader = Reader::<_, read::index::OffsetOnly>::read(
        fs::File::open(dir.join("zip.phar"))?,
        read::Options::builder().build(),
    )?;
    verify_std_case(&mut reader);

    Ok(())
}

fn verify_std_case<R: io::Read + io::Seek, T: phar::read::FileIndex>(phar: &mut Reader<R, T>) {
    assert_eq!(
        phar.stub_bytes().expect("cannot read phar stub").as_ref(),
        b"<?php __HALT_COMPILER(); ?>\r\n"
    );
    assert_eq!(
        phar.metadata_bytes()
            .expect("cannot read phar stub")
            .as_ref(),
        br#"s:3:"met";"#
    );
}
