#![cfg(test)]
//! Direct checks for decoder invariants that valid public inputs cannot violate.

use super::{
    CodecLimits, DecodeError, DecodeErrorKind, DecodeState, ItemHeader, ItemKind, ItemSpan,
    copy_payload, decode_item, item_span, parse_item_header, read_i32, read_i64, read_u32,
    read_u64, validate_parent_boundary,
};
use crate::{RawTag, Tag};

fn limits(message_bytes: usize) -> CodecLimits {
    CodecLimits::new(message_bytes, CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH, 16)
        .expect("bounded depth is valid")
}

fn error_kind<T>(result: &Result<T, DecodeError>) -> Option<DecodeErrorKind> {
    result.as_ref().err().map(DecodeError::kind)
}

fn assigned_tag() -> Tag {
    RawTag::new(0x0042_0173)
        .expect("tag fits its wire field")
        .try_checked()
        .expect("fixture tag is allocated")
}

fn wire_header(item_type: u8, item_length: u32) -> [u8; 8] {
    let length = item_length.to_be_bytes();
    [
        0x42, 0x01, 0x73, item_type, length[0], length[1], length[2], length[3],
    ]
}

fn state(limits: &CodecLimits) -> DecodeState<'_> {
    DecodeState {
        elements: 0,
        limits,
        observer: None,
    }
}

#[test]
fn checked_read_helpers_reject_wrong_widths() {
    assert_eq!(
        error_kind(&read_i32(&[], 3)),
        Some(DecodeErrorKind::InvalidItemLength)
    );
    assert_eq!(
        error_kind(&read_u32(&[], 3)),
        Some(DecodeErrorKind::InvalidItemLength)
    );
    assert_eq!(
        error_kind(&read_i64(&[], 3)),
        Some(DecodeErrorKind::InvalidItemLength)
    );
    assert_eq!(
        error_kind(&read_u64(&[], 3)),
        Some(DecodeErrorKind::InvalidItemLength)
    );
}

#[test]
fn bounded_payload_copy_rejects_over_limit_before_reserving() {
    let result = copy_payload(&[0x5a; 9], 4, &limits(8), None);
    assert_eq!(error_kind(&result), Some(DecodeErrorKind::MessageTooLarge));
}

#[test]
fn item_span_rejects_value_and_padding_offset_overflow() {
    let value_overflow = ItemHeader {
        tag: assigned_tag(),
        item_kind: ItemKind::TextString,
        item_length: usize::MAX,
        value_start: 8,
        length_offset: 4,
    };
    assert_eq!(
        error_kind(&item_span(&value_overflow, 0, &limits(usize::MAX))),
        Some(DecodeErrorKind::TruncatedValue)
    );

    let padding_overflow = ItemHeader {
        tag: assigned_tag(),
        item_kind: ItemKind::TextString,
        item_length: usize::MAX,
        value_start: 0,
        length_offset: 4,
    };
    assert_eq!(
        error_kind(&item_span(&padding_overflow, 0, &limits(usize::MAX))),
        Some(DecodeErrorKind::TruncatedValue)
    );
}

#[test]
fn parent_boundary_distinguishes_nested_truncation_and_missing_padding() {
    let complete_span = ItemSpan {
        item_start: 0,
        value_start: 8,
        value_end: 16,
        item_end: 16,
        exceeds_message_limit: false,
    };
    assert_eq!(
        error_kind(&validate_parent_boundary(&complete_span, 12, 1)),
        Some(DecodeErrorKind::StructureBoundary)
    );
    assert_eq!(
        error_kind(&validate_parent_boundary(&complete_span, 12, 0)),
        Some(DecodeErrorKind::TruncatedValue)
    );

    let missing_padding = ItemSpan {
        item_start: 0,
        value_start: 8,
        value_end: 12,
        item_end: 16,
        exceeds_message_limit: false,
    };
    assert_eq!(
        error_kind(&validate_parent_boundary(&missing_padding, 12, 0)),
        Some(DecodeErrorKind::InvalidPaddingExtent)
    );
}

#[test]
fn item_decoder_rejects_parent_inconsistency_and_limit_before_value_access() {
    let header = wire_header(0x08, 8);
    assert_eq!(
        error_kind(&decode_item(&header, 0, 16, 0, &mut state(&limits(16)))),
        Some(DecodeErrorKind::TruncatedValue)
    );

    let complete = [header.as_slice(), &[0; 8]].concat();
    assert_eq!(
        error_kind(&decode_item(&complete, 0, 16, 0, &mut state(&limits(8)))),
        Some(DecodeErrorKind::MessageTooLarge)
    );
}

#[test]
fn item_header_rejects_unavailable_and_parent_crossing_headers() {
    assert_eq!(
        error_kind(&parse_item_header(&[], 0, 8, 0)),
        Some(DecodeErrorKind::TruncatedHeader)
    );
    assert_eq!(
        error_kind(&parse_item_header(&[], 0, 0, 1)),
        Some(DecodeErrorKind::StructureBoundary)
    );
}
