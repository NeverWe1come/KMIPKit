//! Operation Data tests derived from OASIS KMIP v2.1 §7.9, Tables 360–361.
//! These are source-derived shared-value tests contributing to
//! KMIPKIT-0019-FR-001, not official OASIS Test Cases.
//!
//! Traceability: `KMIPKIT-0019-FR-001` and
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA`.

use std::fmt::{Debug, Display};

use crate::{OperationData, SecretBytes, operation_test_support::item};
use kmipkit_ttlv::{Item, ItemType, ValueView};

const DATA_TAG: u32 = 0x0042_00C2;
const BYTE_STRING_SENTINELS: &[&str] = &["167", "184", "201", "a7", "b8", "c9"];
const ENUMERATION_SENTINELS: &[&str] = &["3735928559", "deadbeef"];
const INTEGER_SENTINELS: &[&str] = &["-2147483648"];

fn operation_data_item(data: OperationData) -> Item {
    item(DATA_TAG, data.into_ttlv_value())
}

fn assert_redacted<T: Debug + Display>(value: &T, sentinels: &[&str]) {
    let debug = format!("{value:?}");
    let display = value.to_string();

    for sentinel in sentinels {
        let sentinel = sentinel.to_ascii_lowercase();
        assert!(
            !debug.to_ascii_lowercase().contains(&sentinel),
            "Debug representation exposed an OperationData value"
        );
        assert!(
            !display.to_ascii_lowercase().contains(&sentinel),
            "Display representation exposed an OperationData value"
        );
    }
}

#[test]
fn operation_data_preserves_byte_string_type_and_exact_value() {
    let expected = [0x00, 0x7F, 0x80, 0xFF];
    let data = operation_data_item(OperationData::ByteString(SecretBytes::new(
        expected.to_vec(),
    )));

    assert_eq!(data.item_type(), ItemType::ByteString);
    assert!(data.with_value(|value| {
        matches!(value, ValueView::ByteString(actual) if *actual == expected)
    }));
}

#[test]
fn operation_data_preserves_enumeration_type_and_exact_unknown_value() {
    let expected = u32::MAX;
    let data = operation_data_item(OperationData::Enumeration(expected));

    assert_eq!(data.item_type(), ItemType::Enumeration);
    assert!(data.with_value(|value| {
        matches!(value, ValueView::Enumeration(actual) if *actual == expected)
    }));
}

#[test]
fn operation_data_preserves_integer_type_and_exact_signed_value() {
    let expected = i32::MIN;
    let data = operation_data_item(OperationData::Integer(expected));

    assert_eq!(data.item_type(), ItemType::Integer);
    assert!(data.with_value(|value| {
        matches!(value, ValueView::Integer(actual) if *actual == expected)
    }));
}

#[test]
fn debug_and_display_redact_every_operation_data_variant() {
    let cases = [
        (
            OperationData::ByteString(SecretBytes::new(vec![0xA7, 0xB8, 0xC9])),
            BYTE_STRING_SENTINELS,
        ),
        (
            OperationData::Enumeration(0xDEAD_BEEF),
            ENUMERATION_SENTINELS,
        ),
        (OperationData::Integer(i32::MIN), INTEGER_SENTINELS),
    ];

    for (data, sentinels) in cases {
        assert_redacted(&data, sentinels);
    }
}

#[test]
fn byte_string_data_uses_secret_bytes_and_zeroizing_ttlv_ownership() {
    // KMIPKit transfers owned bytes through SecretBytes into the TTLV byte
    // string owner. The TTLV crate's value_zeroization tests cover drop-time
    // zeroization; a safe public API cannot inspect storage after deallocation.
    let expected = [0xA7, 0xB8, 0xC9];
    let data = operation_data_item(OperationData::ByteString(SecretBytes::new(
        expected.to_vec(),
    )));

    assert!(data.with_value(|value| {
        matches!(value, ValueView::ByteString(actual) if *actual == expected)
    }));
    drop(data);
}
