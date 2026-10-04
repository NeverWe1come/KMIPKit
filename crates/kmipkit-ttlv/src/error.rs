//! Payload-free errors produced by the TTLV model.

use std::fmt::{Display, Formatter};

/// A local construction error that never retains caller-provided tag values.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelError {
    /// The raw tag exceeds the 24-bit KMIP tag width.
    RawTagOutOfRange,
    /// The tag is reserved or not allocated by the KMIP 2.1 catalog policy.
    TagNotAllocated,
}

impl Display for ModelError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RawTagOutOfRange => formatter.write_str("raw tag exceeds the 24-bit range"),
            Self::TagNotAllocated => formatter.write_str("tag is reserved or unallocated"),
        }
    }
}

impl std::error::Error for ModelError {}
