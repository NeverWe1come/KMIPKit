//! Successful Decrypt response payload tests from OASIS KMIP Specification
//! v2.1 §6.1.11, Table 197. Data is the optional Byte String in §7.9, and
//! Correlation Value is defined by §7.8. The operation Enumeration value is
//! defined by §11.36. These table-derived vectors are not official Test Case
//! passes.
//!
//! Traceability: `KMIPKIT-REQ-SPEC-6.1-001-002`,
//! `KMIPKIT-ELEM-OP-C2S-DECRYPT`,
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE`, and
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA`.

use crate::async_operation_fixtures::response_message;
use crate::operation_test_support::item;
use crate::{
    DecryptResponse, ResponseBatchItemView, ResponseMessage, ResultStatus, SecretBytes,
    UniqueIdentifier,
};
use kmipkit_ttlv::{Structure, Value};

const DECRYPT_OPERATION: u32 = 0x0000_0020;
const SUCCESS: u32 = 0;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const DATA_TAG: u32 = 0x0042_00C2;
const CORRELATION_VALUE_TAG: u32 = 0x0042_00D6;

fn payload(items: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    let mut payload = Structure::new();
    for (raw_tag, value) in items {
        payload
            .try_push(item(raw_tag, value))
            .expect("the Table 197 test payload stays within the TTLV depth limit");
    }
    payload
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("the fixture contains one validated Decrypt response batch item")
}

fn response_payload_tags(item: ResponseBatchItemView<'_>) -> Option<Vec<u32>> {
    item.with_response_payload(|payload| {
        payload
            .children()
            .iter()
            .map(|child| child.tag().raw())
            .collect()
    })
}

fn assert_secret_bytes(actual: Option<&SecretBytes>, expected: &[u8]) {
    actual
        .expect("the supplied Table 197 optional byte string is present")
        .with_bytes(|actual| assert_eq!(actual, expected));
}

#[test]
fn successful_response_preserves_table_197_fields_and_their_exact_bytes() {
    let identifier = UniqueIdentifier::TextString("decrypt-response-key".to_owned());
    let data = [0x00, 0x7F, 0x80, 0xFF];
    let correlation_value = [0x11, 0x22, 0x33];
    let payload = payload([
        (
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("decrypt-response-key".to_owned()),
        ),
        (DATA_TAG, Value::byte_string(data.to_vec())),
        (
            CORRELATION_VALUE_TAG,
            Value::byte_string(correlation_value.to_vec()),
        ),
    ]);
    let message = response_message(DECRYPT_OPERATION, SUCCESS, None, None, Some(payload));
    let item = response_item(&message);
    let response = DecryptResponse::try_from_response_item(item)
        .expect("Table 197 successful response retains its required and optional fields");

    assert_eq!(response.result().status(), ResultStatus::from_raw(SUCCESS));
    assert_eq!(response.unique_identifier(), Some(&identifier));
    assert_eq!(
        response_payload_tags(item),
        Some(vec![UNIQUE_IDENTIFIER_TAG, DATA_TAG, CORRELATION_VALUE_TAG]),
        "the retained generic payload keeps Table 197 field order"
    );
    assert_secret_bytes(response.data(), &data);
    assert_secret_bytes(response.correlation_value(), &correlation_value);
}

#[test]
fn successful_response_accepts_uid_only_table_197_payload() {
    let identifier = UniqueIdentifier::TextString("decrypt-uid-only".to_owned());
    let payload = payload([(
        UNIQUE_IDENTIFIER_TAG,
        Value::text_string("decrypt-uid-only".to_owned()),
    )]);
    let message = response_message(DECRYPT_OPERATION, SUCCESS, None, None, Some(payload));
    let item = response_item(&message);
    let response = DecryptResponse::try_from_response_item(item)
        .expect("Table 197 optional fields may all be omitted on success");

    assert_eq!(response.unique_identifier(), Some(&identifier));
    assert_eq!(
        response_payload_tags(item),
        Some(vec![UNIQUE_IDENTIFIER_TAG])
    );
    assert!(response.data().is_none());
    assert!(response.correlation_value().is_none());
}
