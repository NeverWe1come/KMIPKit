//! Bounded decoding for one complete generic TTLV item.
//!
//! The decoder preserves Structure child order and represented values while
//! rejecting malformed framing, unsupported Item Types, and Tags that cannot
//! enter the checked generic model.

use std::fmt::{self, Display, Formatter};

mod decoder;

pub use decoder::decode;

/// The safe category of a TTLV decoding failure.
///
/// This value contains no input bytes or decoded payload.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeErrorKind {
    /// The complete input exceeds the default message-size limit.
    MessageTooLarge,
    /// The input ends before a complete eight-byte header is available.
    TruncatedHeader,
    /// The input ends before the declared Item Value is available.
    TruncatedValue,
    /// A type's declared Item Length is not valid for that type.
    InvalidItemLength,
    /// A Big Integer has zero Item Value octets.
    EmptyBigInteger,
    /// A Text String contains bytes that are not valid UTF-8.
    InvalidUtf8,
    /// A Boolean is not encoded as the defined eight-byte false or true value.
    InvalidBoolean,
    /// The Item Type byte is not one of the eleven represented by the model.
    UnsupportedItemType,
    /// Bytes remain after the single complete root item.
    TrailingBytes,
    /// A child item extends beyond its parent Structure boundary.
    StructureBoundary,
    /// A received Tag is classified as Reserved by the KMIP 2.1 catalog.
    ReservedTag,
    /// A Tag is not assigned or in an accepted extension range.
    UnallocatedTag,
    /// Required non-Structure padding bytes are missing.
    InvalidPaddingExtent,
    /// The nested Structure depth exceeds the generic model's limit.
    StructureDepthExceeded,
    /// The total number of Items exceeds the default element limit.
    ElementLimitExceeded,
    /// A decoder-owned payload reservation failed.
    AllocationFailed,
    /// A parsed value violates a generic model construction constraint.
    ModelConstraint,
}

impl Display for DecodeErrorKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MessageTooLarge => "message exceeds the byte limit",
            Self::TruncatedHeader => "truncated TTLV header",
            Self::TruncatedValue => "truncated TTLV value",
            Self::InvalidItemLength => "invalid TTLV Item Length",
            Self::EmptyBigInteger => "empty Big Integer value",
            Self::InvalidUtf8 => "Text String is not valid UTF-8",
            Self::InvalidBoolean => "invalid TTLV Boolean value",
            Self::UnsupportedItemType => "unsupported TTLV Item Type",
            Self::TrailingBytes => "bytes remain after the complete TTLV item",
            Self::StructureBoundary => "child item exceeds its Structure boundary",
            Self::ReservedTag => "received a Reserved Tag",
            Self::UnallocatedTag => "Tag is not allocated by the KMIP 2.1 catalog",
            Self::InvalidPaddingExtent => "invalid TTLV padding extent",
            Self::StructureDepthExceeded => "Structure depth exceeds the default limit",
            Self::ElementLimitExceeded => "item count exceeds the default limit",
            Self::AllocationFailed => "unable to reserve decoder-owned storage",
            Self::ModelConstraint => "decoded item violates a model constraint",
        };
        formatter.write_str(message)
    }
}

/// A payload-free TTLV decoding error with a safe input offset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeError {
    kind: DecodeErrorKind,
    offset: usize,
}

impl DecodeError {
    const fn new(kind: DecodeErrorKind, offset: usize) -> Self {
        Self { kind, offset }
    }

    /// Returns the safe category of this decoding failure.
    #[must_use]
    pub const fn kind(&self) -> DecodeErrorKind {
        self.kind
    }

    /// Returns the byte offset at which decoding detected the failure.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }
}

impl Display for DecodeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte offset {}", self.kind, self.offset)
    }
}

impl std::error::Error for DecodeError {}
