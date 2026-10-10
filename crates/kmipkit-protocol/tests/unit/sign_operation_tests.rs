//! Source-derived Sign contracts from OASIS KMIP v2.1 §6.1.55, Tables 334–336.
//! Cryptographic Algorithm and Digital Signature Algorithm values follow
//! §§11.12/11.16, Tables 443/447. These tests do not claim an official pass.
//!
//! Traceability: FR-003, FR-004, FR-009–FR-015; `KMIPKIT-ELEM-OP-C2S-SIGN`,
//! `KMIPKIT-ELEM-ENUMERATION-CRYPTOGRAPHIC-ALGORITHM`,
//! `KMIPKIT-ELEM-ENUMERATION-DIGITAL-SIGNATURE-ALGORITHM`, and the shared Data,
//! Digested Data, Correlation Value, Init Indicator, and Final Indicator.

use crate::async_operation_fixtures::response_message;
use crate::operation_test_support::item;
use crate::{
    CryptographicAlgorithm, CryptographicOperationErrorKind, DigitalSignatureAlgorithm,
    OperationData, SecretBytes, SignRequest, SignResponse, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, Structure, Value, ValueView};

const SIGN_OPERATION: u32 = 0x0000_0021;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const GENERAL_FAILURE: u32 = 0x0000_0100;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_002A;
const DIGITAL_SIGNATURE_ALGORITHM: u32 = 0x0042_002C;
const DATA: u32 = 0x0042_00C2;
const DIGESTED_DATA: u32 = 0x0042_0107;
const SIGNATURE_DATA: u32 = 0x0042_00C7;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const INIT_INDICATOR: u32 = 0x0042_00D7;
const FINAL_INDICATOR: u32 = 0x0042_00D8;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

fn payload(items: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    let mut payload = Structure::new();
    for (raw_tag, value) in items {
        payload
            .try_push(item(raw_tag, value))
            .expect("Table 334–335 fixture fits TTLV Structure limits");
    }
    payload
}

fn field_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

fn response_item(message: &crate::ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("the fixture contains one validated Sign response item")
}

fn response_payload_tags(item: crate::ResponseBatchItemView<'_>) -> Option<Vec<u32>> {
    item.with_response_payload(|payload| {
        payload
            .children()
            .iter()
            .map(|field| field.tag().raw())
            .collect()
    })
}

fn output_bytes(value: Option<&SecretBytes>, expected: &[u8]) {
    value
        .expect("Table 335 Signature Data is present for a single-part response")
        .with_bytes(|actual| assert_eq!(actual, expected));
}

fn snapshot(item: &Item) -> (u32, Vec<u8>) {
    (
        item.tag().raw(),
        item.with_value(|value| match value {
            ValueView::ByteString(value) => Some(value.to_vec()),
            _ => None,
        })
        .expect("the selected field is a Byte String"),
    )
}

#[test]
fn request_emits_optional_identifier_parameters_inputs_and_framing_in_order() {
    let data = [0x00, 0x7F, 0x80, 0xFF];
    let digest = [0xA0, 0xB1, 0xFE];
    let request = SignRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("signing-key".to_owned()))
        .with_cryptographic_parameters(payload([
            (
                CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(CryptographicAlgorithm::from_raw(0x38).raw()),
            ),
            (
                DIGITAL_SIGNATURE_ALGORITHM,
                Value::enumeration(DigitalSignatureAlgorithm::from_raw(0x13).raw()),
            ),
        ]))
        .with_data(OperationData::ByteString(SecretBytes::new(data.to_vec())))
        .with_digested_data(SecretBytes::new(digest.to_vec()))
        .with_init_indicator(true)
        .with_final_indicator(true);

    let actual = request
        .to_ttlv_payload()
        .expect("Table 334 supports optional input fields and explicit framing");

    assert_eq!(
        field_tags(&actual),
        [
            UNIQUE_IDENTIFIER,
            CRYPTOGRAPHIC_PARAMETERS,
            DATA,
            DIGESTED_DATA,
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
        (DIGESTED_DATA, digest.to_vec())
    );
}

#[test]
fn request_allows_optional_identifier_and_parameters_to_be_omitted() {
    let actual = SignRequest::new()
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"message".to_vec(),
        )))
        .to_ttlv_payload()
        .expect("Unique Identifier and Cryptographic Parameters are optional in Table 334");

    assert_eq!(field_tags(&actual), [DATA]);
}

#[test]
fn request_requires_data_or_digested_data_for_single_part_and_accepts_digest_only() {
    assert!(
        SignRequest::new().to_ttlv_payload().is_err(),
        "Table 334 requires Data unless Digested Data is supplied"
    );

    let digest = [0x00, 0x81, 0xFF];
    let actual = SignRequest::new()
        .with_digested_data(SecretBytes::new(digest.to_vec()))
        .to_ttlv_payload()
        .expect("single-part Sign may supply Digested Data without Data");
    assert_eq!(field_tags(&actual), [DIGESTED_DATA]);
    assert_eq!(
        snapshot(&actual.view().children()[0]),
        (DIGESTED_DATA, digest.to_vec())
    );
}

#[test]
fn request_omits_data_for_explicit_multipart_calls() {
    let actual = SignRequest::new()
        .with_correlation_value(SecretBytes::new(b"multipart-id".to_vec()))
        .with_init_indicator(false)
        .with_final_indicator(false)
        .to_ttlv_payload()
        .expect("Table 334 omits Data on caller-controlled multipart requests");

    assert_eq!(
        field_tags(&actual),
        [CORRELATION_VALUE, INIT_INDICATOR, FINAL_INDICATOR]
    );
}

#[test]
fn successful_response_exposes_identifier_signature_data_and_unknown_fields() {
    let signature = [0x00, 0x80, 0xFF];
    let response_payload = payload([
        (
            UNIQUE_IDENTIFIER,
            Value::text_string("signing-key".to_owned()),
        ),
        (SIGNATURE_DATA, Value::byte_string(signature.to_vec())),
        (UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ]);
    let message = response_message(SIGN_OPERATION, SUCCESS, None, None, Some(response_payload));
    let item = response_item(&message);
    let response = SignResponse::try_from_response_item(item)
        .expect("Table 335 fields form a successful typed response");

    assert_eq!(response.result().status().raw(), SUCCESS);
    assert!(response.unique_identifier().is_some());
    output_bytes(response.signature_data(), &signature);
    assert_eq!(
        response_payload_tags(item),
        Some(vec![UNIQUE_IDENTIFIER, SIGNATURE_DATA, UNKNOWN_VENDOR_TAG]),
        "unknown fields remain generically inspectable in source order"
    );
}

#[test]
fn successful_response_requires_one_well_typed_unique_identifier() {
    let cases = [
        payload([(SIGNATURE_DATA, Value::byte_string(b"signature".to_vec()))]),
        payload([
            (UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            (UNIQUE_IDENTIFIER, Value::text_string("second".to_owned())),
            (SIGNATURE_DATA, Value::byte_string(b"signature".to_vec())),
        ]),
        payload([
            (UNIQUE_IDENTIFIER, Value::boolean(true)),
            (SIGNATURE_DATA, Value::byte_string(b"signature".to_vec())),
        ]),
    ];

    for response_payload in cases {
        let message = response_message(SIGN_OPERATION, SUCCESS, None, None, Some(response_payload));
        let result = SignResponse::try_from_response_item(response_item(&message));
        assert_eq!(
            result
                .expect_err("missing, repeated, and malformed identifiers are rejected")
                .kind(),
            CryptographicOperationErrorKind::MalformedPayload
        );
    }
}

#[test]
fn failure_response_preserves_server_result_without_success_payload() {
    let message = response_message(
        SIGN_OPERATION,
        OPERATION_FAILED,
        Some(GENERAL_FAILURE),
        None,
        None,
    );
    let response = SignResponse::try_from_response_item(response_item(&message))
        .expect("server Operation Failed remains a typed operation result");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(|reason| reason.raw()),
        Some(GENERAL_FAILURE)
    );
    assert!(response.unique_identifier().is_none());
    assert!(response.signature_data().is_none());
}

#[test]
fn successful_response_rejects_duplicate_or_malformed_signature_data() {
    let cases = [
        payload([
            (
                UNIQUE_IDENTIFIER,
                Value::text_string("signing-key".to_owned()),
            ),
            (SIGNATURE_DATA, Value::byte_string(b"first".to_vec())),
            (SIGNATURE_DATA, Value::byte_string(b"second".to_vec())),
        ]),
        payload([
            (
                UNIQUE_IDENTIFIER,
                Value::text_string("signing-key".to_owned()),
            ),
            (SIGNATURE_DATA, Value::integer(17)),
        ]),
    ];

    for response_payload in cases {
        let message = response_message(SIGN_OPERATION, SUCCESS, None, None, Some(response_payload));
        assert_eq!(
            SignResponse::try_from_response_item(response_item(&message))
                .expect_err("repeated or malformed Signature Data is rejected")
                .kind(),
            CryptographicOperationErrorKind::MalformedPayload
        );
    }
}

#[test]
fn cryptographic_and_signature_algorithm_values_preserve_standard_and_future_values() {
    for raw in 1..=0x38 {
        assert_eq!(CryptographicAlgorithm::from_raw(raw).raw(), raw);
    }
    for raw in 1..=0x13 {
        assert_eq!(DigitalSignatureAlgorithm::from_raw(raw).raw(), raw);
    }
    for raw in [0, 0x8000_0000, 0x8FFF_FFFF, 0x1234_5678, u32::MAX] {
        assert_eq!(CryptographicAlgorithm::from_raw(raw).raw(), raw);
        assert_eq!(DigitalSignatureAlgorithm::from_raw(raw).raw(), raw);
    }
}

#[test]
fn request_and_response_debug_redact_data_and_signature_bytes() {
    let sentinel = "KMIPKIT_SIGN_SECRET_SENTINEL";
    let request =
        SignRequest::new().with_digested_data(SecretBytes::new(sentinel.as_bytes().to_vec()));
    assert!(!format!("{request:?}").contains(sentinel));

    let message = response_message(
        SIGN_OPERATION,
        SUCCESS,
        None,
        None,
        Some(payload([
            (
                UNIQUE_IDENTIFIER,
                Value::text_string("signing-key".to_owned()),
            ),
            (
                SIGNATURE_DATA,
                Value::byte_string(sentinel.as_bytes().to_vec()),
            ),
        ])),
    );
    let response = SignResponse::try_from_response_item(response_item(&message))
        .expect("the redaction fixture is a valid Table 335 response");
    assert!(!format!("{response:?}").contains(sentinel));
}
