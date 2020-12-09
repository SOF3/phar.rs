#![cfg_attr(feature = "docsrs", feature(doc_cfg))]
#![warn(
    unused_results,
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
    clippy::unwrap_used
)]
#![cfg_attr(
    debug_assertions,
    allow(
        dead_code,
        unused_imports,
        unused_variables,
        unreachable_code,
        unused_qualifications,
    )
)]
#![cfg_attr(not(debug_assertions), deny(warnings, missing_docs, clippy::dbg_macro))]
#![cfg_attr(not(debug_assertions), allow(clippy::unknown_clippy_lints))]

//! A library for reading and writing files of the PHP phar format.
//!
//! Currently, this library only supports read-only and write-only styles.

#[cfg(feature = "reader")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "reader")))]
pub use read::Reader;

#[cfg(feature = "reader")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "reader")))]
pub mod read;

#[cfg(feature = "writer")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "writer")))]
pub use write::write;

#[cfg(feature = "writer")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "writer")))]
pub mod write;

mod signature;
pub use signature::Signature;

mod compression;
pub use compression::Compression;

mod util;
