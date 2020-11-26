#[allow(clippy::module_inception)]
mod reader;
pub use reader::{Options, Reader};

mod section;
use section::Section;

mod index;
use index::FileIndex;

mod entry;
use entry::Entry;

mod util;
