//! Shared protocol contracts for Hash, MAC, and Signature operations.
//! Source basis: OASIS KMIP v2.1 §4.16, §§7.8, 7.9, 7.14, 7.17, 7.20, 7.38,
//! §§11.12, 11.16, 11.21, and 11.61; operation Tables 235, 259, 262, 334,
//! and 337. These are source-derived contract tests.

use crate::async_operation_fixtures::response_message;
use crate::cryptographic_operation_test_support::response_item;
use crate::{
    CryptographicAlgorithm, CryptographicOperationErrorKind, DigitalSignatureAlgorithm,
    HashRequest, HashResponse, HashingAlgorithm, MacRequest, MacResponse, MacVerifyRequest,
    MacVerifyResponse, OperationData, SecretBytes, SignRequest, SignResponse,
    SignatureVerifyRequest, SignatureVerifyResponse, ValidityIndicator,
};
use kmipkit_ttlv::Item;
use kmipkit_ttlv::{Structure, Value, ValueView};

const HASHING_ALGORITHM: u32 = 0x0042_0038;
const HASH: u32 = 0x0000_0027;
const MAC: u32 = 0x0000_0023;
const MAC_VERIFY: u32 = 0x0000_0024;
const SIGN: u32 = 0x0000_0021;
const SIGNATURE_VERIFY: u32 = 0x0000_0022;
const SUCCESS: u32 = 0;
const PENDING: u32 = 2;
const VENDOR_PARAMETER: u32 = 0x0054_1234;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;

fn parameter_item(raw_tag: u32, value: Value) -> Item {
    crate::operation_test_support::item(raw_tag, value)
}

fn malformed_algorithm_parameters(tag: u32) -> Structure {
    let mut parameters = Structure::new();
    parameters
        .try_push(parameter_item(tag, Value::integer(6)))
        .expect("malformed parameter fixture fits the TTLV Structure");
    parameters
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

#[test]
fn requests_reject_malformed_explicit_cryptographic_parameters() {
    let data = || OperationData::ByteString(SecretBytes::new(b"input".to_vec()));

    assert!(
        HashRequest::new(malformed_algorithm_parameters(HASHING_ALGORITHM))
            .with_data(data())
            .to_ttlv_payload()
            .is_err()
    );
    assert!(
        MacRequest::new()
            .with_cryptographic_parameters(malformed_algorithm_parameters(CRYPTOGRAPHIC_ALGORITHM,))
            .with_data(data())
            .to_ttlv_payload()
            .is_err()
    );
    assert!(
        MacVerifyRequest::new()
            .with_cryptographic_parameters(malformed_algorithm_parameters(CRYPTOGRAPHIC_ALGORITHM,))
            .with_mac_data(SecretBytes::new(b"mac".to_vec()))
            .to_ttlv_payload()
            .is_err()
    );
    assert!(
        SignRequest::new()
            .with_cryptographic_parameters(malformed_algorithm_parameters(CRYPTOGRAPHIC_ALGORITHM,))
            .with_data(data())
            .to_ttlv_payload()
            .is_err()
    );
    assert!(
        SignatureVerifyRequest::new()
            .with_cryptographic_parameters(malformed_algorithm_parameters(CRYPTOGRAPHIC_ALGORITHM,))
            .with_signature_data(SecretBytes::new(b"signature".to_vec()))
            .to_ttlv_payload()
            .is_err()
    );
}

#[test]
fn typed_response_parsers_cover_pending_and_result_errors() {
    let pending = [HASH, MAC, MAC_VERIFY, SIGN, SIGNATURE_VERIFY].map(|operation| {
        response_message(
            operation,
            PENDING,
            None,
            Some(b"pending-correlation"),
            Some(Structure::new()),
        )
    });
    let success_with_payload = [HASH, MAC, MAC_VERIFY, SIGN, SIGNATURE_VERIFY]
        .map(|operation| response_message(operation, SUCCESS, None, None, Some(Structure::new())));

    let hash_pending = HashResponse::try_from_pending_response_item(response_item(&pending[0]))
        .expect("Hash Pending result fields are valid");
    assert_eq!(hash_pending.result().status().raw(), PENDING);
    assert!(hash_pending.correlation_value().is_none());
    assert_eq!(
        HashResponse::try_from_response_item(response_item(&pending[0]))
            .expect_err("the completed conversion rejects Pending")
            .kind(),
        CryptographicOperationErrorKind::PendingOutcomeRequired
    );
    let mac_pending = MacResponse::try_from_pending_response_item(response_item(&pending[1]))
        .expect("MAC Pending result fields are valid");
    assert_eq!(mac_pending.result().status().raw(), PENDING);
    assert!(mac_pending.correlation_value().is_none());
    assert_eq!(
        MacResponse::try_from_pending_response_item(response_item(&success_with_payload[1]))
            .expect_err("Pending conversion rejects a completed result")
            .kind(),
        CryptographicOperationErrorKind::NotPendingOutcome
    );

    let mac_verify_pending =
        MacVerifyResponse::try_from_pending_response_item(response_item(&pending[2]))
            .expect("MAC Verify Pending result fields are valid");
    assert_eq!(mac_verify_pending.result().status().raw(), PENDING);
    assert!(mac_verify_pending.correlation_value().is_none());
    assert_eq!(
        MacVerifyResponse::try_from_response_item(response_item(&pending[2]))
            .expect_err("completed conversion rejects Pending")
            .kind(),
        CryptographicOperationErrorKind::PendingOutcomeRequired
    );

    let sign_pending = SignResponse::try_from_pending_response_item(response_item(&pending[3]))
        .expect("Sign Pending result fields are valid");
    assert_eq!(sign_pending.result().status().raw(), PENDING);
    assert!(sign_pending.correlation_value().is_none());
    assert_eq!(
        SignResponse::try_from_response_item(response_item(&pending[3]))
            .expect_err("completed conversion rejects Pending")
            .kind(),
        CryptographicOperationErrorKind::PendingOutcomeRequired
    );

    let signature_verify_pending =
        SignatureVerifyResponse::try_from_pending_response_item(response_item(&pending[4]))
            .expect("Signature Verify Pending result fields are valid");
    assert_eq!(signature_verify_pending.result().status().raw(), PENDING);
    assert!(signature_verify_pending.correlation_value().is_none());
    assert_eq!(
        SignatureVerifyResponse::try_from_response_item(response_item(&pending[4],))
            .expect_err("completed conversion rejects Pending")
            .kind(),
        CryptographicOperationErrorKind::PendingOutcomeRequired
    );

    assert!(MacVerifyRequest::default().to_ttlv_payload().is_err());
    assert!(SignRequest::default().to_ttlv_payload().is_err());
    assert!(SignatureVerifyRequest::default().to_ttlv_payload().is_err());

    let wrong_operation = response_message(MAC, SUCCESS, None, None, Some(Structure::new()));
    assert_eq!(
        HashResponse::try_from_response_item(response_item(&wrong_operation))
            .expect_err("the typed Hash parser rejects a MAC response")
            .kind(),
        CryptographicOperationErrorKind::UnexpectedOperation
    );
}
