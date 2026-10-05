use kmipkit_ttlv::ValueView;
use kmipkit_ttlv::codec::{DecodeError, DecodeErrorKind, decode};

const ASSIGNED_TAG: u32 = 0x0042_0173;

fn item(tag: u32, item_type: u8, length: u32, body: &[u8]) -> Vec<u8> {
    let mut wire = Vec::with_capacity(8 + body.len());
    wire.extend_from_slice(&tag.to_be_bytes()[1..]);
    wire.push(item_type);
    wire.extend_from_slice(&length.to_be_bytes());
    wire.extend_from_slice(body);
    wire
}

fn assert_kind(wire: &[u8], expected: DecodeErrorKind) {
    let error = decode(wire).expect_err("malformed public fixture is rejected");
    assert_eq!(error.kind(), expected);
}

// OASIS KMIP Specification v2.1, §§10.1.1–10.1.3; traceability
// KMIPKIT-0005-NR-001 and KMIPKIT-0005-NR-004.
#[test]
fn public_decoder_rejects_incomplete_headers_and_values() {
    let complete_header = item(ASSIGNED_TAG, 0x01, 0, &[]);
    for length in 0..8 {
        assert_kind(&complete_header[..length], DecodeErrorKind::TruncatedHeader);
    }

    let truncated = item(ASSIGNED_TAG, 0x03, 8, &[0, 1, 2, 3, 4, 5, 6]);
    assert_kind(&truncated, DecodeErrorKind::TruncatedValue);
}

// KMIPKit requirement KMIPKIT-0005-FR-004 requires exactly one complete item.
#[test]
fn public_decoder_rejects_trailing_bytes() {
    let mut wire = item(ASSIGNED_TAG, 0x01, 0, &[]);
    wire.push(0xFE);

    assert_kind(&wire, DecodeErrorKind::TrailingBytes);
}

// OASIS KMIP Specification v2.1, §§10.1.2–10.1.3 and §11.23; traceability
// KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-004.
#[test]
fn public_decoder_rejects_invalid_fixed_width_lengths_and_boolean_values() {
    assert_kind(
        &item(ASSIGNED_TAG, 0x02, 3, &[0, 0, 1, 0, 0, 0, 0]),
        DecodeErrorKind::InvalidItemLength,
    );
    assert_kind(
        &item(ASSIGNED_TAG, 0x06, 8, &[0, 0, 0, 0, 0, 0, 0, 2]),
        DecodeErrorKind::InvalidBoolean,
    );
}

// OASIS KMIP Specification v2.1, §10.1.2; traceability KMIPKIT-0005-NR-003.
// Rejecting the empty value is the explicit KMIPKit rule in FR-006.
#[test]
fn public_decoder_rejects_empty_and_misaligned_big_integer_values() {
    assert_kind(
        &item(ASSIGNED_TAG, 0x04, 0, &[]),
        DecodeErrorKind::EmptyBigInteger,
    );
    assert_kind(
        &item(ASSIGNED_TAG, 0x04, 7, &[1, 2, 3, 4, 5, 6, 7]),
        DecodeErrorKind::InvalidItemLength,
    );
}

// OASIS KMIP Specification v2.1, §10.1.2; Text String UTF-8 requirement,
// traceability KMIPKIT-0005-NR-002.
#[test]
fn public_decoder_rejects_invalid_text_string_utf8() {
    assert_kind(
        &item(ASSIGNED_TAG, 0x07, 1, &[0xFF, 0, 0, 0, 0, 0, 0, 0]),
        DecodeErrorKind::InvalidUtf8,
    );
}

// OASIS KMIP Specification v2.1 §11.23 lists assigned Item Types
// (KMIPKIT-0005-NR-002); rejection of an unsupported value is KMIPKit project
// requirement KMIPKIT-0005-FR-005, not an OASIS decoder requirement.
#[test]
fn public_decoder_rejects_item_types_outside_the_generic_model() {
    assert_kind(
        &item(ASSIGNED_TAG, 0x7F, 0, &[]),
        DecodeErrorKind::UnsupportedItemType,
    );
}

// KMIPKit requirement KMIPKIT-0005-FR-006 and OASIS KMIP Specification v2.1,
// §10.1.5; traceability KMIPKIT-0005-NR-005.
#[test]
fn public_decoder_checks_padding_extents_and_ignores_padding_octet_values() {
    assert_kind(
        &item(ASSIGNED_TAG, 0x02, 4, &[0, 0, 0, 1, 0xA5, 0xA5, 0xA5]),
        DecodeErrorKind::InvalidPaddingExtent,
    );
    let accepted = item(ASSIGNED_TAG, 0x02, 4, &[0, 0, 0, 1, 0xA5, 0xA5, 0xA5, 0xA5]);
    assert!(decode(&accepted).is_ok());
}

// KMIPKit requirement KMIPKIT-0005-FR-006 and OASIS KMIP Specification v2.1,
// §§10.1.2, 10.1.3, and 10.1.5; traceability KMIPKIT-0005-NR-002,
// KMIPKIT-0005-NR-004, and KMIPKIT-0005-NR-005.
#[test]
fn public_decoder_rejects_children_that_cross_structure_boundaries() {
    let child = item(ASSIGNED_TAG, 0x02, 4, &[0, 0, 0, 1, 0, 0, 0, 0]);
    let wire = item(ASSIGNED_TAG, 0x01, 8, &child);

    assert_kind(&wire, DecodeErrorKind::StructureBoundary);

    let child_header_only = item(ASSIGNED_TAG, 0x02, 4, &[]);
    let wire = item(ASSIGNED_TAG, 0x01, 8, &child_header_only);
    assert_kind(&wire, DecodeErrorKind::StructureBoundary);
}

// KMIPKit requirement KMIPKIT-0005-FR-010 and accepted ADR-0011 require
// Reserved-tag rejection. OASIS KMIP Specification v2.1, Chapter 11 and
// §11.56 classify the tag; traceability KMIPKIT-0005-NR-006.
#[test]
fn public_decoder_rejects_received_reserved_tags() {
    let wire = item(0x0042_0009, 0x05, 4, &[0, 0, 0, 9, 0, 0, 0, 0]);

    assert_kind(&wire, DecodeErrorKind::ReservedTag);
}

// KMIPKit requirement KMIPKIT-0005-FR-010 also rejects Tags outside the
// assigned and extension allocations. OASIS KMIP Specification v2.1, Chapter
// 11 and §11.56 provide the allocation context; traceability KMIPKIT-0005-NR-006.
#[test]
fn public_decoder_rejects_unallocated_tags() {
    let wire = item(0x0000_0001, 0x05, 4, &[0, 0, 0, 9, 0, 0, 0, 0]);

    assert_kind(&wire, DecodeErrorKind::UnallocatedTag);
}

// OASIS KMIP Specification v2.1, §§10.1.2–10.1.3, §11.23, Chapter 11, and
// §11.56; traceability KMIPKIT-0005-NR-002, NR-003, NR-004, and NR-006. The
// preservation expectation is KMIPKit project requirement KMIPKIT-0005-FR-005.
#[test]
fn public_decoder_preserves_extension_tags_and_exact_big_integer_octets() {
    let value = [0xFF, 0, 1, 2, 3, 4, 5, 6];
    let extension = item(0x0054_1234, 0x04, 8, &value);
    let decoded = decode(&extension).expect("extension Tags and aligned Big Integers are accepted");

    assert_eq!(decoded.tag().raw(), 0x0054_1234);
    assert!(
        decoded.with_value(
            |decoded| matches!(decoded, ValueView::BigInteger(actual) if actual == value)
        )
    );
}

// KMIPKit requirement KMIPKIT-0005-FR-009 requires payload-free diagnostics.
#[test]
fn public_decode_errors_do_not_retain_or_format_input_payloads() {
    let secret_marker = b"payload-secret-marker";
    let mut wire = item(ASSIGNED_TAG, 0x07, 1, &[0xFF, 0, 0, 0, 0, 0, 0, 0]);
    wire.extend_from_slice(secret_marker);
    let error: DecodeError = decode(&wire).expect_err("invalid UTF-8 is rejected");
    let diagnostics = format!("{error:?} {error}");

    assert!(!diagnostics.contains("payload-secret-marker"));
    assert_eq!(error.kind(), DecodeErrorKind::InvalidUtf8);
    assert_eq!(error.offset(), 8);
}

// KMIPKit requirements KMIPKIT-0005-FR-007 and FR-008 bound input size before
// parsing or allocation; these are local resource policies, not OASIS clauses.
#[test]
fn public_decoder_rejects_input_over_the_default_message_limit_before_parsing() {
    let oversized = vec![0; 16 * 1024 * 1024 + 1];

    assert_kind(&oversized, DecodeErrorKind::MessageTooLarge);
}

// KMIPKit requirement KMIPKIT-0005-FR-007 sets a 16 MiB default message cap.
#[test]
fn public_decoder_accepts_an_item_at_the_default_message_limit() {
    let maximum = 16 * 1024 * 1024;
    let payload = vec![0xA5; maximum - 8];
    let wire = item(
        ASSIGNED_TAG,
        0x08,
        u32::try_from(payload.len()).expect("fixture length fits the wire field"),
        &payload,
    );

    assert_eq!(wire.len(), maximum);
    assert!(decode(&wire).is_ok());
}

// KMIPKit requirements KMIPKIT-0005-FR-007 and FR-008 bound nested work.
#[test]
fn public_decoder_enforces_the_default_structure_depth_limit() {
    let mut nested = item(ASSIGNED_TAG, 0x08, 0, &[]);
    for _ in 0..64 {
        let child_length =
            u32::try_from(nested.len()).expect("small nested fixture length fits the wire field");
        nested = item(ASSIGNED_TAG, 0x01, child_length, &nested);
    }
    assert!(decode(&nested).is_ok());

    let child_length =
        u32::try_from(nested.len()).expect("small nested fixture length fits the wire field");
    let too_deep = item(ASSIGNED_TAG, 0x01, child_length, &nested);
    assert_kind(&too_deep, DecodeErrorKind::StructureDepthExceeded);
}

// KMIPKit requirements KMIPKIT-0005-FR-007 and FR-008 count the root and every
// descendant before accepting the complete tree.
#[test]
fn public_decoder_enforces_the_default_item_count_limit() {
    let empty_child = item(ASSIGNED_TAG, 0x01, 0, &[]);
    let mut exactly_at_limit = Vec::with_capacity(8 + 8 * 99_999);
    for _ in 0..99_999 {
        exactly_at_limit.extend_from_slice(&empty_child);
    }
    let root = item(
        ASSIGNED_TAG,
        0x01,
        u32::try_from(exactly_at_limit.len()).expect("fixture fits the wire field"),
        &exactly_at_limit,
    );
    assert!(decode(&root).is_ok());

    let mut one_over = exactly_at_limit;
    one_over.extend_from_slice(&empty_child);
    let root = item(
        ASSIGNED_TAG,
        0x01,
        u32::try_from(one_over.len()).expect("fixture fits the wire field"),
        &one_over,
    );
    assert_kind(&root, DecodeErrorKind::ElementLimitExceeded);
}
