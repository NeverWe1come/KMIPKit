//! Checked, bounded parsing of a single TTLV item.

use std::str;

use super::{DecodeError, DecodeErrorKind};
use crate::{Item, ModelError, RawTag, Structure, Tag, Value};

const HEADER_LENGTH: usize = 8;
const DEFAULT_MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_MAX_STRUCTURE_DEPTH: usize = 64;
const DEFAULT_MAX_ELEMENTS: usize = 100_000;

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

    let header = parse_item_header(bytes, start, parent_end, parent_structure_depth)?;
    let structure_depth = structure_depth_for(header.item_kind, parent_structure_depth, start)?;
    let span = item_span(&header, start)?;
    validate_parent_boundary(&span, parent_end, parent_structure_depth)?;
    let value_bytes = bytes
        .get(span.value_start..span.value_end)
        .ok_or_else(|| DecodeError::new(DecodeErrorKind::TruncatedValue, span.value_start))?;
    let value = decode_value(
        bytes,
        span.value_start,
        span.value_end,
        value_bytes,
        header.item_kind,
        structure_depth,
        state,
    )?;
    let item = Item::new(header.tag, value)
        .map_err(|_| DecodeError::new(DecodeErrorKind::ModelConstraint, start))?;
    Ok((item, span.item_end))
}

struct ItemHeader {
    tag: Tag,
    item_kind: ItemKind,
    item_length: usize,
    value_start: usize,
    length_offset: usize,
}

fn parse_item_header(
    bytes: &[u8],
    start: usize,
    parent_end: usize,
    parent_structure_depth: usize,
) -> Result<ItemHeader, DecodeError> {
    let header_end = checked_end_offset(
        start,
        HEADER_LENGTH,
        DecodeErrorKind::TruncatedHeader,
        start,
    )?;
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
    let item_type_offset = checked_end_offset(start, 3, DecodeErrorKind::TruncatedHeader, start)?;
    let length_offset = checked_end_offset(start, 4, DecodeErrorKind::TruncatedHeader, start)?;
    let tag_value =
        (u32::from(header[0]) << 16) | (u32::from(header[1]) << 8) | u32::from(header[2]);
    let item_type = header[3];
    let item_length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
    let item_length = usize::try_from(item_length)
        .map_err(|_| DecodeError::new(DecodeErrorKind::TruncatedValue, length_offset))?;
    let tag = checked_tag(tag_value, start)?;
    let item_kind = ItemKind::from_wire(item_type)
        .ok_or_else(|| DecodeError::new(DecodeErrorKind::UnsupportedItemType, item_type_offset))?;
    validate_item_length(item_kind, item_length, length_offset)?;

    Ok(ItemHeader {
        tag,
        item_kind,
        item_length,
        value_start: header_end,
        length_offset,
    })
}

fn checked_tag(tag_value: u32, offset: usize) -> Result<Tag, DecodeError> {
    let raw_tag = RawTag::new(tag_value)
        .map_err(|_| DecodeError::new(DecodeErrorKind::UnallocatedTag, offset))?;
    raw_tag.try_checked().map_err(|error| {
        let kind = match error {
            ModelError::TagNotAllocated if raw_tag.is_reserved() => DecodeErrorKind::ReservedTag,
            ModelError::TagNotAllocated | ModelError::RawTagOutOfRange => {
                DecodeErrorKind::UnallocatedTag
            }
            ModelError::StructureDepthExceeded => DecodeErrorKind::ModelConstraint,
        };
        DecodeError::new(kind, offset)
    })
}

fn structure_depth_for(
    item_kind: ItemKind,
    parent_structure_depth: usize,
    offset: usize,
) -> Result<usize, DecodeError> {
    if item_kind != ItemKind::Structure {
        return Ok(parent_structure_depth);
    }

    let depth = parent_structure_depth
        .checked_add(1)
        .ok_or_else(|| DecodeError::new(DecodeErrorKind::StructureDepthExceeded, offset))?;
    if depth > DEFAULT_MAX_STRUCTURE_DEPTH {
        return Err(DecodeError::new(
            DecodeErrorKind::StructureDepthExceeded,
            offset,
        ));
    }
    Ok(depth)
}

struct ItemSpan {
    item_start: usize,
    value_start: usize,
    value_end: usize,
    item_end: usize,
}

fn item_span(header: &ItemHeader, item_start: usize) -> Result<ItemSpan, DecodeError> {
    let value_end = checked_end_offset(
        header.value_start,
        header.item_length,
        DecodeErrorKind::TruncatedValue,
        header.length_offset,
    )?;
    let item_end = checked_end_offset(
        value_end,
        padding_length(header.item_kind, header.item_length),
        DecodeErrorKind::TruncatedValue,
        header.length_offset,
    )?;
    Ok(ItemSpan {
        item_start,
        value_start: header.value_start,
        value_end,
        item_end,
    })
}

fn validate_parent_boundary(
    span: &ItemSpan,
    parent_end: usize,
    parent_structure_depth: usize,
) -> Result<(), DecodeError> {
    if span.item_end > parent_end && parent_structure_depth > 0 {
        return Err(DecodeError::new(
            DecodeErrorKind::StructureBoundary,
            span.item_start,
        ));
    }
    if span.value_end > parent_end {
        return Err(DecodeError::new(
            DecodeErrorKind::TruncatedValue,
            span.value_start,
        ));
    }
    if span.item_end > parent_end {
        return Err(DecodeError::new(
            DecodeErrorKind::InvalidPaddingExtent,
            span.value_end,
        ));
    }
    Ok(())
}

fn checked_end_offset(
    offset: usize,
    extent: usize,
    error_kind: DecodeErrorKind,
    error_offset: usize,
) -> Result<usize, DecodeError> {
    offset
        .checked_add(extent)
        .ok_or_else(|| DecodeError::new(error_kind, error_offset))
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
