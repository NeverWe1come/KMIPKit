//! TTLV value types, encoding, and decoding for `KMIPKit`.
#![forbid(unsafe_code)]

mod error;
mod tag;

#[path = "generated/tag_allocations.rs"]
mod generated;

pub use error::ModelError;
pub use tag::{RawTag, Tag};
