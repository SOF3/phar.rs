use std::env;
use std::fs;
use std::io::Result;
use std::path::PathBuf;

use phar::{read, Reader};

#[test]
fn test_plain() -> Result<()> {
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dir.push("data");

    let reader = Reader::<_, read::index::NoIndex>::read(
        fs::File::open(dir.join("plain.phar"))?,
        read::Options::builder().build(),
    );

    Ok(())
}
