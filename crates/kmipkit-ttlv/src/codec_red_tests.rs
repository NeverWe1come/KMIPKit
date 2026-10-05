//! Red harness for the not-yet-implemented TTLV decoder.
//!
//! Every decoder expectation goes through `decode_candidate`. The deliberately
//! incomplete seam keeps this commit compile-clean while each missing behavior
//! fails as a test assertion rather than as an unresolved production symbol.

use crate::{Item, ItemType, ValueView};

const ASSIGNED_TAG: u32 = 0x0042_0173;
const INTEGER: u8 = 0x02;
const LONG_INTEGER: u8 = 0x03;
const BIG_INTEGER: u8 = 0x04;
const ENUMERATION: u8 = 0x05;
const BOOLEAN: u8 = 0x06;
const TEXT_STRING: u8 = 0x07;
const BYTE_STRING: u8 = 0x08;
const DATE_TIME: u8 = 0x09;
const INTERVAL: u8 = 0x0A;
const DATE_TIME_EXTENDED: u8 = 0x0B;
const STRUCTURE: u8 = 0x01;

// Expected variants are referenced by assertions now and constructed only
// when T006 replaces the incomplete candidate stub.
#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq)]
enum CandidateError {
    Incomplete,
    TruncatedHeader,
    TruncatedValue,
    InvalidItemLength,
    EmptyBigInteger,
    InvalidUtf8,
    InvalidBoolean,
    UnsupportedItemType,
    TrailingBytes,
    StructureBoundary,
    ReservedTag,
    InvalidPaddingExtent,
}

fn decode_candidate(_input: &[u8]) -> Result<Item, CandidateError> {
    Err(CandidateError::Incomplete)
}

fn item_bytes(tag: u32, item_type: u8, item_length: u32, body: &[u8]) -> Vec<u8> {
    assert!(tag <= 0x00FF_FFFF, "fixture tag fits the 24-bit wire field");
    let mut bytes = Vec::with_capacity(8 + body.len());
    bytes.extend_from_slice(&tag.to_be_bytes()[1..]);
    bytes.push(item_type);
    bytes.extend_from_slice(&item_length.to_be_bytes());
    bytes.extend_from_slice(body);
    bytes
}

fn has_integer(item: &Item, expected: i32) -> bool {
    item.tag().raw() == ASSIGNED_TAG
        && item.item_type() == ItemType::Integer
        && item
            .with_value(|value| matches!(value, ValueView::Integer(actual) if *actual == expected))
}

fn has_long_integer(item: &Item, expected: i64) -> bool {
    item.tag().raw() == ASSIGNED_TAG
        && item.item_type() == ItemType::LongInteger
        && item.with_value(
            |value| matches!(value, ValueView::LongInteger(actual) if *actual == expected),
        )
}

fn has_big_integer(item: &Item, expected: &[u8]) -> bool {
    item.tag().raw() == ASSIGNED_TAG
        && item.item_type() == ItemType::BigInteger
        && item.with_value(
            |value| matches!(value, ValueView::BigInteger(actual) if actual == expected),
        )
}

fn has_enumeration(item: &Item, expected: u32) -> bool {
    item.tag().raw() == ASSIGNED_TAG
        && item.item_type() == ItemType::Enumeration
        && item.with_value(
            |value| matches!(value, ValueView::Enumeration(actual) if *actual == expected),
        )
}

fn has_boolean(item: &Item, expected: bool) -> bool {
    item.tag().raw() == ASSIGNED_TAG
        && item.item_type() == ItemType::Boolean
        && item
            .with_value(|value| matches!(value, ValueView::Boolean(actual) if *actual == expected))
}

fn has_text_string(item: &Item, expected: &str) -> bool {
    item.tag().raw() == ASSIGNED_TAG
        && item.item_type() == ItemType::TextString
        && item.with_value(
            |value| matches!(value, ValueView::TextString(actual) if actual == expected),
        )
}

fn has_byte_string(item: &Item, expected: &[u8]) -> bool {
    item.tag().raw() == ASSIGNED_TAG
        && item.item_type() == ItemType::ByteString
        && item.with_value(
            |value| matches!(value, ValueView::ByteString(actual) if actual == expected),
        )
}

fn has_date_time(item: &Item, expected: i64) -> bool {
    item.tag().raw() == ASSIGNED_TAG
        && item.item_type() == ItemType::DateTime
        && item
            .with_value(|value| matches!(value, ValueView::DateTime(actual) if *actual == expected))
}

fn has_interval(item: &Item, expected: u32) -> bool {
    item.tag().raw() == ASSIGNED_TAG
        && item.item_type() == ItemType::Interval
        && item
            .with_value(|value| matches!(value, ValueView::Interval(actual) if *actual == expected))
}

fn has_date_time_extended(item: &Item, expected: i64) -> bool {
    item.tag().raw() == ASSIGNED_TAG
        && item.item_type() == ItemType::DateTimeExtended
        && item.with_value(
            |value| matches!(value, ValueView::DateTimeExtended(actual) if *actual == expected),
        )
}

// OASIS KMIP Specification v2.1, §§10.1.2–10.1.3 and §11.23; traceability
// KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-004.
#[test]
fn decodes_a_valid_empty_structure_item() {
    let wire = item_bytes(ASSIGNED_TAG, STRUCTURE, 0, &[]);

    assert!(matches!(decode_candidate(&wire), Ok(item)
        if item.tag().raw() == ASSIGNED_TAG
            && item.item_type() == ItemType::Structure
            && item.with_value(|value| matches!(value, ValueView::Structure(structure) if structure.children().is_empty()))));
}

// OASIS KMIP Specification v2.1, §§10.1.2, 10.1.5, and §11.23; traceability
// KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-005.
#[test]
fn decodes_a_valid_integer_item_and_its_four_byte_padding() {
    let wire = item_bytes(
        ASSIGNED_TAG,
        INTEGER,
        4,
        &[0x80, 0x00, 0x00, 0x00, 0xA5, 0xA5, 0xA5, 0xA5],
    );

    assert!(matches!(decode_candidate(&wire), Ok(item) if has_integer(&item, i32::MIN)));
}

// OASIS KMIP Specification v2.1, §§10.1.2–10.1.3 and §11.23; traceability
// KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-004.
#[test]
fn decodes_a_valid_long_integer_item() {
    let wire = item_bytes(
        ASSIGNED_TAG,
        LONG_INTEGER,
        8,
        &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
    );

    assert!(matches!(decode_candidate(&wire), Ok(item)
        if has_long_integer(&item, 0x0102_0304_0506_0708)));
}

// OASIS KMIP Specification v2.1, §§10.1.2–10.1.3 and §11.23; traceability
// KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-003, and KMIPKIT-0005-NR-004.
#[test]
fn decodes_a_valid_aligned_big_integer_item_without_changing_its_octets() {
    let value = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A];
    let wire = item_bytes(ASSIGNED_TAG, BIG_INTEGER, 8, &value);

    assert!(matches!(decode_candidate(&wire), Ok(item) if has_big_integer(&item, &value)));
}

// OASIS KMIP Specification v2.1, §§10.1.2, 10.1.5, and §11.23; traceability
// KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-005.
#[test]
fn decodes_a_valid_enumeration_item_and_its_four_byte_padding() {
    let wire = item_bytes(
        ASSIGNED_TAG,
        ENUMERATION,
        4,
        &[0xFF, 0xFF, 0xFF, 0xFF, 0x5A, 0x5A, 0x5A, 0x5A],
    );

    assert!(matches!(decode_candidate(&wire), Ok(item)
        if has_enumeration(&item, u32::MAX)));
}

// OASIS KMIP Specification v2.1, §§10.1.2 and §11.23; traceability
// KMIPKIT-0005-NR-002.
#[test]
fn decodes_a_valid_true_boolean_item() {
    let wire = item_bytes(ASSIGNED_TAG, BOOLEAN, 8, &[0, 0, 0, 0, 0, 0, 0, 1]);

    assert!(matches!(decode_candidate(&wire), Ok(item) if has_boolean(&item, true)));
}

// OASIS KMIP Specification v2.1, §§10.1.2, 10.1.5, and §11.23; traceability
// KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-005.
#[test]
fn decodes_a_valid_utf8_text_string_with_minimal_padding() {
    let wire = item_bytes(
        ASSIGNED_TAG,
        TEXT_STRING,
        2,
        &[0xC3, 0xA9, 0xA5, 0xA5, 0xA5, 0xA5, 0xA5, 0xA5],
    );

    assert!(matches!(decode_candidate(&wire), Ok(item) if has_text_string(&item, "é")));
}

// OASIS KMIP Specification v2.1, §§10.1.2, 10.1.5, and §11.23; traceability
// KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-005.
#[test]
fn decodes_a_valid_byte_string_with_minimal_padding() {
    let value = [0x00, 0x80, 0xFF];
    let wire = item_bytes(
        ASSIGNED_TAG,
        BYTE_STRING,
        3,
        &[0x00, 0x80, 0xFF, 0xA5, 0xA5, 0xA5, 0xA5, 0xA5],
    );

    assert!(matches!(decode_candidate(&wire), Ok(item) if has_byte_string(&item, &value)));
}

// OASIS KMIP Specification v2.1, §§10.1.2–10.1.3 and §11.23; traceability
// KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-004.
#[test]
fn decodes_a_valid_date_time_item() {
    let wire = item_bytes(ASSIGNED_TAG, DATE_TIME, 8, &[0xFF; 8]);

    assert!(matches!(decode_candidate(&wire), Ok(item) if has_date_time(&item, -1)));
}

// OASIS KMIP Specification v2.1, §§10.1.2, 10.1.5, and §11.23; traceability
// KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-005.
#[test]
fn decodes_a_valid_interval_item_and_its_four_byte_padding() {
    let wire = item_bytes(
        ASSIGNED_TAG,
        INTERVAL,
        4,
        &[0xFF, 0xFF, 0xFF, 0xFF, 0xA5, 0xA5, 0xA5, 0xA5],
    );

    assert!(matches!(decode_candidate(&wire), Ok(item) if has_interval(&item, u32::MAX)));
}

// OASIS KMIP Specification v2.1, §§10.1.2–10.1.3 and §11.23; traceability
// KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-004.
#[test]
fn decodes_a_valid_date_time_extended_item() {
    let wire = item_bytes(
        ASSIGNED_TAG,
        DATE_TIME_EXTENDED,
        8,
        &[0x80, 0, 0, 0, 0, 0, 0, 0],
    );

    assert!(matches!(decode_candidate(&wire), Ok(item)
        if has_date_time_extended(&item, i64::MIN)));
}

// OASIS KMIP Specification v2.1, §§10.1.2 and 11.23 define the wire forms;
// preserving every unknown Enumeration bit is KMIPKit project requirement
// KMIPKIT-0005-FR-005, not an OASIS operation-level enumeration requirement.
// Traceability for the OASIS value form: KMIPKIT-0005-NR-002.
#[test]
fn preserves_an_unknown_enumeration_value_as_raw_u32_bits() {
    let wire = item_bytes(
        ASSIGNED_TAG,
        ENUMERATION,
        4,
        &[0xDE, 0xAD, 0xBE, 0xEF, 0xA5, 0xA5, 0xA5, 0xA5],
    );

    assert!(matches!(decode_candidate(&wire), Ok(item)
        if has_enumeration(&item, 0xDEAD_BEEF)));
}

// OASIS KMIP Specification v2.1, §§10.1.2 and 11.23 define the wire forms;
// preserving unknown Integer mask bits is KMIPKit project requirement
// KMIPKIT-0005-FR-005, not an OASIS decoder semantic requirement.
// Traceability for the OASIS value form: KMIPKIT-0005-NR-002.
#[test]
fn preserves_unknown_integer_mask_bits() {
    let bits = [0xF1, 0x23, 0x45, 0x67];
    let wire = item_bytes(
        ASSIGNED_TAG,
        INTEGER,
        4,
        &[0xF1, 0x23, 0x45, 0x67, 0xA5, 0xA5, 0xA5, 0xA5],
    );

    assert!(matches!(decode_candidate(&wire), Ok(item)
        if has_integer(&item, i32::from_be_bytes(bits))));
}

// OASIS KMIP Specification v2.1, §§10.1.2 and 11.23; traceability
// KMIPKIT-0005-NR-002. Preservation of repeated Tags and caller order is the
// generic-model behavior in KMIPKit project requirement KMIPKIT-0005-FR-005.
#[test]
fn preserves_structure_child_order_and_repeated_tags() {
    let first = item_bytes(
        ASSIGNED_TAG,
        INTEGER,
        4,
        &[0, 0, 0, 1, 0xA5, 0xA5, 0xA5, 0xA5],
    );
    let second = item_bytes(
        ASSIGNED_TAG,
        INTEGER,
        4,
        &[0, 0, 0, 2, 0xA5, 0xA5, 0xA5, 0xA5],
    );
    let mut children = first;
    children.extend_from_slice(&second);
    let wire = item_bytes(ASSIGNED_TAG, STRUCTURE, 32, &children);

    assert!(matches!(decode_candidate(&wire), Ok(item)
    if item.with_value(|value| match value {
        ValueView::Structure(structure) => {
            let children = structure.children();
            children.len() == 2
                && has_integer(&children[0], 1)
                && has_integer(&children[1], 2)
        }
        _ => false,
    })));
}

// OASIS KMIP Specification v2.1, §§10.1.2 and 11.23; traceability
// KMIPKIT-0005-NR-002. Cumulative Structure boundaries follow
// KMIPKit project requirement KMIPKIT-0005-FR-006.
#[test]
fn preserves_nested_structure_boundaries_and_child_value() {
    let child = item_bytes(
        ASSIGNED_TAG,
        INTEGER,
        4,
        &[0, 0, 0, 42, 0xA5, 0xA5, 0xA5, 0xA5],
    );
    let nested = item_bytes(ASSIGNED_TAG, STRUCTURE, 16, &child);
    let wire = item_bytes(ASSIGNED_TAG, STRUCTURE, 24, &nested);

    assert!(matches!(decode_candidate(&wire), Ok(item)
    if item.with_value(|value| match value {
        ValueView::Structure(outer) if outer.children().len() == 1 => {
            let nested = &outer.children()[0];
            nested.item_type() == ItemType::Structure
                && nested.with_value(|nested_value| match nested_value {
                    ValueView::Structure(inner) => {
                        inner.children().len() == 1 && has_integer(&inner.children()[0], 42)
                    }
                    _ => false,
                })
        }
        _ => false,
    })));
}

// OASIS KMIP Specification v2.1, Chapter 11 introduction and §11.56; the
// assigned Tag remains accepted under project requirement KMIPKIT-0005-FR-005.
// Traceability: KMIPKIT-0005-NR-006 covers allocation classification.
#[test]
fn accepts_an_assigned_tag() {
    let tag = 0x0042_0173;
    let wire = item_bytes(tag, ENUMERATION, 4, &[0, 0, 0, 9, 0xA5, 0xA5, 0xA5, 0xA5]);

    assert!(matches!(decode_candidate(&wire), Ok(item)
        if item.tag().raw() == tag && has_enumeration(&item, 9)));
}

// OASIS KMIP Specification v2.1, Chapter 11 introduction and §11.56; an
// accepted extension Tag is preserved by KMIPKit project requirement
// KMIPKIT-0005-FR-005. Traceability: KMIPKIT-0005-NR-006 covers allocation.
#[test]
fn accepts_an_extension_tag() {
    let extension_tag = 0x0054_1234;
    let wire = item_bytes(
        extension_tag,
        ENUMERATION,
        4,
        &[0, 0, 0, 9, 0xA5, 0xA5, 0xA5, 0xA5],
    );

    assert!(matches!(decode_candidate(&wire), Ok(item)
        if item.tag().raw() == extension_tag && item.item_type() == ItemType::Enumeration));
}

// Project requirement KMIPKIT-0005-FR-010 and accepted ADR-0011 require
// Reserved-tag rejection; this is not an OASIS decoder rejection requirement.
// OASIS KMIP Specification v2.1, Chapter 11 introduction and §11.56 define the
// Tag classification covered by KMIPKIT-0005-NR-006.
#[test]
fn rejects_a_received_reserved_tag_before_item_construction() {
    let wire = item_bytes(0x0042_0009, ENUMERATION, 4, &[0, 0, 0, 9, 0, 0, 0, 0]);

    assert!(matches!(
        decode_candidate(&wire),
        Err(CandidateError::ReservedTag)
    ));
}

// KMIPKit project requirement KMIPKIT-0005-FR-004 requires one complete item
// and rejects trailing bytes; this is project framing behavior, not a separate
// OASIS requirement.
#[test]
fn rejects_bytes_after_one_complete_item() {
    let mut wire = item_bytes(ASSIGNED_TAG, STRUCTURE, 0, &[]);
    wire.push(0xFF);

    assert!(matches!(
        decode_candidate(&wire),
        Err(CandidateError::TrailingBytes)
    ));
}

// OASIS KMIP Specification v2.1, §§10.1.1–10.1.3; traceability
// KMIPKIT-0005-NR-001 and KMIPKIT-0005-NR-004 require the complete header and
// declared Item Value fields. Each prefix is a distinct truncation vector.
#[test]
fn rejects_every_incomplete_header_prefix() {
    let complete_header = item_bytes(ASSIGNED_TAG, STRUCTURE, 0, &[]);

    for length in 0..8 {
        assert!(
            matches!(
                decode_candidate(&complete_header[..length]),
                Err(CandidateError::TruncatedHeader)
            ),
            "the {length}-byte header prefix must be rejected"
        );
    }
}

// OASIS KMIP Specification v2.1, §§10.1.2–10.1.3; traceability
// KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-004 require the declared value extent.
#[test]
fn rejects_a_truncated_fixed_width_value() {
    let wire = item_bytes(ASSIGNED_TAG, LONG_INTEGER, 8, &[0, 1, 2, 3, 4, 5, 6]);

    assert!(matches!(
        decode_candidate(&wire),
        Err(CandidateError::TruncatedValue)
    ));
}

// OASIS KMIP Specification v2.1, §§10.1.2–10.1.3 and §11.23; traceability
// KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-004 define fixed-width values and
// allowed Item Lengths for all seven fixed-width Item Types.
#[test]
fn rejects_invalid_lengths_for_every_fixed_width_item_type() {
    let cases = [
        (INTEGER, 3, 4),
        (LONG_INTEGER, 7, 0),
        (ENUMERATION, 3, 4),
        (BOOLEAN, 7, 0),
        (DATE_TIME, 7, 0),
        (INTERVAL, 3, 4),
        (DATE_TIME_EXTENDED, 7, 0),
    ];

    for (item_type, item_length, padding_length) in cases {
        let mut body = vec![0; item_length as usize];
        body.resize(body.len() + padding_length, 0xA5);
        let wire = item_bytes(ASSIGNED_TAG, item_type, item_length, &body);

        assert!(
            matches!(
                decode_candidate(&wire),
                Err(CandidateError::InvalidItemLength)
            ),
            "Item Type 0x{item_type:02X} with length {item_length} must be rejected"
        );
    }
}

// OASIS KMIP Specification v2.1, §10.1.2; traceability KMIPKIT-0005-NR-003
// requires a Big Integer Item Value length that is a multiple of eight.
#[test]
fn rejects_a_nonempty_big_integer_with_a_non_multiple_of_eight_length() {
    let wire = item_bytes(ASSIGNED_TAG, BIG_INTEGER, 7, &[1, 2, 3, 4, 5, 6, 7]);

    assert!(matches!(
        decode_candidate(&wire),
        Err(CandidateError::InvalidItemLength)
    ));
}

// KMIPKit project requirement KMIPKIT-0005-FR-006 and the approved
// clarification record reject empty Big Integer values. OASIS KMIP
// Specification v2.1 §10.1.2 does not state a minimum length, so this is not
// presented as an OASIS MUST. Its Big Integer value form is KMIPKIT-0005-NR-003.
#[test]
fn rejects_an_empty_big_integer_under_the_project_validity_rule() {
    let wire = item_bytes(ASSIGNED_TAG, BIG_INTEGER, 0, &[]);

    assert!(matches!(
        decode_candidate(&wire),
        Err(CandidateError::EmptyBigInteger)
    ));
}

// OASIS KMIP Specification v2.1, §10.1.2 and §11.23 define Text String as
// UTF-8; traceability KMIPKIT-0005-NR-002.
#[test]
fn rejects_invalid_utf8_text_string_octets() {
    let wire = item_bytes(
        ASSIGNED_TAG,
        TEXT_STRING,
        1,
        &[0xFF, 0xA5, 0xA5, 0xA5, 0xA5, 0xA5, 0xA5, 0xA5],
    );

    assert!(matches!(
        decode_candidate(&wire),
        Err(CandidateError::InvalidUtf8)
    ));
}

// OASIS KMIP Specification v2.1, §§10.1.2 and 11.23; traceability
// KMIPKIT-0005-NR-002 requires the exact eight-byte Boolean encodings.
#[test]
fn rejects_a_boolean_value_other_than_the_defined_false_or_true_encoding() {
    let wire = item_bytes(ASSIGNED_TAG, BOOLEAN, 8, &[0, 0, 0, 0, 0, 0, 0, 2]);

    assert!(matches!(
        decode_candidate(&wire),
        Err(CandidateError::InvalidBoolean)
    ));
}

// KMIPKit project requirement KMIPKIT-0005-FR-005 rejects Item Type codes the
// generic model cannot represent. OASIS KMIP Specification v2.1 §11.23 lists
// the assigned Item Types (`KMIPKIT-0005-NR-002`); rejection of an unsupported
// code is project policy.
#[test]
fn rejects_an_unsupported_item_type_code() {
    let wire = item_bytes(ASSIGNED_TAG, 0x7F, 0, &[]);

    assert!(matches!(
        decode_candidate(&wire),
        Err(CandidateError::UnsupportedItemType)
    ));
}

// OASIS KMIP Specification v2.1, §10.1.5; traceability KMIPKIT-0005-NR-005
// specifies padding extents but no required octet value. KMIPKit project
// requirement KMIPKIT-0005-FR-006 therefore accepts these nonzero octets.
#[test]
fn accepts_nonzero_padding_octets_when_the_required_extents_are_present() {
    let vectors = [
        item_bytes(
            ASSIGNED_TAG,
            INTEGER,
            4,
            &[0, 0, 0, 1, 0xD1, 0xD2, 0xD3, 0xD4],
        ),
        item_bytes(
            ASSIGNED_TAG,
            ENUMERATION,
            4,
            &[0, 0, 0, 1, 0xD1, 0xD2, 0xD3, 0xD4],
        ),
        item_bytes(
            ASSIGNED_TAG,
            INTERVAL,
            4,
            &[0, 0, 0, 1, 0xD1, 0xD2, 0xD3, 0xD4],
        ),
        item_bytes(
            ASSIGNED_TAG,
            TEXT_STRING,
            1,
            &[b'x', 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7],
        ),
        item_bytes(
            ASSIGNED_TAG,
            BYTE_STRING,
            1,
            &[0x80, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7],
        ),
    ];

    for wire in vectors {
        assert!(
            decode_candidate(&wire).is_ok(),
            "padding values are unconstrained when the required extent is present"
        );
    }
}

// OASIS KMIP Specification v2.1, §10.1.5; traceability
// KMIPKIT-0005-NR-005 covers exact four-byte and minimum string/byte padding
// extents. Extra bytes after the one item are rejected under KMIPKit project
// requirement KMIPKIT-0005-FR-004, rather than treated as padding.
#[test]
fn rejects_short_padding_and_bytes_beyond_each_required_padding_extent() {
    let cases = [
        (INTEGER, 4, vec![0; 4], 4),
        (ENUMERATION, 4, vec![0; 4], 4),
        (INTERVAL, 4, vec![0; 4], 4),
        (TEXT_STRING, 3, vec![b'x'; 3], 5),
        (BYTE_STRING, 3, vec![0x80; 3], 5),
    ];

    for (item_type, item_length, value, required_padding) in cases {
        let short_body = [value.clone(), vec![0xA5; required_padding - 1]].concat();
        let short = item_bytes(ASSIGNED_TAG, item_type, item_length, &short_body);
        assert!(
            matches!(
                decode_candidate(&short),
                Err(CandidateError::InvalidPaddingExtent)
            ),
            "Item Type 0x{item_type:02X} must reject one missing padding octet"
        );

        let long_body = [value, vec![0xA5; required_padding + 1]].concat();
        let long = item_bytes(ASSIGNED_TAG, item_type, item_length, &long_body);
        assert!(
            matches!(decode_candidate(&long), Err(CandidateError::TrailingBytes)),
            "Item Type 0x{item_type:02X} must not consume an extra octet as padding"
        );
    }
}

// KMIPKit project requirement KMIPKIT-0005-FR-006 checks Structure child
// boundaries; OASIS KMIP Specification v2.1, §§10.1.2, 10.1.3, and 10.1.5
// define nested Structure values and the length inclusion rule. Traceability:
// KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-004, and KMIPKIT-0005-NR-005.
#[test]
fn rejects_a_child_that_crosses_its_parent_structure_boundary() {
    let child = item_bytes(
        ASSIGNED_TAG,
        INTEGER,
        4,
        &[0, 0, 0, 1, 0xA5, 0xA5, 0xA5, 0xA5],
    );
    let wire = item_bytes(ASSIGNED_TAG, STRUCTURE, 8, &child);

    assert!(matches!(
        decode_candidate(&wire),
        Err(CandidateError::StructureBoundary)
    ));
}

// KMIPKit project requirement KMIPKIT-0005-FR-006 rejects child data that
// cannot fill its declared parent Structure extent. OASIS KMIP Specification
// v2.1, §§10.1.2, 10.1.3, and 10.1.5 define child encodings and Structure
// length. Traceability: KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-004, and
// KMIPKIT-0005-NR-005.
#[test]
fn rejects_a_structure_whose_declared_extent_exceeds_available_input() {
    let child_header_only = item_bytes(ASSIGNED_TAG, INTEGER, 4, &[]);
    let wire = item_bytes(ASSIGNED_TAG, STRUCTURE, 16, &child_header_only);

    assert!(matches!(
        decode_candidate(&wire),
        Err(CandidateError::TruncatedValue)
    ));
}

// OASIS KMIP Specification v2.1, §10.1.3; traceability
// KMIPKIT-0005-NR-004 requires Item Length to fit the U32 field. KMIPKit
// project requirement KMIPKIT-0005-FR-008 rejects a declared span that is not
// available before allocation; this boundary vector uses no giant allocation.
#[test]
fn checks_a_maximum_u32_length_against_available_input_before_allocation() {
    let wire = item_bytes(ASSIGNED_TAG, BYTE_STRING, u32::MAX, &[]);

    assert!(matches!(
        decode_candidate(&wire),
        Err(CandidateError::TruncatedValue)
    ));
}

// OASIS KMIP Specification v2.1, §§10.1.1–10.1.3; traceability
// KMIPKIT-0005-NR-001 and KMIPKIT-0005-NR-004 cover the fixed header fields and
// declared value span used by this boundary check.
#[test]
fn rejects_a_value_whose_declared_extent_is_not_available() {
    let wire = item_bytes(ASSIGNED_TAG, BYTE_STRING, 9, &[0; 8]);

    assert!(matches!(
        decode_candidate(&wire),
        Err(CandidateError::TruncatedValue)
    ));
}
