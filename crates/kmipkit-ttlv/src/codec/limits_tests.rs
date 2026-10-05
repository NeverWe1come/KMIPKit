//! Decoder resource-limit tests tied to the production preflight helpers.

use std::cell::Cell;

use super::{CodecLimits, DecodeError, DecodeErrorKind, decode_with_limits};

#[derive(Default)]
pub(super) struct DecodeObserver {
    reservation_attempts: Cell<usize>,
    requested_reservation_bytes: Cell<u64>,
    peer_copy_calls: Cell<usize>,
}

impl DecodeObserver {
    pub(super) fn record_reservation(&self, requested_bytes: usize) {
        self.reservation_attempts
            .set(self.reservation_attempts.get().saturating_add(1));
        let requested_bytes = u64::try_from(requested_bytes).unwrap_or(u64::MAX);
        self.requested_reservation_bytes.set(
            self.requested_reservation_bytes
                .get()
                .saturating_add(requested_bytes),
        );
    }

    pub(super) fn record_peer_copy(&self) {
        self.peer_copy_calls
            .set(self.peer_copy_calls.get().saturating_add(1));
    }

    fn reservation_attempts(&self) -> usize {
        self.reservation_attempts.get()
    }

    fn requested_reservation_bytes(&self) -> u64 {
        self.requested_reservation_bytes.get()
    }

    fn peer_copy_calls(&self) -> usize {
        self.peer_copy_calls.get()
    }
}

fn limits(message_bytes: usize, structure_depth: usize, elements: usize) -> CodecLimits {
    CodecLimits::new(message_bytes, structure_depth, elements)
        .expect("bounded test values are valid CodecLimits")
}

fn error_kind<T>(result: &Result<T, DecodeError>) -> Option<DecodeErrorKind> {
    result.as_ref().err().map(DecodeError::kind)
}

fn wire_header(item_type: u8, item_length: u32) -> [u8; 8] {
    let length = item_length.to_be_bytes();
    [
        0x42, 0x01, 0x73, item_type, length[0], length[1], length[2], length[3],
    ]
}

fn empty_structure_item() -> Vec<u8> {
    wire_header(0x01, 0).to_vec()
}

fn structure_item(children: &[u8]) -> Vec<u8> {
    let length = u32::try_from(children.len()).expect("bounded test Structure fits U32");
    let mut item = wire_header(0x01, length).to_vec();
    item.extend_from_slice(children);
    item
}

fn integer_item() -> Vec<u8> {
    let mut item = wire_header(0x02, 4).to_vec();
    item.extend_from_slice(&[0, 0, 0, 1, 0, 0, 0, 0]);
    item
}

fn byte_string_item(value: &[u8]) -> Vec<u8> {
    let length = u32::try_from(value.len()).expect("bounded test Byte String fits U32");
    let padding = (8 - (value.len() % 8)) % 8;
    let mut item = wire_header(0x08, length).to_vec();
    item.extend_from_slice(value);
    item.resize(item.len() + padding, 0);
    item
}

#[test]
fn limits_getters_match_defaults_and_explicit_values() {
    // KMIPKit default and immutable per-call configuration: FR-007.
    let defaults = CodecLimits::defaults();
    assert_eq!(
        defaults.max_message_bytes(),
        CodecLimits::DEFAULT_MAX_MESSAGE_BYTES
    );
    assert_eq!(
        defaults.max_structure_depth(),
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH
    );
    assert_eq!(defaults.max_elements(), CodecLimits::DEFAULT_MAX_ELEMENTS);

    let explicit = limits(32, 7, 19);
    assert_eq!(explicit.max_message_bytes(), 32);
    assert_eq!(explicit.max_structure_depth(), 7);
    assert_eq!(explicit.max_elements(), 19);
}

#[test]
fn production_message_preflight_accepts_exact_and_rejects_one_over_synthetically() {
    // Project resource policy only: KMIPKit FR-007; no 16 MiB fixture allocation.
    let limits = CodecLimits::defaults();
    assert_eq!(
        super::decoder::test_preflight_message_length(
            CodecLimits::DEFAULT_MAX_MESSAGE_BYTES,
            &limits,
        ),
        Ok(())
    );
    assert_eq!(
        error_kind(&super::decoder::test_preflight_message_length(
            CodecLimits::DEFAULT_MAX_MESSAGE_BYTES + 1,
            &limits,
        )),
        Some(DecodeErrorKind::MessageTooLarge)
    );
}

#[test]
fn production_depth_preflight_accepts_exact_and_rejects_one_over_synthetically() {
    // Project resource policy only: KMIPKit FR-007; no depth-65 tree is built.
    let limits = CodecLimits::defaults();
    assert_eq!(
        super::decoder::test_structure_depth(CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH - 1, &limits,),
        Ok(CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH)
    );
    assert_eq!(
        error_kind(&super::decoder::test_structure_depth(
            CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
            &limits,
        )),
        Some(DecodeErrorKind::StructureDepthExceeded)
    );
}

#[test]
fn production_item_preflight_accepts_exact_and_rejects_one_over_synthetically() {
    // Project resource policy only: KMIPKit FR-007; no 100,001-item tree is built.
    let limits = CodecLimits::defaults();
    assert_eq!(
        super::decoder::test_next_element_count(CodecLimits::DEFAULT_MAX_ELEMENTS - 1, &limits),
        Ok(CodecLimits::DEFAULT_MAX_ELEMENTS)
    );
    assert_eq!(
        error_kind(&super::decoder::test_next_element_count(
            CodecLimits::DEFAULT_MAX_ELEMENTS,
            &limits,
        )),
        Some(DecodeErrorKind::ElementLimitExceeded)
    );
}

#[test]
fn configured_byte_limit_accepts_exact_and_rejects_one_over() {
    // Project resource policy only: KMIPKit FR-007. Fixtures are 8 and 16 bytes.
    let limits = limits(
        8,
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
        CodecLimits::DEFAULT_MAX_ELEMENTS,
    );
    assert!(decode_with_limits(&empty_structure_item(), &limits).is_ok());

    let one_over = integer_item();
    assert_eq!(one_over.len(), 16);
    assert_eq!(
        error_kind(&decode_with_limits(&one_over, &limits)),
        Some(DecodeErrorKind::MessageTooLarge)
    );
}

#[test]
fn configured_structure_depth_accepts_exact_and_rejects_one_over() {
    // Project resource policy only: KMIPKit FR-007. The nested fixture has depth two.
    let limits = limits(
        CodecLimits::DEFAULT_MAX_MESSAGE_BYTES,
        1,
        CodecLimits::DEFAULT_MAX_ELEMENTS,
    );
    assert!(decode_with_limits(&empty_structure_item(), &limits).is_ok());

    let child = empty_structure_item();
    let depth_two = structure_item(&child);
    assert_eq!(
        error_kind(&decode_with_limits(&depth_two, &limits)),
        Some(DecodeErrorKind::StructureDepthExceeded)
    );
}

#[test]
fn configured_item_count_accepts_exact_and_rejects_one_over() {
    // Project resource policy only: KMIPKit FR-007. The one-over tree has two Items.
    let limits = limits(
        CodecLimits::DEFAULT_MAX_MESSAGE_BYTES,
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
        1,
    );
    assert!(decode_with_limits(&empty_structure_item(), &limits).is_ok());

    let child = integer_item();
    let two_items = structure_item(&child);
    assert_eq!(
        error_kind(&decode_with_limits(&two_items, &limits)),
        Some(DecodeErrorKind::ElementLimitExceeded)
    );
}

#[test]
fn synthetic_u32_item_length_header_is_big_endian_and_truncation_is_bounded() {
    // OASIS KMIP Specification v2.1 §10.1.3, KMIPKIT-0005-NR-004; KMIPKit
    // FR-008 requires checking available input before declared-length allocation.
    let header = wire_header(0x08, u32::MAX);
    assert_eq!(header.len(), 8);
    assert_eq!(
        u32::from_be_bytes(header[4..8].try_into().expect("four length bytes")),
        u32::MAX
    );
    assert_eq!(
        error_kind(&decode_with_limits(&header, &CodecLimits::defaults())),
        Some(DecodeErrorKind::TruncatedValue)
    );
}

#[test]
fn production_checked_end_rejects_arithmetic_overflow() {
    // Project parser-safety requirement only: KMIPKit FR-008; no input buffer is made.
    assert_eq!(
        error_kind(&super::decoder::test_checked_end(usize::MAX - 3, 8)),
        Some(DecodeErrorKind::TruncatedValue)
    );
}

#[test]
fn truncated_u32_length_is_rejected_before_size_driven_allocation_or_copy() {
    // KMIPKit FR-008 requires length and available-input preflight before allocation.
    let header = wire_header(0x08, u32::MAX);
    let limits = CodecLimits::defaults();
    let observer = DecodeObserver::default();
    let result = super::decoder::decode_with_observer(&header, &limits, &observer);

    assert_eq!(error_kind(&result), Some(DecodeErrorKind::TruncatedValue));
    assert_eq!(observer.reservation_attempts(), 0);
    assert_eq!(observer.requested_reservation_bytes(), 0);
    assert_eq!(observer.peer_copy_calls(), 0);
}

#[test]
fn configured_limit_rejects_before_size_driven_allocation_or_copy() {
    // KMIPKit FR-007 applies the message limit; FR-008 requires rejection before
    // size-driven allocation. The 16-byte fixture stays below the default cap.
    let bytes = byte_string_item(&[0x5a]);
    let limits = limits(
        8,
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
        CodecLimits::DEFAULT_MAX_ELEMENTS,
    );
    let observer = DecodeObserver::default();
    let result = super::decoder::decode_with_observer(&bytes, &limits, &observer);

    assert_eq!(bytes.len(), 16);
    assert_eq!(error_kind(&result), Some(DecodeErrorKind::MessageTooLarge));
    assert_eq!(observer.reservation_attempts(), 0);
    assert_eq!(observer.requested_reservation_bytes(), 0);
    assert_eq!(observer.peer_copy_calls(), 0);
}
