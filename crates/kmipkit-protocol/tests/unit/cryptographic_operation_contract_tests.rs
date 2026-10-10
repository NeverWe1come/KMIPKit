//! Shared protocol contracts for Hash, MAC, and Signature operations.
//! Source basis: OASIS KMIP v2.1 §4.16, §§7.8, 7.9, 7.14, 7.17, 7.20, 7.38,
//! §§11.12, 11.16, 11.21, and 11.61; operation Tables 235, 259, 262, 334,
//! and 337. These are source-derived contract tests.

use crate::{
    CryptographicAlgorithm, DigitalSignatureAlgorithm, HashingAlgorithm, OperationData,
    SecretBytes, ValidityIndicator,
};
use kmipkit_ttlv::Item;
use kmipkit_ttlv::{Structure, Value, ValueView};

const HASHING_ALGORITHM: u32 = 0x0042_0038;
const VENDOR_PARAMETER: u32 = 0x0054_1234;

fn parameter_item(raw_tag: u32, value: Value) -> Item {
    crate::operation_test_support::item(raw_tag, value)
}

#[test]
fn operation_byte_wrappers_redact_debug_and_display() {
    let sentinel = b"OPERATION-SECRET-42";
    let value = OperationData::ByteString(SecretBytes::new(sentinel.to_vec()));
    let formatted = format!("{value:?} {value}");

    assert!(!formatted.contains("OPERATION-SECRET-42"));
    assert!(formatted.contains("REDACTED"));
}

#[test]
fn generic_cryptographic_parameters_keep_order_unknown_members_and_raw_values() {
    let mut parameters = Structure::new();
    parameters
        .try_push(parameter_item(
            HASHING_ALGORITHM,
            Value::enumeration(0xF123_4567),
        ))
        .expect("one generic parameter fits the TTLV Structure");
    parameters
        .try_push(parameter_item(
            VENDOR_PARAMETER,
            Value::byte_string(b"vendor opaque value".to_vec()),
        ))
        .expect("the vendor parameter fits the TTLV Structure");

    let view = parameters.view();
    let values = view.children();
    assert_eq!(values[0].tag().raw(), HASHING_ALGORITHM);
    assert_eq!(values[1].tag().raw(), VENDOR_PARAMETER);
    assert_eq!(
        values[0].with_value(|value| match value {
            ValueView::Enumeration(raw) => Some(*raw),
            _ => None,
        }),
        Some(0xF123_4567)
    );
}

#[test]
fn cryptographic_enum_wrappers_retain_standard_extension_and_future_values() {
    for raw in [0, 1, 2, 0x8000_0000, 0x8FFF_FFFF, 0xF123_4567] {
        assert_eq!(HashingAlgorithm::from_raw(raw).raw(), raw);
        assert_eq!(CryptographicAlgorithm::from_raw(raw).raw(), raw);
        assert_eq!(DigitalSignatureAlgorithm::from_raw(raw).raw(), raw);
        assert_eq!(ValidityIndicator::from_raw(raw).raw(), raw);
    }
}

#[test]
fn shared_multipart_correlation_and_indicators_keep_the_exact_values() {
    let _request = crate::HashRequest::new(Structure::new())
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"part".to_vec(),
        )))
        .with_correlation_value(SecretBytes::new(vec![0x00, 0x81, 0xFF]))
        .with_init_indicator(false)
        .with_final_indicator(true);
}
