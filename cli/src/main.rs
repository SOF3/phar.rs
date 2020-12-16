use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;

use structopt::StructOpt;

#[derive(StructOpt)]
enum Args {
    /// Reads or sets phar stub
    Stub {
        /// Path to the phar file
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        /// If set, the phar stub is changed to this argument
        new_value: Option<String>,
    },
    /// Reads or sets the alias
    Alias {
        /// Path to the phar file
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        /// If set, the phar alias is changed to this argument
        new_value: Option<String>,
    },
    /// Reads or sets the metadata
    Metadata {
        /// Path to the phar file
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        /// If set, the phar metadata is changed to this argument
        new_value: Option<String>,
    },
    /// Verifies the signature of a phar file
    Verify {
        /// Path to the phar file
        #[structopt(parse(from_os_str))]
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
    /// Re-signs a phar file with a different signature algorithm
    Sign {
        /// Path to the phar file
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        /// The new signature algorithm to use (md5, sha1, sha256, sha512)
        #[structopt(parse(try_from_str))]
        new_algorithm: SignatureAlgo,
    },
    /// Lists the files in a phar file
    #[structopt(alias = "l", alias = "ls")]
    List {
        /// Path to the phar file
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        /// The subdirectory to list (default ".")
        subdir: Option<String>,
        /// Whether to list files recursively
        #[structopt(short)]
        recursive: bool,
    },
    /// Adds files to a phar file, creating a new one if it does not already exist
    #[structopt(alias = "a", alias = "c", alias = "create")]
    Add {
        /// Path to the phar file
        #[structopt(parse(from_os_str))]
        dest: PathBuf,
        /// Paths of files to add
        sources: Vec<PathBuf>,
        /// Base directory in phar to add the sources.
        /// Incompatible with `--rename`.
        #[structopt(long)]
        base: Option<String>,
        /// Filename to add sources as.
        /// Only allowed if there is only one source.
        /// Incompatible with `--base`.
        #[structopt(long)]
        rename: Option<String>,
    },
}

enum SignatureAlgo {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

impl FromStr for SignatureAlgo {
    type Err = &'static str;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let mut clean = str.replace(|c: char| !c.is_ascii_alphanumeric(), "");
        clean.make_ascii_lowercase();
        Ok(match clean.as_str() {
            "md5" => Self::Md5,
            "sha1" => Self::Sha1,
            "sha256" => Self::Sha256,
            "sha512" => Self::Sha512,
            _ => return Err("unknown signature algorithm"),
        })
    }
}

fn main() -> Result<()> {
    let args = Args::from_args();
    Ok(())
}
