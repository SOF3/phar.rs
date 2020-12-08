use std::collections::HashMap;
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

    verify_std_header(&mut reader);

    Ok(())
}

#[test]
fn test_plain_offset_only() -> Result<()> {
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dir.push("tests/data");

    let mut reader = Reader::<_, read::index::OffsetOnly>::read(
        fs::File::open(dir.join("plain.phar"))?,
        read::Options::builder().build(),
    )?;

    verify_std_header(&mut reader);
    verify_std_contents(&mut reader);

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

    verify_std_header(&mut reader);
    verify_std_contents(&mut reader);

    Ok(())
}

#[test]
fn test_zip_name_map() -> Result<()> {
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dir.push("tests/data");

    let mut reader = Reader::<_, read::index::NameHashMap>::read(
        fs::File::open(dir.join("zip.phar"))?,
        read::Options::builder().build(),
    )?;

    verify_std_header(&mut reader);
    verify_std_contents(&mut reader);

    Ok(())
}

#[test]
fn test_zip_metadata_map() -> Result<()> {
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dir.push("tests/data");

    let mut reader = Reader::<_, read::index::MetadataHashMap>::read(
        fs::File::open(dir.join("zip.phar"))?,
        read::Options::builder().build(),
    )?;

    verify_std_header(&mut reader);
    verify_std_contents(&mut reader);

    Ok(())
}


fn verify_std_header<R: io::Read + io::Seek, T: phar::read::FileIndex>(phar: &mut Reader<R, T>) {
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
fn verify_std_contents<R: io::Read + io::Seek, T: phar::read::index::Iterable>(
    phar: &mut Reader<R, T>,
) {
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

    let mut contents = HashMap::new();
    phar.for_each_file(|name, read| {
        let mut vec = vec![];
        read.read_to_end(&mut vec)?;
        contents.insert(name.to_vec(), vec);
        Ok(())
    })
    .expect("Failed reading phar contents");
    println!("{:?}", contents);

    assert_eq!(contents.len(), 2);
    assert_eq!(contents.get(&b"foo"[..]), Some(&b"bar".to_vec()));
    assert_eq!(contents.get(&b"qux"[..]), Some(&b"corge".to_vec()));
}
