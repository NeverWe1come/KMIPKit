//! Checked, bounded parsing of a single TTLV item.

use std::fmt::{self, Display, Formatter};
use std::str;

use crate::{Item, ModelError, RawTag, Structure, Value};

const HEADER_LENGTH: usize = 8;
const DEFAULT_MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_MAX_STRUCTURE_DEPTH: usize = 64;
const DEFAULT_MAX_ELEMENTS: usize = 100_000;

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

/// Decodes exactly one complete TTLV item using the default resource limits.
///
/// The default limits are 16 MiB per input, 64 nested Structures, and 100,000
/// total Items including the root. No input bytes are retained by the result
/// or by a decoding error.
///
/// # Errors
///
/// Returns a payload-free [`DecodeError`] for malformed input, unsupported or
/// unallocated Tags, extra trailing bytes, or an exceeded default limit.
pub fn decode(bytes: &[u8]) -> Result<Item, DecodeError> {
    if bytes.len() > DEFAULT_MAX_MESSAGE_BYTES {
        return Err(DecodeError::new(DecodeErrorKind::MessageTooLarge, 0));
    }

    let mut state = DecodeState { elements: 0 };
    let (item, end) = decode_item(bytes, 0, bytes.len(), 0, &mut state)?;
    if end != bytes.len() {
        return Err(DecodeError::new(DecodeErrorKind::TrailingBytes, end));
    }
    Ok(item)
}

struct DecodeState {
    elements: usize,
}

impl DecodeState {
    fn consume_element(&mut self, offset: usize) -> Result<(), DecodeError> {
        let next = self
            .elements
            .checked_add(1)
            .ok_or_else(|| DecodeError::new(DecodeErrorKind::ElementLimitExceeded, offset))?;
        if next > DEFAULT_MAX_ELEMENTS {
            return Err(DecodeError::new(
                DecodeErrorKind::ElementLimitExceeded,
                offset,
            ));
        }
        self.elements = next;
        Ok(())
    }
}

fn decode_item(
    bytes: &[u8],
    start: usize,
    parent_end: usize,
    parent_structure_depth: usize,
    state: &mut DecodeState,
) -> Result<(Item, usize), DecodeError> {
    state.consume_element(start)?;

    let header_end = start
        .checked_add(HEADER_LENGTH)
        .ok_or_else(|| DecodeError::new(DecodeErrorKind::TruncatedHeader, start))?;
    if header_end > parent_end {
        let kind = if parent_structure_depth > 0 {
            DecodeErrorKind::StructureBoundary
        } else {
            DecodeErrorKind::TruncatedHeader
        };
        return Err(DecodeError::new(kind, start));
    }
    let header = bytes
        .get(start..header_end)
        .ok_or_else(|| DecodeError::new(DecodeErrorKind::TruncatedHeader, start))?;
    let item_type_offset = start
        .checked_add(3)
        .ok_or_else(|| DecodeError::new(DecodeErrorKind::TruncatedHeader, start))?;
    let item_length_offset = start
        .checked_add(4)
        .ok_or_else(|| DecodeError::new(DecodeErrorKind::TruncatedHeader, start))?;

    let tag_value =
        (u32::from(header[0]) << 16) | (u32::from(header[1]) << 8) | u32::from(header[2]);
    let item_type = header[3];
    let item_length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
    let item_length = usize::try_from(item_length)
        .map_err(|_| DecodeError::new(DecodeErrorKind::TruncatedValue, item_length_offset))?;

    let raw_tag = RawTag::new(tag_value)
        .map_err(|_| DecodeError::new(DecodeErrorKind::UnallocatedTag, start))?;
    let tag = raw_tag.try_checked().map_err(|error| {
        let kind = match error {
            ModelError::TagNotAllocated if raw_tag.is_reserved() => DecodeErrorKind::ReservedTag,
            ModelError::TagNotAllocated | ModelError::RawTagOutOfRange => {
                DecodeErrorKind::UnallocatedTag
            }
            ModelError::StructureDepthExceeded => DecodeErrorKind::ModelConstraint,
        };
        DecodeError::new(kind, start)
    })?;

    let item_kind = ItemKind::from_wire(item_type)
        .ok_or_else(|| DecodeError::new(DecodeErrorKind::UnsupportedItemType, item_type_offset))?;
    validate_item_length(item_kind, item_length, item_length_offset)?;

    let structure_depth = if item_kind == ItemKind::Structure {
        let depth = parent_structure_depth
            .checked_add(1)
            .ok_or_else(|| DecodeError::new(DecodeErrorKind::StructureDepthExceeded, start))?;
        if depth > DEFAULT_MAX_STRUCTURE_DEPTH {
            return Err(DecodeError::new(
                DecodeErrorKind::StructureDepthExceeded,
                start,
            ));
        }
        depth
    } else {
        parent_structure_depth
    };

    let value_start = header_end;
    let value_end = value_start
        .checked_add(item_length)
        .ok_or_else(|| DecodeError::new(DecodeErrorKind::TruncatedValue, item_length_offset))?;
    let padding_length = padding_length(item_kind, item_length);
    let item_end = value_end
        .checked_add(padding_length)
        .ok_or_else(|| DecodeError::new(DecodeErrorKind::TruncatedValue, item_length_offset))?;

    if item_end > parent_end && parent_structure_depth > 0 {
        return Err(DecodeError::new(DecodeErrorKind::StructureBoundary, start));
    }
    if value_end > parent_end {
        return Err(DecodeError::new(
            DecodeErrorKind::TruncatedValue,
            value_start,
        ));
    }
    if item_end > parent_end {
        return Err(DecodeError::new(
            DecodeErrorKind::InvalidPaddingExtent,
            value_end,
        ));
    }

    let value_bytes = bytes
        .get(value_start..value_end)
        .ok_or_else(|| DecodeError::new(DecodeErrorKind::TruncatedValue, value_start))?;
    let value = decode_value(
        bytes,
        value_start,
        value_end,
        value_bytes,
        item_kind,
        structure_depth,
        state,
    )?;
    let item = Item::new(tag, value)
        .map_err(|_| DecodeError::new(DecodeErrorKind::ModelConstraint, start))?;
    Ok((item, item_end))
}

fn decode_value(
    bytes: &[u8],
    value_start: usize,
    value_end: usize,
    value_bytes: &[u8],
    item_kind: ItemKind,
    structure_depth: usize,
    state: &mut DecodeState,
) -> Result<Value, DecodeError> {
    let value = match item_kind {
        ItemKind::Structure => {
            let mut structure = Structure::new();
            let mut cursor = value_start;
            while cursor < value_end {
                let (child, child_end) =
                    decode_item(bytes, cursor, value_end, structure_depth, state)?;
                structure
                    .try_push(child)
                    .map_err(|_| DecodeError::new(DecodeErrorKind::ModelConstraint, cursor))?;
                cursor = child_end;
            }
            if cursor != value_end {
                return Err(DecodeError::new(DecodeErrorKind::StructureBoundary, cursor));
            }
            Value::structure(structure)
        }
        ItemKind::Integer => Value::integer(read_i32(value_bytes, value_start)?),
        ItemKind::LongInteger => Value::long_integer(read_i64(value_bytes, value_start)?),
        ItemKind::BigInteger => Value::big_integer(copy_payload(value_bytes, value_start)?),
        ItemKind::Enumeration => Value::enumeration(read_u32(value_bytes, value_start)?),
        ItemKind::Boolean => {
            let raw = read_u64(value_bytes, value_start)?;
            match raw {
                0 => Value::boolean(false),
                1 => Value::boolean(true),
                _ => {
                    return Err(DecodeError::new(
                        DecodeErrorKind::InvalidBoolean,
                        value_start,
                    ));
                }
            }
        }
        ItemKind::TextString => {
            str::from_utf8(value_bytes)
                .map_err(|_| DecodeError::new(DecodeErrorKind::InvalidUtf8, value_start))?;
            let owned = copy_payload(value_bytes, value_start)?;
            let text = String::from_utf8(owned)
                .map_err(|_| DecodeError::new(DecodeErrorKind::InvalidUtf8, value_start))?;
            Value::text_string(text)
        }
        ItemKind::ByteString => Value::byte_string(copy_payload(value_bytes, value_start)?),
        ItemKind::DateTime => Value::date_time(read_i64(value_bytes, value_start)?),
        ItemKind::Interval => Value::interval(read_u32(value_bytes, value_start)?),
        ItemKind::DateTimeExtended => {
            Value::date_time_extended(read_i64(value_bytes, value_start)?)
        }
    };
    Ok(value)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ItemKind {
    Structure,
    Integer,
    LongInteger,
    BigInteger,
    Enumeration,
    Boolean,
    TextString,
    ByteString,
    DateTime,
    Interval,
    DateTimeExtended,
}

impl ItemKind {
    fn from_wire(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Structure),
            0x02 => Some(Self::Integer),
            0x03 => Some(Self::LongInteger),
            0x04 => Some(Self::BigInteger),
            0x05 => Some(Self::Enumeration),
            0x06 => Some(Self::Boolean),
            0x07 => Some(Self::TextString),
            0x08 => Some(Self::ByteString),
            0x09 => Some(Self::DateTime),
            0x0A => Some(Self::Interval),
            0x0B => Some(Self::DateTimeExtended),
            _ => None,
        }
    }
}

fn validate_item_length(
    item_kind: ItemKind,
    length: usize,
    offset: usize,
) -> Result<(), DecodeError> {
    let valid = match item_kind {
        ItemKind::Structure => length.is_multiple_of(8),
        ItemKind::Integer | ItemKind::Enumeration | ItemKind::Interval => length == 4,
        ItemKind::LongInteger
        | ItemKind::Boolean
        | ItemKind::DateTime
        | ItemKind::DateTimeExtended => length == 8,
        ItemKind::BigInteger => {
            if length == 0 {
                return Err(DecodeError::new(DecodeErrorKind::EmptyBigInteger, offset));
            }
            length.is_multiple_of(8)
        }
        ItemKind::TextString | ItemKind::ByteString => true,
    };
    if valid {
        Ok(())
    } else {
        Err(DecodeError::new(DecodeErrorKind::InvalidItemLength, offset))
    }
}

fn padding_length(item_kind: ItemKind, value_length: usize) -> usize {
    match item_kind {
        ItemKind::Integer | ItemKind::Enumeration | ItemKind::Interval => 4,
        ItemKind::TextString | ItemKind::ByteString => (8 - value_length % 8) % 8,
        ItemKind::Structure
        | ItemKind::LongInteger
        | ItemKind::BigInteger
        | ItemKind::Boolean
        | ItemKind::DateTime
        | ItemKind::DateTimeExtended => 0,
    }
}

fn copy_payload(bytes: &[u8], offset: usize) -> Result<Vec<u8>, DecodeError> {
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(bytes.len())
        .map_err(|_| DecodeError::new(DecodeErrorKind::AllocationFailed, offset))?;
    owned.extend_from_slice(bytes);
    Ok(owned)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, DecodeError> {
    let value: [u8; 4] = bytes
        .try_into()
        .map_err(|_| DecodeError::new(DecodeErrorKind::InvalidItemLength, offset))?;
    Ok(u32::from_be_bytes(value))
}

fn read_i32(bytes: &[u8], offset: usize) -> Result<i32, DecodeError> {
    let value: [u8; 4] = bytes
        .try_into()
        .map_err(|_| DecodeError::new(DecodeErrorKind::InvalidItemLength, offset))?;
    Ok(i32::from_be_bytes(value))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, DecodeError> {
    let value: [u8; 8] = bytes
        .try_into()
        .map_err(|_| DecodeError::new(DecodeErrorKind::InvalidItemLength, offset))?;
    Ok(u64::from_be_bytes(value))
}

fn read_i64(bytes: &[u8], offset: usize) -> Result<i64, DecodeError> {
    let value: [u8; 8] = bytes
        .try_into()
        .map_err(|_| DecodeError::new(DecodeErrorKind::InvalidItemLength, offset))?;
    Ok(i64::from_be_bytes(value))
}
