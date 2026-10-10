//! Source-derived Signature Verify contracts from OASIS KMIP v2.1 §6.1.56,
//! Tables 337–339; Validity Indicator values follow §11.61. These tests are
//! not official Test Case executions.
//!
//! Traceability: FR-001, FR-005–FR-013, FR-015, and
//! `KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY`.

use crate::async_operation_fixtures::response_message;
use crate::cryptographic_operation_test_support::{
    field_tags, payload, response_item, response_payload_tags, snapshot,
};
use crate::{
    CryptographicAlgorithm, CryptographicOperationErrorKind, OperationData, SecretBytes,
    SignatureVerifyRequest, SignatureVerifyResponse, UniqueIdentifier, ValidityIndicator,
    VerificationResponseContext,
};
use kmipkit_ttlv::Value;

const SIGNATURE_VERIFY_OPERATION: u32 = 0x0000_0022;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const GENERAL_FAILURE: u32 = 0x0000_0100;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const DATA: u32 = 0x0042_00C2;
const SIGNATURE_DATA: u32 = 0x0042_00C7;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const INIT_INDICATOR: u32 = 0x0042_00D7;
const FINAL_INDICATOR: u32 = 0x0042_00D8;
const VALIDITY_INDICATOR: u32 = 0x0042_0128;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

fn success(payload: kmipkit_ttlv::Structure) -> crate::ResponseMessage {
    response_message(
        SIGNATURE_VERIFY_OPERATION,
        SUCCESS,
        None,
        None,
        Some(payload),
    )
}

#[test]
fn request_emits_optional_key_parameters_data_digest_signature_and_framing_in_order() {
    let data = [0x00, 0x7F, 0x80, 0xFF];
    let digest = [0xA0, 0xB1, 0xFE];
    let signature = [0x01, 0x81, 0xFF];
    let mut parameters = kmipkit_ttlv::Structure::new();
    parameters
        .try_push(crate::operation_test_support::item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(CryptographicAlgorithm::from_raw(0x38).raw()),
        ))
        .expect("the table fixture contains one algorithm parameter");
    let request = SignatureVerifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("verification-key".to_owned()))
        .with_cryptographic_parameters(parameters)
        .with_data(OperationData::ByteString(SecretBytes::new(data.to_vec())))
        .with_digested_data(SecretBytes::new(digest.to_vec()))
        .with_signature_data(SecretBytes::new(signature.to_vec()))
        .with_init_indicator(true)
        .with_final_indicator(true);

    let actual = request
        .to_ttlv_payload()
        .expect("Table 337 allows optional original Data and Digested Data");

    assert_eq!(
        field_tags(&actual),
        [
            UNIQUE_IDENTIFIER,
            CRYPTOGRAPHIC_PARAMETERS,
            DATA,
            0x0042_0107,
            SIGNATURE_DATA,
            INIT_INDICATOR,
            FINAL_INDICATOR,
        ]
    );
    assert_eq!(
        snapshot(&actual.view().children()[2]),
        (DATA, data.to_vec())
    );
    assert_eq!(
        snapshot(&actual.view().children()[3]),
        (0x0042_0107, digest.to_vec())
    );
    assert_eq!(
        snapshot(&actual.view().children()[4]),
        (SIGNATURE_DATA, signature.to_vec())
    );
}

#[test]
fn request_allows_optional_key_parameters_and_input_data_to_be_omitted() {
    let signature = [0x00, 0x81, 0xFF];
    let actual = SignatureVerifyRequest::new()
        .with_signature_data(SecretBytes::new(signature.to_vec()))
        .to_ttlv_payload()
        .expect("Table 337 makes the key, parameters, Data, and Digested Data optional");

    assert_eq!(field_tags(&actual), [SIGNATURE_DATA]);
    assert_eq!(
        snapshot(&actual.view().children()[0]),
        (SIGNATURE_DATA, signature.to_vec())
    );
}

#[test]
fn request_requires_signature_data_for_single_part_and_omits_it_for_multipart() {
    assert!(
        SignatureVerifyRequest::new().to_ttlv_payload().is_err(),
        "Table 337 requires Signature Data for single-part verification"
    );

    let actual = SignatureVerifyRequest::new()
        .with_correlation_value(SecretBytes::new(b"signature-stream".to_vec()))
        .with_init_indicator(false)
        .with_final_indicator(false)
        .to_ttlv_payload()
        .expect("Table 337 forbids Signature Data on a multipart request");
    assert_eq!(
        field_tags(&actual),
        [CORRELATION_VALUE, INIT_INDICATOR, FINAL_INDICATOR]
    );
}

#[test]
fn request_classifies_single_part_initial_non_final_and_final_multipart_contexts() {
    let single_part = SignatureVerifyRequest::new()
        .with_signature_data(SecretBytes::new(b"signature".to_vec()))
        .verification_response_context();
    let framed_single_part = SignatureVerifyRequest::new()
        .with_signature_data(SecretBytes::new(b"signature".to_vec()))
        .with_init_indicator(true)
        .with_final_indicator(true)
        .verification_response_context();
    let initial_non_final = SignatureVerifyRequest::new()
        .with_init_indicator(true)
        .with_final_indicator(false)
        .verification_response_context();
    let continuation_non_final = SignatureVerifyRequest::new()
        .with_correlation_value(SecretBytes::new(b"stream".to_vec()))
        .with_final_indicator(false)
        .verification_response_context();
    let final_part = SignatureVerifyRequest::new()
        .with_correlation_value(SecretBytes::new(b"stream".to_vec()))
        .with_final_indicator(true)
        .verification_response_context();

    assert_eq!(single_part, VerificationResponseContext::SinglePart);
    assert_eq!(framed_single_part, VerificationResponseContext::SinglePart);
    assert_eq!(
        initial_non_final,
        VerificationResponseContext::MultipartNonFinal
    );
    assert_eq!(
        continuation_non_final,
        VerificationResponseContext::MultipartNonFinal
    );
    assert_eq!(final_part, VerificationResponseContext::MultipartFinal);
}

#[test]
fn successful_single_part_response_requires_one_identifier_and_validity_indicator() {
    let missing = success(payload([(
        UNIQUE_IDENTIFIER,
        Value::text_string("key".to_owned()),
    )]));
    assert_eq!(
        SignatureVerifyResponse::try_from_response_item(response_item(&missing))
            .expect_err("single-part Table 338 requires Validity Indicator")
            .kind(),
        CryptographicOperationErrorKind::MalformedPayload
    );

    for raw in [1, 2, 3, 0x8000_0000, u32::MAX] {
        let recovered = [0x00, 0x80, 0xFF];
        let message = success(payload([
            (UNIQUE_IDENTIFIER, Value::text_string("key".to_owned())),
            (VALIDITY_INDICATOR, Value::enumeration(raw)),
            (DATA, Value::byte_string(recovered.to_vec())),
            (UNKNOWN_VENDOR_TAG, Value::integer(-7)),
        ]));
        let item = response_item(&message);
        let typed = SignatureVerifyResponse::try_from_response_item(item)
            .expect("single-part Validity Indicator values remain open");
        assert_eq!(
            typed.validity_indicator(),
            Some(ValidityIndicator::from_raw(raw))
        );
        typed
            .recovered_data()
            .expect("Table 338 allows recovered Data")
            .with_bytes(|actual| assert_eq!(actual, recovered));
        assert_eq!(
            response_payload_tags(item),
            Some(vec![
                UNIQUE_IDENTIFIER,
                VALIDITY_INDICATOR,
                DATA,
                UNKNOWN_VENDOR_TAG
            ])
        );
    }
}

#[test]
fn successful_response_requires_one_well_typed_unique_identifier() {
    let cases = [
        payload([(VALIDITY_INDICATOR, Value::enumeration(1))]),
        payload([
            (UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            (UNIQUE_IDENTIFIER, Value::text_string("second".to_owned())),
            (VALIDITY_INDICATOR, Value::enumeration(1)),
        ]),
        payload([
            (UNIQUE_IDENTIFIER, Value::boolean(true)),
            (VALIDITY_INDICATOR, Value::enumeration(1)),
        ]),
    ];

    for response_payload in cases {
        let message = success(response_payload);
        assert_eq!(
            SignatureVerifyResponse::try_from_response_item(response_item(&message))
                .expect_err("missing, repeated, and malformed identifiers are rejected")
                .kind(),
            CryptographicOperationErrorKind::MalformedPayload
        );
    }
}

#[test]
fn successful_response_rejects_duplicate_or_malformed_validity_and_recovered_data() {
    let cases = [
        payload([
            (UNIQUE_IDENTIFIER, Value::text_string("key".to_owned())),
            (VALIDITY_INDICATOR, Value::enumeration(1)),
            (VALIDITY_INDICATOR, Value::enumeration(2)),
        ]),
        payload([
            (UNIQUE_IDENTIFIER, Value::text_string("key".to_owned())),
            (VALIDITY_INDICATOR, Value::integer(1)),
        ]),
        payload([
            (UNIQUE_IDENTIFIER, Value::text_string("key".to_owned())),
            (VALIDITY_INDICATOR, Value::enumeration(1)),
            (DATA, Value::byte_string(b"first".to_vec())),
            (DATA, Value::byte_string(b"second".to_vec())),
        ]),
        payload([
            (UNIQUE_IDENTIFIER, Value::text_string("key".to_owned())),
            (VALIDITY_INDICATOR, Value::enumeration(1)),
            (DATA, Value::integer(7)),
        ]),
    ];

    for response_payload in cases {
        let message = success(response_payload);
        assert_eq!(
            SignatureVerifyResponse::try_from_response_item(response_item(&message))
                .expect_err("response indicator and recovered Data have exact table shapes")
                .kind(),
            CryptographicOperationErrorKind::MalformedPayload
        );
    }
}

#[test]
fn failure_response_preserves_the_server_result_without_success_fields() {
    let message = response_message(
        SIGNATURE_VERIFY_OPERATION,
        OPERATION_FAILED,
        Some(GENERAL_FAILURE),
        None,
        None,
    );
    let response = SignatureVerifyResponse::try_from_response_item(response_item(&message))
        .expect("operation failure is a server result and has no success payload");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response
            .result()
            .reason()
            .map(super::result::ResultReason::raw),
        Some(GENERAL_FAILURE)
    );
    assert!(response.unique_identifier().is_none());
    assert!(response.validity_indicator().is_none());
    assert!(response.recovered_data().is_none());
}

#[test]
fn request_and_response_debug_redact_signature_and_recovered_data() {
    let sentinel = "KMIPKIT_VERIFY_SECRET_SENTINEL";
    let request = SignatureVerifyRequest::new()
        .with_signature_data(SecretBytes::new(sentinel.as_bytes().to_vec()));
    assert!(!format!("{request:?}").contains(sentinel));

    let message = success(payload([
        (UNIQUE_IDENTIFIER, Value::text_string("key".to_owned())),
        (VALIDITY_INDICATOR, Value::enumeration(1)),
        (DATA, Value::byte_string(sentinel.as_bytes().to_vec())),
    ]));
    let response = SignatureVerifyResponse::try_from_response_item(response_item(&message))
        .expect("the sentinel response follows Table 338");
    assert!(!format!("{response:?}").contains(sentinel));
}

#[test]
fn final_multipart_response_accepts_both_discrepant_validity_indicator_forms() {
    for indicator in [None, Some(1)] {
        let mut fields = vec![(UNIQUE_IDENTIFIER, Value::text_string("key".to_owned()))];
        if let Some(raw) = indicator {
            fields.push((VALIDITY_INDICATOR, Value::enumeration(raw)));
        }
        let message = success(payload(fields));
        let response = SignatureVerifyResponse::try_from_response_item_with_context(
            response_item(&message),
            VerificationResponseContext::MultipartFinal,
        )
        .expect("KMIPKIT-DISC-048 leaves both final multipart forms accepted");
        assert_eq!(
            response.validity_indicator().map(ValidityIndicator::raw),
            indicator
        );
    }
}

#[test]
fn non_final_multipart_response_rejects_validity_indicator() {
    let message = success(payload([
        (UNIQUE_IDENTIFIER, Value::text_string("key".to_owned())),
        (VALIDITY_INDICATOR, Value::enumeration(1)),
        (CORRELATION_VALUE, Value::byte_string(b"next-part".to_vec())),
    ]));
    assert_eq!(
        SignatureVerifyResponse::try_from_response_item_with_context(
            response_item(&message),
            VerificationResponseContext::MultipartNonFinal,
        )
        .expect_err("both sources forbid Validity Indicator on a non-final part")
        .kind(),
        CryptographicOperationErrorKind::MalformedPayload
    );
}
