//! Source-derived Hash request and response contracts from OASIS KMIP v2.1
//! §6.1.24, Tables 235–237; Hashing Algorithm values are from §11.21, Table
//! 454. These tests do not claim an official Test Case pass.
//!
//! Traceability: FR-002, FR-003, FR-009, FR-010, and FR-011;
//! `KMIPKIT-ELEM-OP-C2S-HASH`, `KMIPKIT-ELEM-ENUM-VALUE-OPERATION-HASH-00000027`,
//! `KMIPKIT-ELEM-ENUMERATION-HASHING-ALGORITHM`, and
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA`.

use crate::async_operation_fixtures::response_message;
use crate::operation_test_support::item;
use crate::{
    CryptographicOperationErrorKind, HashRequest, HashResponse, HashingAlgorithm, OperationData,
    SecretBytes,
};
use kmipkit_ttlv::{Item, Structure, Value, ValueView};

const HASH_OPERATION: u32 = 0x0000_0027;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const GENERAL_FAILURE: u32 = 0x0000_0100;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const DATA: u32 = 0x0042_00C2;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const INIT_INDICATOR: u32 = 0x0042_00D7;
const FINAL_INDICATOR: u32 = 0x0042_00D8;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

fn payload(items: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    let mut payload = Structure::new();
    for (raw_tag, value) in items {
        payload
            .try_push(item(raw_tag, value))
            .expect("Table 235–236 fixture fits TTLV Structure limits");
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

fn parameters(raw_algorithm: u32) -> Structure {
    payload([(HASHING_ALGORITHM, Value::enumeration(raw_algorithm))])
}

fn response_item(message: &crate::ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("the fixture contains one validated Hash response item")
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

fn secret_bytes(value: Option<&SecretBytes>, expected: &[u8]) {
    value
        .expect("the Table 236 optional byte string is present")
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
fn request_emits_table_235_fields_in_order_and_preserves_caller_values() {
    let input = [0x00, 0x7F, 0x80, 0xFF];
    let actual = HashRequest::new(parameters(6))
        .with_data(OperationData::ByteString(SecretBytes::new(input.to_vec())))
        .with_init_indicator(true)
        .with_final_indicator(true)
        .to_ttlv_payload()
        .expect("Table 235 fields form a valid single-part request");

    let view = actual.view();
    let fields = view.children();
    assert_eq!(
        field_tags(&actual),
        [0x0042_002B, DATA, INIT_INDICATOR, FINAL_INDICATOR,],
        "present Table 235 fields retain their normative order"
    );
    let parameter_values = fields[0].with_value(|value| match value {
        ValueView::Structure(parameters) => Some(
            parameters
                .children()
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert_eq!(parameter_values, Some(vec![HASHING_ALGORITHM]));
    assert_eq!(
        fields[0].with_value(|value| match value {
            ValueView::Structure(parameters) =>
                parameters.children()[0].with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                }),
            _ => None,
        }),
        Some(6),
        "the supplied SHA-256 value is forwarded unchanged"
    );
    assert_eq!(snapshot(&fields[1]), (DATA, input.to_vec()));
    assert_eq!(
        fields[2].with_value(|value| matches!(value, ValueView::Boolean(true))),
        true
    );
    assert_eq!(
        fields[3].with_value(|value| matches!(value, ValueView::Boolean(true))),
        true
    );
}

#[test]
fn request_requires_a_hashing_algorithm_in_cryptographic_parameters() {
    let cases = [
        Structure::new(),
        payload([(UNKNOWN_VENDOR_TAG, Value::integer(4))]),
        payload([(HASHING_ALGORITHM, Value::integer(6))]),
    ];

    for parameters in cases {
        assert!(
            HashRequest::new(parameters)
                .with_data(OperationData::ByteString(SecretBytes::new(
                    b"input".to_vec()
                )))
                .to_ttlv_payload()
                .is_err(),
            "a missing or malformed Hashing Algorithm is locally rejectable"
        );
    }
}

#[test]
fn request_requires_single_part_data_and_omits_it_for_multipart() {
    assert!(
        HashRequest::new(parameters(6)).to_ttlv_payload().is_err(),
        "Table 235 requires Data for a single-part request"
    );

    let multipart = HashRequest::new(parameters(6))
        .with_correlation_value(SecretBytes::new(b"stream-id".to_vec()))
        .with_init_indicator(false)
        .with_final_indicator(false)
        .to_ttlv_payload()
        .expect("Table 235 omits Data on caller-controlled multipart requests");
    assert_eq!(
        field_tags(&multipart),
        [
            0x0042_002B,
            CORRELATION_VALUE,
            INIT_INDICATOR,
            FINAL_INDICATOR
        ]
    );
    let multipart_view = multipart.view();
    assert_eq!(
        snapshot(&multipart_view.children()[1]),
        (CORRELATION_VALUE, b"stream-id".to_vec())
    );
}

#[test]
fn request_rejects_data_when_multipart_framing_is_supplied() {
    let result = HashRequest::new(parameters(6))
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"part".to_vec(),
        )))
        .with_correlation_value(SecretBytes::new(b"stream-id".to_vec()))
        .with_init_indicator(false)
        .to_ttlv_payload();

    assert!(
        result.is_err(),
        "multipart progression is explicitly shaped"
    );
}

#[test]
fn successful_response_preserves_table_236_data_and_correlation_bytes() {
    let data = [0x00, 0x80, 0xFF];
    let correlation = [0x01, 0x02, 0xFE];
    let payload = payload([
        (DATA, Value::byte_string(data.to_vec())),
        (CORRELATION_VALUE, Value::byte_string(correlation.to_vec())),
        (UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ]);
    let message = response_message(HASH_OPERATION, SUCCESS, None, None, Some(payload));
    let item = response_item(&message);
    let response = HashResponse::try_from_response_item(item)
        .expect("Table 236 successful response fields remain available");

    assert_eq!(response.result().status().raw(), SUCCESS);
    secret_bytes(response.data(), &data);
    secret_bytes(response.correlation_value(), &correlation);
    assert_eq!(
        response_payload_tags(item),
        Some(vec![DATA, CORRELATION_VALUE, UNKNOWN_VENDOR_TAG]),
        "unrecognized fields and source ordering remain generically inspectable"
    );
}

#[test]
fn successful_multipart_response_preserves_absent_data() {
    let correlation = [0xA0, 0xB1];
    let message = response_message(
        HASH_OPERATION,
        SUCCESS,
        None,
        None,
        Some(payload([(
            CORRELATION_VALUE,
            Value::byte_string(correlation.to_vec()),
        )])),
    );
    let response = HashResponse::try_from_response_item(response_item(&message))
        .expect("Table 236 permits Data omission for multipart results");

    assert!(response.data().is_none());
    secret_bytes(response.correlation_value(), &correlation);
}

#[test]
fn non_success_response_preserves_common_result_without_success_fields() {
    let message = response_message(
        HASH_OPERATION,
        OPERATION_FAILED,
        Some(GENERAL_FAILURE),
        None,
        None,
    );
    let response = HashResponse::try_from_response_item(response_item(&message))
        .expect("a valid non-success result is not decoded as a success payload");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(|value| value.raw()),
        Some(GENERAL_FAILURE)
    );
    assert!(response.data().is_none());
    assert!(response.correlation_value().is_none());
}

#[test]
fn malformed_success_response_fields_are_rejected_without_echoing_values() {
    let cases = [
        payload([(DATA, Value::integer(i32::MIN))]),
        payload([(CORRELATION_VALUE, Value::boolean(true))]),
        payload([
            (DATA, Value::byte_string(vec![0xA5])),
            (DATA, Value::byte_string(vec![0x5A])),
        ]),
        payload([
            (CORRELATION_VALUE, Value::byte_string(vec![0xA5])),
            (CORRELATION_VALUE, Value::byte_string(vec![0x5A])),
        ]),
    ];

    for payload in cases {
        let message = response_message(HASH_OPERATION, SUCCESS, None, None, Some(payload));
        let Err(error) = HashResponse::try_from_response_item(response_item(&message)) else {
            panic!("malformed Table 236 fields are rejected");
        };
        assert_eq!(
            error.kind(),
            CryptographicOperationErrorKind::MalformedPayload
        );
        assert!(!error.to_string().contains("A5"));
        assert!(!error.to_string().contains("5A"));
    }
}

#[test]
fn hashing_algorithm_values_round_trip_standard_extensions_and_unknown_values() {
    // OASIS KMIP v2.1 §11.21, Table 454 assigns standard values 1 through 17.
    for raw in 1..=17 {
        assert_eq!(HashingAlgorithm::from_raw(raw).raw(), raw);
    }

    for raw in [0, 0x8000_0000, 0x8FFF_FFFF, u32::MAX] {
        assert_eq!(HashingAlgorithm::from_raw(raw).raw(), raw);
    }
}
