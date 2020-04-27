use std::{fmt, fs, io};

use cfg_if::cfg_if;
use derive_builder::Builder;

#[derive(Debug, Default, Builder)]
#[builder(pattern = "owned")]
pub struct Options {
    buffering: BufferPolicy,
}

pub enum BufferPolicy {
    None,
    Memory,
    Files(Box<dyn Fn() -> io::Result<fs::File>>),
}

impl fmt::Debug for BufferPolicy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => f.write_str("BufferPolicy::None"),
            Self::Memory => f.write_str("BufferPolicy::Memory"),
            Self::Files(closure) => f
                .debug_tuple("BufferPolicy::Files")
                .field(&format_args!("boxed closure"))
                .finish(),
        }
    }
}

impl Default for BufferPolicy {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(feature = "temp-buffer")] {
                BufferPolicy::Files(Box::new(tempfile::tempfile))
            } else {
                BufferPolicy::Memory
            }
        }
    }
}
