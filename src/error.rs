use snafu::Snafu;

/// Errors related to opening a phar
#[derive(Debug, Snafu)]
pub enum Open {
    #[snafu(display("IO error: {}", inner))]
    Io { inner: std::io::Error },
    #[snafu(display("Stub must start with <?php"))]
    IncorrectStubStart,
    #[snafu(display("Stub must be terminated by __HALT_COMPILER();"))]
    NoHaltCompiler,
    #[snafu(display("Declared manifest length is longer than 1 MB"))]
    ManifestTooLong,
    #[snafu(display("Unsupported signature type"))]
    UnknownSignatureType,
    #[snafu(display("Phar has a broken signature"))]
    BrokenSignature,
}

impl From<std::io::Error> for Open {
    fn from(inner: std::io::Error) -> Self {
        Self::Io { inner }
    }
}
