//! Source-derived MAC Verify contracts from OASIS KMIP v2.1 §6.1.33,
//! Tables 262–264; Validity Indicator values follow §11.61. These tests are
//! not official Test Case executions.
//!
//! Traceability: FR-001, FR-005–FR-011, FR-013, FR-015, and
//! `KMIPKIT-ELEM-OP-C2S-MAC-VERIFY`.

use crate::async_operation_fixtures::response_message;
use crate::cryptographic_operation_test_support::{
    field_tags, payload, response_item, response_payload_tags, snapshot,
};
use crate::{
    CryptographicAlgorithm, CryptographicOperationErrorKind, MacVerifyRequest, MacVerifyResponse,
    OperationData, SecretBytes, UniqueIdentifier, ValidityIndicator, VerificationResponseContext,
};
use kmipkit_ttlv::Value;

const MAC_VERIFY_OPERATION: u32 = 0x0000_0024;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const GENERAL_FAILURE: u32 = 0x0000_0100;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const DATA: u32 = 0x0042_00C2;
const MAC_DATA: u32 = 0x0042_00C4;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const INIT_INDICATOR: u32 = 0x0042_00D7;
const FINAL_INDICATOR: u32 = 0x0042_00D8;
const VALIDITY_INDICATOR: u32 = 0x0042_0128;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

fn success(payload: kmipkit_ttlv::Structure) -> crate::ResponseMessage {
    response_message(MAC_VERIFY_OPERATION, SUCCESS, None, None, Some(payload))
}

#[test]
fn request_emits_optional_identifier_parameters_original_data_mac_and_framing_in_order() {
    let data = [0x00, 0x7F, 0x80, 0xFF];
    let mac = [0xA0, 0xB1, 0xFE];
    let mut parameters = kmipkit_ttlv::Structure::new();
    parameters
        .try_push(crate::operation_test_support::item(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(CryptographicAlgorithm::from_raw(0x38).raw()),
        ))
        .expect("the table fixture contains one algorithm parameter");
    let request = MacVerifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("verification-key".to_owned()))
        .with_cryptographic_parameters(parameters)
        .with_data(OperationData::ByteString(SecretBytes::new(data.to_vec())))
        .with_mac_data(SecretBytes::new(mac.to_vec()))
        .with_init_indicator(true)
        .with_final_indicator(true);

    let actual = request
        .to_ttlv_payload()
        .expect("Table 262 allows the optional key, parameters, original Data, and framing");

    assert_eq!(
        field_tags(&actual),
        [
            UNIQUE_IDENTIFIER,
            CRYPTOGRAPHIC_PARAMETERS,
            DATA,
            MAC_DATA,
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
        (MAC_DATA, mac.to_vec())
    );
}

#[test]
fn request_allows_optional_key_parameters_and_original_data_to_be_omitted() {
    let mac = [0x00, 0x81, 0xFF];
    let actual = MacVerifyRequest::new()
        .with_mac_data(SecretBytes::new(mac.to_vec()))
        .to_ttlv_payload()
        .expect("Table 262 makes Unique Identifier, parameters, and original Data optional");

    assert_eq!(field_tags(&actual), [MAC_DATA]);
    assert_eq!(
        snapshot(&actual.view().children()[0]),
        (MAC_DATA, mac.to_vec())
    );
}

#[test]
fn request_requires_mac_data_for_single_part_and_omits_it_for_multipart() {
    assert!(
        MacVerifyRequest::new().to_ttlv_payload().is_err(),
        "Table 262 requires MAC Data for single-part MAC Verify"
    );

    let actual = MacVerifyRequest::new()
        .with_correlation_value(SecretBytes::new(b"mac-verify-stream".to_vec()))
        .with_init_indicator(false)
        .with_final_indicator(false)
        .to_ttlv_payload()
        .expect("Table 262 forbids MAC Data on a multipart request");
    assert_eq!(
        field_tags(&actual),
        [CORRELATION_VALUE, INIT_INDICATOR, FINAL_INDICATOR]
    );
}

#[test]
fn request_classifies_single_part_initial_non_final_and_final_multipart_contexts() {
    let single_part = MacVerifyRequest::new()
        .with_mac_data(SecretBytes::new(b"mac".to_vec()))
        .verification_response_context();
    let framed_single_part = MacVerifyRequest::new()
        .with_mac_data(SecretBytes::new(b"mac".to_vec()))
        .with_init_indicator(true)
        .with_final_indicator(true)
        .verification_response_context();
    let initial_non_final = MacVerifyRequest::new()
        .with_init_indicator(true)
        .with_final_indicator(false)
        .verification_response_context();
    let continuation_non_final = MacVerifyRequest::new()
        .with_correlation_value(SecretBytes::new(b"stream".to_vec()))
        .with_final_indicator(false)
        .verification_response_context();
    let final_part = MacVerifyRequest::new()
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
        MacVerifyResponse::try_from_response_item(response_item(&missing))
            .expect_err("single-part Table 263 requires Validity Indicator")
            .kind(),
        CryptographicOperationErrorKind::MalformedPayload
    );

    for raw in [1, 2, 3, 0x8000_0000, u32::MAX] {
        let message = success(payload([
            (UNIQUE_IDENTIFIER, Value::text_string("key".to_owned())),
            (VALIDITY_INDICATOR, Value::enumeration(raw)),
            (UNKNOWN_VENDOR_TAG, Value::integer(-7)),
        ]));
        let item = response_item(&message);
        let typed = MacVerifyResponse::try_from_response_item(item)
            .expect("single-part Validity Indicator values remain open");
        assert_eq!(
            typed.validity_indicator(),
            Some(ValidityIndicator::from_raw(raw))
        );
        assert_eq!(
            response_payload_tags(item),
            Some(vec![
                UNIQUE_IDENTIFIER,
                VALIDITY_INDICATOR,
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
            MacVerifyResponse::try_from_response_item(response_item(&message))
                .expect_err("missing, repeated, and malformed identifiers are rejected")
                .kind(),
            CryptographicOperationErrorKind::MalformedPayload
        );
    }
}

#[test]
fn successful_response_rejects_duplicate_or_malformed_validity_indicators() {
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
    ];

    for response_payload in cases {
        let message = success(response_payload);
        assert_eq!(
            MacVerifyResponse::try_from_response_item(response_item(&message))
                .expect_err("Validity Indicator has one Enumeration occurrence")
                .kind(),
            CryptographicOperationErrorKind::MalformedPayload
        );
    }
}

#[test]
fn failure_response_preserves_the_server_result_without_success_fields() {
    let message = response_message(
        MAC_VERIFY_OPERATION,
        OPERATION_FAILED,
        Some(GENERAL_FAILURE),
        None,
        None,
    );
    let response = MacVerifyResponse::try_from_response_item(response_item(&message))
        .expect("operation failure is a server result and has no success payload");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(|reason| reason.raw()),
        Some(GENERAL_FAILURE)
    );
    assert!(response.unique_identifier().is_none());
    assert!(response.validity_indicator().is_none());
}

#[test]
fn final_multipart_response_accepts_both_discrepant_validity_indicator_forms() {
    for indicator in [None, Some(1)] {
        let mut fields = vec![(UNIQUE_IDENTIFIER, Value::text_string("key".to_owned()))];
        if let Some(raw) = indicator {
            fields.push((VALIDITY_INDICATOR, Value::enumeration(raw)));
        }
        let message = success(payload(fields));
        let response = MacVerifyResponse::try_from_response_item_with_context(
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
        MacVerifyResponse::try_from_response_item_with_context(
            response_item(&message),
            VerificationResponseContext::MultipartNonFinal,
        )
        .expect_err("both sources forbid Validity Indicator on a non-final part")
        .kind(),
        CryptographicOperationErrorKind::MalformedPayload
    );
}
