use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

use phar::{Compression, Signature};

#[test]
pub fn test_plain() -> io::Result<()> {
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dir.push("tests/output");
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(dir.join("test_plain.phar"))?;

    let mut writer = phar::create(&mut file, Signature::sha512())
        .stub(&b"<?php "[..])?
        .metadata(&b""[..])?;
    writer.entry(
        &b"foo"[..],
        &b""[..],
        SystemTime::now(),
        0o664,
        Compression::None,
    )?;
    writer.entry(
        &b"qux"[..],
        &b""[..],
        SystemTime::now(),
        0o664,
        Compression::None,
    )?;
    let mut contents = writer.contents()?;
    contents.feed(&b"bar"[..])?;
    contents.feed(&b"corge"[..])?;

    Ok(())
}

#[test]
pub fn test_zlib() -> io::Result<()> {
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dir.push("tests/output");
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(dir.join("test_zlib.phar"))?;

    let mut writer = phar::create(&mut file, Signature::sha256())
        .stub(&b"<?php "[..])?
        .metadata(&b""[..])?;
    writer.entry(
        &b"foo"[..],
        &b""[..],
        SystemTime::now(),
        0o664,
        Compression::Zlib(9),
    )?;
    writer.entry(
        &b"qux"[..],
        &b""[..],
        SystemTime::now(),
        0o664,
        Compression::Zlib(9),
    )?;
    let mut contents = writer.contents()?;
    contents.feed(&b"bar"[..])?;
    contents.feed(&b"corge"[..])?;

    Ok(())
}

#[test]
pub fn test_bzip() -> io::Result<()> {
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dir.push("tests/output");
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(dir.join("test_bzip.phar"))?;

    let mut writer = phar::create(&mut file, Signature::sha256())
        .stub(&b"<?php "[..])?
        .metadata(&b""[..])?;
    writer.entry(
        &b"foo"[..],
        &b""[..],
        SystemTime::now(),
        0o664,
        Compression::Bzip(9),
    )?;
    writer.entry(
        &b"qux"[..],
        &b""[..],
        SystemTime::now(),
        0o664,
        Compression::Bzip(9),
    )?;
    let mut contents = writer.contents()?;
    contents.feed(&b"bar"[..])?;
    contents.feed(&b"corge"[..])?;

    Ok(())
}

#[test]
pub fn test_mixed() -> io::Result<()> {
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dir.push("tests/output");
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(dir.join("test_mixed.phar"))?;

    let mut writer = phar::create(&mut file, Signature::sha256())
        .stub(&b"<?php "[..])?
        .metadata(&b""[..])?;
    writer.entry(
        &b"foo"[..],
        &b""[..],
        SystemTime::now(),
        0o664,
        Compression::Zlib(9),
    )?;
    writer.entry(
        &b"qux"[..],
        &b""[..],
        SystemTime::now(),
        0o664,
        Compression::Bzip(9),
    )?;
    let mut contents = writer.contents()?;
    contents.feed(&b"bar"[..])?;
    contents.feed(&b"corge"[..])?;

    Ok(())
}
