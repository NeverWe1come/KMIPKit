//! Red coverage for per-call decoder resource limits before T010.

use std::cell::Cell;

use super::{DecodeError, DecodeErrorKind, decode};
use crate::Item;

const DEFAULT_MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_MAX_STRUCTURE_DEPTH: usize = 64;
const DEFAULT_MAX_ELEMENTS: usize = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RedLimits {
    message_bytes: usize,
    structure_depth: usize,
    elements: usize,
}

impl RedLimits {
    const fn defaults() -> Self {
        Self::new(
            DEFAULT_MAX_MESSAGE_BYTES,
            DEFAULT_MAX_STRUCTURE_DEPTH,
            DEFAULT_MAX_ELEMENTS,
        )
    }

    const fn new(message_bytes: usize, structure_depth: usize, elements: usize) -> Self {
        Self {
            message_bytes,
            structure_depth,
            elements,
        }
    }

    const fn max_message_bytes(&self) -> usize {
        self.message_bytes
    }

    const fn max_structure_depth(&self) -> usize {
        self.structure_depth
    }

    const fn max_elements(&self) -> usize {
        self.elements
    }
}

// Red stub: it intentionally ignores RedLimits until T010 promotes this seam.
fn decode_candidate(bytes: &[u8], _limits: &RedLimits) -> Result<Item, DecodeError> {
    decode(bytes)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreflightOutcome {
    Accepted,
    Rejected(DecodeErrorKind),
}

// Red stub: no default or explicit limit is checked yet.
fn preflight_candidate(
    _message_bytes: usize,
    _structure_depth: usize,
    _elements: usize,
    _limits: &RedLimits,
) -> PreflightOutcome {
    PreflightOutcome::Accepted
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CheckedEndOutcome {
    End(usize),
    Overflow,
}

// Red stub: wrapping arithmetic incorrectly reports overflow as a valid end.
fn checked_end_candidate(start: usize, extent: usize) -> CheckedEndOutcome {
    CheckedEndOutcome::End(start.wrapping_add(extent))
}

#[derive(Default)]
struct DecodeObserver {
    reservation_attempts: Cell<usize>,
    requested_reservation_bytes: Cell<u64>,
    peer_copy_calls: Cell<usize>,
}

impl DecodeObserver {
    fn record_size_driven_activity(&self, declared_length: u32) {
        self.reservation_attempts
            .set(self.reservation_attempts.get().saturating_add(1));
        self.requested_reservation_bytes.set(
            self.requested_reservation_bytes
                .get()
                .saturating_add(u64::from(declared_length)),
        );
        self.peer_copy_calls
            .set(self.peer_copy_calls.get().saturating_add(1));
    }
}

// Red stub: it records a would-be allocation/copy before checking available
// input or RedLimits. It updates counters only and never allocates test memory.
fn decode_candidate_with_observer(
    bytes: &[u8],
    limits: &RedLimits,
    observer: &DecodeObserver,
) -> Result<Item, DecodeError> {
    if bytes.len() >= 8
        && matches!(bytes[3], 0x04 | 0x07 | 0x08)
        && let Some(declared_length) = read_header_length(bytes)
    {
        observer.record_size_driven_activity(declared_length);
    }
    decode_candidate(bytes, limits)
}

fn read_header_length(bytes: &[u8]) -> Option<u32> {
    let length_bytes: [u8; 4] = bytes.get(4..8)?.try_into().ok()?;
    Some(u32::from_be_bytes(length_bytes))
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

fn error_kind(result: &Result<Item, DecodeError>) -> Option<DecodeErrorKind> {
    result.as_ref().err().map(DecodeError::kind)
}

#[test]
fn red_limits_getters_match_default_and_explicit_values() {
    // KMIPKit resource defaults and read-only getter behavior: FR-007.
    let defaults = RedLimits::defaults();
    assert_eq!(defaults.max_message_bytes(), DEFAULT_MAX_MESSAGE_BYTES);
    assert_eq!(defaults.max_structure_depth(), DEFAULT_MAX_STRUCTURE_DEPTH);
    assert_eq!(defaults.max_elements(), DEFAULT_MAX_ELEMENTS);

    let explicit = RedLimits::new(32, 7, 19);
    assert_eq!(explicit.max_message_bytes(), 32);
    assert_eq!(explicit.max_structure_depth(), 7);
    assert_eq!(explicit.max_elements(), 19);
}

#[test]
fn default_byte_limit_preflight_accepts_exact_and_rejects_one_over_synthetically() {
    // Project resource policy only: KMIPKit FR-007, without allocating 16 MiB.
    let limits = RedLimits::defaults();
    assert_eq!(
        preflight_candidate(DEFAULT_MAX_MESSAGE_BYTES, 0, 1, &limits),
        PreflightOutcome::Accepted
    );
    assert_eq!(
        preflight_candidate(DEFAULT_MAX_MESSAGE_BYTES + 1, 0, 1, &limits),
        PreflightOutcome::Rejected(DecodeErrorKind::MessageTooLarge)
    );
}

#[test]
fn default_depth_limit_preflight_accepts_exact_and_rejects_one_over_synthetically() {
    // Project resource policy only: KMIPKit FR-007; no depth-65 tree is built.
    let limits = RedLimits::defaults();
    assert_eq!(
        preflight_candidate(8, DEFAULT_MAX_STRUCTURE_DEPTH, 1, &limits),
        PreflightOutcome::Accepted
    );
    assert_eq!(
        preflight_candidate(8, DEFAULT_MAX_STRUCTURE_DEPTH + 1, 1, &limits),
        PreflightOutcome::Rejected(DecodeErrorKind::StructureDepthExceeded)
    );
}

#[test]
fn default_item_count_preflight_accepts_exact_and_rejects_one_over_synthetically() {
    // Project resource policy only: KMIPKit FR-007; no 100,001-item tree is built.
    let limits = RedLimits::defaults();
    assert_eq!(
        preflight_candidate(8, 0, DEFAULT_MAX_ELEMENTS, &limits),
        PreflightOutcome::Accepted
    );
    assert_eq!(
        preflight_candidate(8, 0, DEFAULT_MAX_ELEMENTS + 1, &limits),
        PreflightOutcome::Rejected(DecodeErrorKind::ElementLimitExceeded)
    );
}

#[test]
fn configured_byte_limit_accepts_exact_and_rejects_one_over() {
    // Project resource policy only: KMIPKit FR-007. Fixtures are 8 and 16 bytes.
    let limits = RedLimits::new(8, DEFAULT_MAX_STRUCTURE_DEPTH, DEFAULT_MAX_ELEMENTS);
    assert!(decode_candidate(&empty_structure_item(), &limits).is_ok());

    let one_over = integer_item();
    assert_eq!(one_over.len(), 16);
    assert_eq!(
        error_kind(&decode_candidate(&one_over, &limits)),
        Some(DecodeErrorKind::MessageTooLarge)
    );
}

#[test]
fn configured_structure_depth_accepts_exact_and_rejects_one_over() {
    // Project resource policy only: KMIPKit FR-007. The nested fixture has depth two.
    let limits = RedLimits::new(DEFAULT_MAX_MESSAGE_BYTES, 1, DEFAULT_MAX_ELEMENTS);
    assert!(decode_candidate(&empty_structure_item(), &limits).is_ok());

    let child = empty_structure_item();
    let depth_two = structure_item(&child);
    assert_eq!(
        error_kind(&decode_candidate(&depth_two, &limits)),
        Some(DecodeErrorKind::StructureDepthExceeded)
    );
}

#[test]
fn configured_item_count_accepts_exact_and_rejects_one_over() {
    // Project resource policy only: KMIPKit FR-007. The one-over tree has two Items.
    let limits = RedLimits::new(DEFAULT_MAX_MESSAGE_BYTES, DEFAULT_MAX_STRUCTURE_DEPTH, 1);
    assert!(decode_candidate(&empty_structure_item(), &limits).is_ok());

    let child = integer_item();
    let two_items = structure_item(&child);
    assert_eq!(
        error_kind(&decode_candidate(&two_items, &limits)),
        Some(DecodeErrorKind::ElementLimitExceeded)
    );
}

#[test]
fn synthetic_u32_item_length_header_is_big_endian_and_truncation_is_bounded() {
    // OASIS KMIP Specification v2.1 §10.1.3, KMIPKIT-0005-NR-004; KMIPKit
    // FR-008 requires checking available input before any declared-length allocation.
    let header = wire_header(0x08, u32::MAX);
    assert_eq!(header.len(), 8);
    assert_eq!(read_header_length(&header), Some(u32::MAX));
    assert_eq!(
        error_kind(&decode_candidate(&header, &RedLimits::defaults())),
        Some(DecodeErrorKind::TruncatedValue)
    );
}

#[test]
fn synthetic_checked_end_rejects_arithmetic_overflow() {
    // Project parser-safety requirement only: KMIPKit FR-008; no input buffer is made.
    assert_eq!(
        checked_end_candidate(usize::MAX - 3, 8),
        CheckedEndOutcome::Overflow
    );
}

#[test]
fn truncated_u32_length_is_rejected_before_size_driven_allocation_or_copy() {
    // KMIPKit FR-008 requires length and available-input preflight before allocation.
    let header = wire_header(0x08, u32::MAX);
    let observer = DecodeObserver::default();
    let result = decode_candidate_with_observer(&header, &RedLimits::defaults(), &observer);

    assert_eq!(error_kind(&result), Some(DecodeErrorKind::TruncatedValue));
    assert_eq!(observer.reservation_attempts.get(), 0);
    assert_eq!(observer.requested_reservation_bytes.get(), 0);
    assert_eq!(observer.peer_copy_calls.get(), 0);
}

#[test]
fn configured_limit_is_rejected_before_size_driven_allocation_or_copy() {
    // KMIPKit FR-007 applies the message limit; FR-008 requires rejection before
    // size-driven allocation. The 16-byte Byte String stays well below the default cap.
    let bytes = byte_string_item(&[0x5a]);
    let limits = RedLimits::new(8, DEFAULT_MAX_STRUCTURE_DEPTH, DEFAULT_MAX_ELEMENTS);
    let observer = DecodeObserver::default();
    let result = decode_candidate_with_observer(&bytes, &limits, &observer);

    assert_eq!(bytes.len(), 16);
    assert_eq!(error_kind(&result), Some(DecodeErrorKind::MessageTooLarge));
    assert_eq!(observer.reservation_attempts.get(), 0);
    assert_eq!(observer.requested_reservation_bytes.get(), 0);
    assert_eq!(observer.peer_copy_calls.get(), 0);
}
