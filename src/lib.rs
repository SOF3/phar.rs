#![warn(
    unused_qualifications,
    variant_size_differences,
    clippy::checked_conversions,
    clippy::needless_borrow,
    clippy::shadow_unrelated,
    clippy::wrong_pub_self_convention
)]
#![deny(
    anonymous_parameters,
    bare_trait_objects,
    clippy::as_conversions,
    clippy::clone_on_ref_ptr,
    clippy::float_cmp_const,
    clippy::if_not_else,
    clippy::indexing_slicing,
    clippy::option_unwrap_used,
    clippy::result_unwrap_used
)]
#![cfg_attr(
    debug_assertions,
    allow(
        dead_code,
        unused_imports,
        unused_variables,
        unreachable_code,
        unused_qualifications
    )
)]
#![cfg_attr(not(debug_assertions), deny(warnings, missing_docs, clippy::dbg_macro))]

//! A library for manipulating PHP phar format.

mod abstract_entry;
mod entry;
mod error;
mod manifest;
mod options;
mod phar;
mod signature;
mod stub;
mod util;

pub use entry::FileEntry;
pub use error::*;
pub use options::*;
pub use phar::Phar;

use abstract_entry::AbstractEntry;
