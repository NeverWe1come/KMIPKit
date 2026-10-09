//! Encrypt request Data tests derived from OASIS KMIP v2.1 §6.1.17,
//! Table 214, and §7.9, Tables 360–361. These are source-derived tests, not
//! official OASIS Test Cases.
//!
//! Traceability: `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA` and
//! `KMIPKIT-REQ-SPEC-6.1.17-001-001`.

use std::fmt::{Debug, Display};

use crate::{EncryptRequest, OperationData, SecretBytes, UniqueIdentifier};
use kmipkit_ttlv::{ItemType, Structure, ValueView};

const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
const DATA_TAG: u32 = 0x0042_00C2;
const IV_COUNTER_NONCE_TAG: u32 = 0x0042_003D;
const CORRELATION_VALUE_TAG: u32 = 0x0042_00D6;
const INIT_INDICATOR_TAG: u32 = 0x0042_00D7;
const FINAL_INDICATOR_TAG: u32 = 0x0042_00D8;
const AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA_TAG: u32 = 0x0042_00FE;
const BYTE_STRING_SENTINELS: &[&str] = &["167", "184", "201", "a7", "b8", "c9"];
const ENUMERATION_SENTINELS: &[&str] = &["3735928559", "deadbeef"];
const INTEGER_SENTINELS: &[&str] = &["-2147483648"];

fn request_payload(data: OperationData) -> Structure {
    EncryptRequest::new(Some(UniqueIdentifier::TextString(
        "object-identifier".to_owned(),
    )))
    .with_data(data)
    .into_ttlv_payload()
    .expect("a source-derived request Data encoding forms a valid payload")
}

fn data_item_index(payload: &Structure) -> usize {
    payload
        .view()
        .children()
        .iter()
        .position(|item| item.tag().raw() == DATA_TAG)
        .expect("Table 214 request contains the supplied Data member")
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
fn request_data_preserves_byte_string_type_and_exact_value() {
    let expected = [0x00, 0x7F, 0x80, 0xFF];
    let payload = request_payload(OperationData::ByteString(SecretBytes::new(
        expected.to_vec(),
    )));
    let view = payload.view();
    let data = &view.children()[data_item_index(&payload)];

    assert_eq!(data.item_type(), ItemType::ByteString);
    assert!(data.with_value(|value| {
        matches!(value, ValueView::ByteString(actual) if *actual == expected)
    }));
}

#[test]
fn request_data_preserves_enumeration_type_and_exact_unknown_value() {
    let expected = u32::MAX;
    let payload = request_payload(OperationData::Enumeration(expected));
    let view = payload.view();
    let data = &view.children()[data_item_index(&payload)];

    assert_eq!(data.item_type(), ItemType::Enumeration);
    assert!(data.with_value(|value| {
        matches!(value, ValueView::Enumeration(actual) if *actual == expected)
    }));
}

#[test]
fn request_data_preserves_integer_type_and_exact_signed_value() {
    let expected = i32::MIN;
    let payload = request_payload(OperationData::Integer(expected));
    let view = payload.view();
    let data = &view.children()[data_item_index(&payload)];

    assert_eq!(data.item_type(), ItemType::Integer);
    assert!(data.with_value(|value| {
        matches!(value, ValueView::Integer(actual) if *actual == expected)
    }));
}

#[test]
fn request_data_remains_in_table_214_member_order() {
    let payload = EncryptRequest::new(Some(UniqueIdentifier::TextString(
        "object-identifier".to_owned(),
    )))
    .with_cryptographic_parameters(Structure::new())
    .with_data(OperationData::Enumeration(0x8000_0001))
    .with_iv_counter_nonce(SecretBytes::new(vec![0x11]))
    .with_correlation_value(SecretBytes::new(vec![0x22]))
    .with_init_indicator(true)
    .with_final_indicator(false)
    .with_authenticated_encryption_additional_data(SecretBytes::new(vec![0x33]))
    .into_ttlv_payload()
    .expect("Table 214 fields form an ordered request payload");
    let actual_tags = payload
        .view()
        .children()
        .iter()
        .map(|item| item.tag().raw())
        .collect::<Vec<_>>();

    assert_eq!(
        actual_tags,
        [
            UNIQUE_IDENTIFIER_TAG,
            CRYPTOGRAPHIC_PARAMETERS_TAG,
            DATA_TAG,
            IV_COUNTER_NONCE_TAG,
            CORRELATION_VALUE_TAG,
            INIT_INDICATOR_TAG,
            FINAL_INDICATOR_TAG,
            AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA_TAG,
        ]
    );
}

#[test]
fn debug_and_display_redact_every_request_data_variant() {
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
fn byte_string_request_uses_secret_bytes_and_zeroizing_ttlv_ownership() {
    // This variant requires the existing zeroizing SecretBytes owner. The
    // request serializes to kmipkit-ttlv's owned ByteString path, whose drop
    // zeroization is covered by that crate's value_zeroization tests. A safe
    // public API cannot inspect storage after deallocation.
    let expected = [0xA7, 0xB8, 0xC9];
    let payload = request_payload(OperationData::ByteString(SecretBytes::new(
        expected.to_vec(),
    )));
    let view = payload.view();
    let data = &view.children()[data_item_index(&payload)];

    assert!(data.with_value(|value| {
        matches!(value, ValueView::ByteString(actual) if *actual == expected)
    }));
    drop(payload);
}
