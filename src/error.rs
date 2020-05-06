use snafu::Snafu;

use crate::signature;

/// Errors related to opening a phar
#[derive(Debug, Snafu)]
#[allow(variant_size_differences)]
pub enum Open {
    #[snafu(display("IO error: {}", inner))]
    Io { inner: std::io::Error },
    #[snafu(display("Stub must start with <?php; possibly file corruption"))]
    IncorrectStubStart,
    #[snafu(display("Stub must be terminated by __HALT_COMPILER(); possibly file corruption"))]
    NoHaltCompiler,
    #[snafu(display("Declared manifest length is longer than 1 MB, possibly file corruption"))]
    ManifestTooLong,
    #[snafu(display("File contains more content bytes than declared"))]
    ContentTooLong,
    #[snafu(display("Unsupported signature type; possibly file corruption"))]
    UnknownSignatureType,
    #[snafu(display("{}", inner))]
    UnsupportedSignatureType {
        inner: signature::UnsupportedSignature,
    },
    #[snafu(display("Phar has a broken signature"))]
    BrokenSignature,
}

impl From<std::io::Error> for Open {
    fn from(inner: std::io::Error) -> Self {
        Self::Io { inner }
    }
}
