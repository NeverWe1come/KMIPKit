//! Source-derived MAC contracts from OASIS KMIP v2.1 §6.1.32, Tables 259–261.
//! Cryptographic Algorithm values follow §11.12, Table 443. These tests do not
//! claim an official Test Case pass.
//!
//! Traceability: FR-003, FR-009–FR-013, FR-015; `KMIPKIT-ELEM-OP-C2S-MAC`,
//! `KMIPKIT-ELEM-ENUMERATION-CRYPTOGRAPHIC-ALGORITHM`, and the shared Data,
//! Correlation Value, Init Indicator, and Final Indicator elements.

use crate::async_operation_fixtures::response_message;
use crate::cryptographic_operation_test_support::{
    field_tags, output_bytes, payload, response_item, response_payload_tags, snapshot,
};
use crate::{
    CryptographicAlgorithm, CryptographicOperationErrorKind, MacRequest, MacResponse,
    OperationData, SecretBytes, UniqueIdentifier,
};
use kmipkit_ttlv::Value;

const MAC_OPERATION: u32 = 0x0000_0023;
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
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

#[test]
fn request_emits_optional_identifier_and_parameters_before_input_and_framing() {
    let input = [0x00, 0x7F, 0x80, 0xFF];
    let request = MacRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("mac-key".to_owned()))
        .with_cryptographic_parameters(payload([(
            CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(CryptographicAlgorithm::from_raw(0x38).raw()),
        )]))
        .with_data(OperationData::ByteString(SecretBytes::new(input.to_vec())))
        .with_init_indicator(true)
        .with_final_indicator(true);

    let actual = request
        .to_ttlv_payload()
        .expect("Table 259 optional fields and single-part input are valid");

    assert_eq!(
        field_tags(&actual),
        [
            UNIQUE_IDENTIFIER,
            CRYPTOGRAPHIC_PARAMETERS,
            DATA,
            INIT_INDICATOR,
            FINAL_INDICATOR,
        ]
    );
    assert_eq!(
        snapshot(&actual.view().children()[2]),
        (DATA, input.to_vec())
    );
}

#[test]
fn request_allows_optional_identifier_and_parameters_to_be_omitted() {
    let actual = MacRequest::new()
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"input".to_vec(),
        )))
        .to_ttlv_payload()
        .expect("Unique Identifier and Cryptographic Parameters are optional in Table 259");

    assert_eq!(field_tags(&actual), [DATA]);
}

#[test]
fn request_requires_single_part_data_and_omits_data_for_multipart() {
    assert!(
        MacRequest::new().to_ttlv_payload().is_err(),
        "Table 259 requires Data when no multipart framing is supplied"
    );

    let multipart = MacRequest::new()
        .with_correlation_value(SecretBytes::new(b"multipart-id".to_vec()))
        .with_init_indicator(false)
        .with_final_indicator(false)
        .to_ttlv_payload()
        .expect("Table 259 omits Data for explicit multipart calls");
    assert_eq!(
        field_tags(&multipart),
        [CORRELATION_VALUE, INIT_INDICATOR, FINAL_INDICATOR]
    );
}

#[test]
fn successful_response_exposes_identifier_mac_data_and_unknown_generic_fields() {
    let mac = [0x00, 0x80, 0xFF];
    let response_payload = payload([
        (UNIQUE_IDENTIFIER, Value::text_string("mac-key".to_owned())),
        (MAC_DATA, Value::byte_string(mac.to_vec())),
        (UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ]);
    let message = response_message(MAC_OPERATION, SUCCESS, None, None, Some(response_payload));
    let item = response_item(&message);
    let response = MacResponse::try_from_response_item(item)
        .expect("Table 260 fields form a successful typed response");

    assert_eq!(response.result().status().raw(), SUCCESS);
    assert!(response.unique_identifier().is_some());
    output_bytes(response.mac_data(), &mac);
    assert_eq!(
        response_payload_tags(item),
        Some(vec![UNIQUE_IDENTIFIER, MAC_DATA, UNKNOWN_VENDOR_TAG]),
        "unrecognized fields and their source order remain generically inspectable"
    );
}

#[test]
fn successful_multipart_response_preserves_correlation_without_mac_data() {
    let correlation = [0x00, 0x81, 0xFF];
    let response_payload = payload([
        (UNIQUE_IDENTIFIER, Value::text_string("mac-key".to_owned())),
        (CORRELATION_VALUE, Value::byte_string(correlation.to_vec())),
    ]);
    let message = response_message(MAC_OPERATION, SUCCESS, None, None, Some(response_payload));
    let item = response_item(&message);
    let response = MacResponse::try_from_response_item(item)
        .expect("Table 260 permits multipart response framing without MAC Data");

    assert!(response.mac_data().is_none());
    response
        .correlation_value()
        .expect("the server returned the multipart correlation value")
        .with_bytes(|actual| assert_eq!(actual, correlation));
}

#[test]
fn successful_response_requires_one_well_typed_unique_identifier() {
    let cases = [
        payload([(MAC_DATA, Value::byte_string(b"mac".to_vec()))]),
        payload([
            (UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            (UNIQUE_IDENTIFIER, Value::text_string("second".to_owned())),
            (MAC_DATA, Value::byte_string(b"mac".to_vec())),
        ]),
        payload([
            (UNIQUE_IDENTIFIER, Value::boolean(true)),
            (MAC_DATA, Value::byte_string(b"mac".to_vec())),
        ]),
    ];

    for response_payload in cases {
        let message = response_message(MAC_OPERATION, SUCCESS, None, None, Some(response_payload));
        let result = MacResponse::try_from_response_item(response_item(&message));
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
        MAC_OPERATION,
        OPERATION_FAILED,
        Some(GENERAL_FAILURE),
        None,
        None,
    );
    let response = MacResponse::try_from_response_item(response_item(&message))
        .expect("server Operation Failed remains a typed operation result");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response
            .result()
            .reason()
            .map(super::result::ResultReason::raw),
        Some(GENERAL_FAILURE)
    );
    assert!(response.unique_identifier().is_none());
    assert!(response.mac_data().is_none());
}

#[test]
fn successful_response_rejects_duplicate_or_malformed_mac_data() {
    let cases = [
        payload([
            (UNIQUE_IDENTIFIER, Value::text_string("mac-key".to_owned())),
            (MAC_DATA, Value::byte_string(b"first".to_vec())),
            (MAC_DATA, Value::byte_string(b"second".to_vec())),
        ]),
        payload([
            (UNIQUE_IDENTIFIER, Value::text_string("mac-key".to_owned())),
            (MAC_DATA, Value::integer(17)),
        ]),
    ];

    for response_payload in cases {
        let message = response_message(MAC_OPERATION, SUCCESS, None, None, Some(response_payload));
        assert_eq!(
            MacResponse::try_from_response_item(response_item(&message))
                .expect_err("repeated or malformed MAC Data is rejected")
                .kind(),
            CryptographicOperationErrorKind::MalformedPayload
        );
    }
}

#[test]
fn successful_response_rejects_duplicate_or_malformed_correlation_value() {
    let cases = [
        payload([
            (UNIQUE_IDENTIFIER, Value::text_string("mac-key".to_owned())),
            (CORRELATION_VALUE, Value::byte_string(b"first".to_vec())),
            (CORRELATION_VALUE, Value::byte_string(b"second".to_vec())),
        ]),
        payload([
            (UNIQUE_IDENTIFIER, Value::text_string("mac-key".to_owned())),
            (CORRELATION_VALUE, Value::integer(17)),
        ]),
    ];

    for response_payload in cases {
        let message = response_message(MAC_OPERATION, SUCCESS, None, None, Some(response_payload));
        assert_eq!(
            MacResponse::try_from_response_item(response_item(&message))
                .expect_err("repeated or malformed Correlation Value is rejected")
                .kind(),
            CryptographicOperationErrorKind::MalformedPayload
        );
    }
}

#[test]
fn cryptographic_algorithm_preserves_every_standard_extension_and_future_value() {
    for raw in 1..=0x38 {
        assert_eq!(CryptographicAlgorithm::from_raw(raw).raw(), raw);
    }
    for raw in [0, 0x8000_0000, 0x8FFF_FFFF, 0x1234_5678, u32::MAX] {
        assert_eq!(CryptographicAlgorithm::from_raw(raw).raw(), raw);
    }
}

#[test]
fn request_and_response_debug_redact_mac_material() {
    let sentinel = "KMIPKIT_MAC_SECRET_SENTINEL";
    let request = MacRequest::new().with_data(OperationData::ByteString(SecretBytes::new(
        sentinel.as_bytes().to_vec(),
    )));
    assert!(!format!("{request:?}").contains(sentinel));

    let message = response_message(
        MAC_OPERATION,
        SUCCESS,
        None,
        None,
        Some(payload([
            (UNIQUE_IDENTIFIER, Value::text_string("mac-key".to_owned())),
            (MAC_DATA, Value::byte_string(sentinel.as_bytes().to_vec())),
        ])),
    );
    let response = MacResponse::try_from_response_item(response_item(&message))
        .expect("the redaction fixture is a valid Table 260 response");
    assert!(!format!("{response:?}").contains(sentinel));
}
