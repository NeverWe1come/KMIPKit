//! Bounded decoding for one complete generic TTLV item.
//!
//! The decoder preserves Structure child order and represented values while
//! rejecting malformed framing, unsupported Item Types, and Tags that cannot
//! enter the checked generic model. [`crate::codec::decode`] applies
//! [`crate::codec::CodecLimits::defaults`]; [`crate::codec::decode_with_limits`]
//! applies immutable per-call limits.
//!
//! Resource accounting is per decode call: total input size is checked before
//! traversal, each Item consumes an element slot before its header is parsed,
//! and Structure depth is checked once its Item Type is known. Checked spans
//! and available parent bounds are validated before a value slice is decoded.
//! Variable-length payloads are copied only after a fallible reservation; the
//! existing model constructors remain infallible allocation boundaries.

use std::fmt::{self, Display, Formatter};

mod decoder;

#[cfg(test)]
mod limits_tests;

pub use decoder::{decode, decode_with_limits};

/// Immutable per-call resource limits for TTLV decoding and internal encoding.
///
/// Message size and item count may be set to any `usize`, including zero.
/// Structure depth may be lowered to zero but cannot exceed the model maximum
/// of 64. Construct a value with [`CodecLimits::new`] or use
/// [`CodecLimits::defaults`]; this type has no setters or builders.
#[derive(Debug, Eq, PartialEq)]
pub struct CodecLimits {
    message_bytes: usize,
    structure_depth: usize,
    elements: usize,
}

impl CodecLimits {
    /// Default maximum complete input size: 16 MiB.
    pub const DEFAULT_MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
    /// Default maximum number of nested Structure Items.
    pub const DEFAULT_MAX_STRUCTURE_DEPTH: usize = 64;
    /// Default maximum number of Items, including the root.
    pub const DEFAULT_MAX_ELEMENTS: usize = 100_000;

    /// Creates immutable per-call limits.
    ///
    /// Message-size and item-count values, including zero, are accepted as
    /// configured. Structure depth may range from zero through
    /// [`CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH`].
    ///
    /// # Errors
    ///
    /// Returns a payload-free [`LimitsError`] when Structure depth exceeds the
    /// generic model's maximum depth.
    pub fn new(
        max_message_bytes: usize,
        max_structure_depth: usize,
        max_elements: usize,
    ) -> Result<Self, LimitsError> {
        if max_structure_depth > Self::DEFAULT_MAX_STRUCTURE_DEPTH {
            return Err(LimitsError { _private: () });
        }
        Ok(Self {
            message_bytes: max_message_bytes,
            structure_depth: max_structure_depth,
            elements: max_elements,
        })
    }

    /// Returns the default 16 MiB, 64 Structure, and 100,000 Item limits.
    #[must_use]
    pub const fn defaults() -> Self {
        Self {
            message_bytes: Self::DEFAULT_MAX_MESSAGE_BYTES,
            structure_depth: Self::DEFAULT_MAX_STRUCTURE_DEPTH,
            elements: Self::DEFAULT_MAX_ELEMENTS,
        }
    }

    /// Returns the maximum complete input size for one decode call.
    #[must_use]
    pub const fn max_message_bytes(&self) -> usize {
        self.message_bytes
    }

    /// Returns the maximum nested Structure depth for one decode call.
    #[must_use]
    pub const fn max_structure_depth(&self) -> usize {
        self.structure_depth
    }

    /// Returns the maximum number of Items, including the root, for one decode call.
    #[must_use]
    pub const fn max_elements(&self) -> usize {
        self.elements
    }
}

/// A redacted error returned when [`CodecLimits`] cannot be constructed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LimitsError {
    _private: (),
}

impl Display for LimitsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("Structure depth cannot exceed 64")
    }
}

impl std::error::Error for LimitsError {}

/// The safe category of a TTLV decoding failure.
///
/// This value contains no input bytes or decoded payload.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeErrorKind {
    /// The complete input exceeds the per-call message-size limit.
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
    /// The nested Structure depth exceeds the per-call limit.
    StructureDepthExceeded,
    /// The total number of Items exceeds the per-call limit.
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
            Self::StructureDepthExceeded => "Structure depth exceeds the configured limit",
            Self::ElementLimitExceeded => "item count exceeds the configured limit",
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
