//! Shared protocol contracts for Hash, MAC, and Signature operations.
//! Source basis: OASIS KMIP v2.1 §4.16, §§7.8, 7.9, 7.14, 7.17, 7.20, 7.38,
//! §§11.12, 11.16, 11.21, and 11.61; operation Tables 235, 259, 262, 334,
//! and 337. These are source-derived contract tests.

use std::error::Error;

use crate::async_operation_fixtures::response_message;
use crate::cryptographic_operation_test_support::{payload, response_item};
use crate::{
    CryptographicAlgorithm, CryptographicOperationErrorKind, DigitalSignatureAlgorithm,
    HashRequest, HashResponse, HashingAlgorithm, KmipOperationResult, MacRequest, MacResponse,
    MacVerifyRequest, MacVerifyResponse, OperationData, ResultReason, ResultStatus, SecretBytes,
    SignRequest, SignResponse, SignatureVerifyRequest, SignatureVerifyResponse, UniqueIdentifier,
    ValidityIndicator,
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
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;

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
    let error = HashResponse::try_from_response_item(response_item(&wrong_operation))
        .expect_err("the typed Hash parser rejects a MAC response");
    assert_eq!(
        error.kind(),
        CryptographicOperationErrorKind::UnexpectedOperation
    );
    assert!(Error::source(&error).is_none());
}

#[test]
fn every_crypto_request_shape_method_accepts_single_part_and_rejects_missing_input() {
    let data = || OperationData::ByteString(SecretBytes::new(b"shape-input".to_vec()));

    assert!(
        HashRequest::new(Structure::new())
            .validate_multipart_shape()
            .is_err()
    );
    assert!(
        HashRequest::new(Structure::new())
            .with_data(data())
            .validate_multipart_shape()
            .is_ok()
    );

    assert!(MacRequest::new().validate_multipart_shape().is_err());
    assert!(
        MacRequest::new()
            .with_data(data())
            .validate_multipart_shape()
            .is_ok()
    );

    assert!(MacVerifyRequest::new().validate_multipart_shape().is_err());
    assert!(
        MacVerifyRequest::new()
            .with_mac_data(SecretBytes::new(b"mac".to_vec()))
            .validate_multipart_shape()
            .is_ok()
    );

    assert!(SignRequest::new().validate_multipart_shape().is_err());
    assert!(
        SignRequest::new()
            .with_data(data())
            .validate_multipart_shape()
            .is_ok()
    );

    assert!(
        SignatureVerifyRequest::new()
            .validate_multipart_shape()
            .is_err()
    );
    assert!(
        SignatureVerifyRequest::new()
            .with_signature_data(SecretBytes::new(b"signature".to_vec()))
            .validate_multipart_shape()
            .is_ok()
    );
}

#[test]
fn every_crypto_request_shape_method_redacts_invalid_framing_errors() {
    let data = || OperationData::ByteString(SecretBytes::new(b"SHAPE-SECRET-SENTINEL".to_vec()));
    let secret = "SHAPE-SECRET-SENTINEL";

    let errors = [
        HashRequest::new(Structure::new())
            .with_data(data())
            .with_final_indicator(true)
            .validate_multipart_shape()
            .expect_err("Hash Final without Correlation Value is invalid"),
        MacRequest::new()
            .with_data(data())
            .with_final_indicator(true)
            .validate_multipart_shape()
            .expect_err("MAC Final without Correlation Value is invalid"),
        MacVerifyRequest::new()
            .with_mac_data(SecretBytes::new(secret.as_bytes().to_vec()))
            .with_final_indicator(true)
            .validate_multipart_shape()
            .expect_err("MAC Verify Final without Correlation Value is invalid"),
        SignRequest::new()
            .with_data(data())
            .with_final_indicator(true)
            .validate_multipart_shape()
            .expect_err("Sign Final without Correlation Value is invalid"),
        SignatureVerifyRequest::new()
            .with_signature_data(SecretBytes::new(secret.as_bytes().to_vec()))
            .with_final_indicator(true)
            .validate_multipart_shape()
            .expect_err("Signature Verify Final without Correlation Value is invalid"),
    ];
    for error in errors {
        assert!(!error.to_string().contains(secret));
    }
}

#[test]
fn mac_request_preserves_enumeration_and_integer_unique_identifier_values() {
    for identifier in [
        UniqueIdentifier::Enumeration(0xF123_4567),
        UniqueIdentifier::Integer(-12_345),
    ] {
        let payload = MacRequest::new()
            .with_unique_identifier(identifier.clone())
            .with_data(OperationData::ByteString(SecretBytes::new(
                b"input".to_vec(),
            )))
            .to_ttlv_payload()
            .expect("MAC accepts every lossless Unique Identifier form");
        let view = payload.view();
        let item = &view.children()[0];
        assert_eq!(item.tag().raw(), UNIQUE_IDENTIFIER);
        let preserved = item.with_value(|value| match (identifier, value) {
            (UniqueIdentifier::Enumeration(expected), ValueView::Enumeration(actual)) => {
                expected == *actual
            }
            (UniqueIdentifier::Integer(expected), ValueView::Integer(actual)) => {
                expected == *actual
            }
            _ => false,
        });
        assert!(preserved, "MAC changed the caller-supplied identifier form");
    }
}

#[test]
fn internal_crypto_errors_preserve_sources_without_exposing_sensitive_values() {
    let result_cause = KmipOperationResult::new(
        ResultStatus::from_raw(SUCCESS),
        Some(ResultReason::from_raw(0x0000_0100)),
        None,
    )
    .expect_err("Success with a Result Reason violates the common result contract");
    let response_error = crate::cryptographic_operation::CryptographicOperationError::new(
        "Hash",
        CryptographicOperationErrorKind::InvalidOperationResult(result_cause),
    );
    assert!(Error::source(&response_error).is_some());
    assert!(!response_error.to_string().contains("SHAPE-SECRET-SENTINEL"));

    let model_error = crate::cryptographic_operation::push(
        &mut Structure::new(),
        0,
        Value::byte_string(b"SHAPE-SECRET-SENTINEL".to_vec()),
    )
    .expect_err("an unallocated TTLV tag is rejected");
    assert!(Error::source(&model_error).is_some());
    assert!(!model_error.to_string().contains("SHAPE-SECRET-SENTINEL"));
}

#[test]
fn malformed_successful_unique_identifier_is_rejected_without_echoing_bytes() {
    let sentinel = b"MALFORMED-IDENTIFIER-SECRET";
    let response = response_message(
        MAC,
        SUCCESS,
        None,
        None,
        Some(payload([(
            UNIQUE_IDENTIFIER,
            Value::byte_string(sentinel.to_vec()),
        )])),
    );
    let error = MacResponse::try_from_response_item(response_item(&response))
        .expect_err("MAC requires a table-defined Unique Identifier value type");
    assert_eq!(
        error.kind(),
        CryptographicOperationErrorKind::MalformedPayload
    );
    assert!(!error.to_string().contains("MALFORMED-IDENTIFIER-SECRET"));
}

#[test]
fn single_part_hash_mac_and_sign_successes_require_their_output_fields() {
    // OASIS KMIP v2.1 §6.1.24 Table 236, §6.1.32 Table 260, and
    // §6.1.55 Table 335 require Hash Data, MAC Data, and Signature Data for
    // single-part responses.
    let hash = response_message(HASH, SUCCESS, None, None, Some(Structure::new()));
    let mac = response_message(
        MAC,
        SUCCESS,
        None,
        None,
        Some(payload([(
            UNIQUE_IDENTIFIER,
            Value::text_string("mac-object".to_owned()),
        )])),
    );
    let sign = response_message(
        SIGN,
        SUCCESS,
        None,
        None,
        Some(payload([(
            UNIQUE_IDENTIFIER,
            Value::text_string("sign-object".to_owned()),
        )])),
    );

    let rejected = [
        HashResponse::try_from_response_item(response_item(&hash)).is_err(),
        MacResponse::try_from_response_item(response_item(&mac)).is_err(),
        SignResponse::try_from_response_item(response_item(&sign)).is_err(),
    ];
    assert_eq!(rejected, [true; 3]);
}
