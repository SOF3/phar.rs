#[allow(clippy::module_inception)]
mod reader;
pub use reader::{Options, Reader};

mod section;
use section::Section;

pub mod index;
pub use index::FileIndex;

mod entry;
use entry::Entry;

mod util;
