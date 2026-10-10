//! Successful Encrypt response payload tests from OASIS KMIP Specification
//! v2.1 §6.1.17, Table 215. Data is the optional Byte String in §7.9,
//! Correlation Value is defined by §7.8, and Authenticated Encryption Tag by
//! §7.4. The operation Enumeration value is defined by §11.36. These
//! table-derived vectors are not official Test Case passes.
//!
//! Traceability: `KMIPKIT-REQ-SPEC-6.1-001-002`,
//! `KMIPKIT-ELEM-OP-C2S-ENCRYPT`,
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-4-AUTHENTICATED-ENCRYPTION-TAG`,
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE`, and
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA`.

use crate::async_operation_fixtures::response_message;
use crate::operation_test_support::item;
use crate::{
    EncryptResponse, ResponseBatchItemView, ResponseMessage, ResultStatus, SecretBytes,
    UniqueIdentifier,
};
use kmipkit_ttlv::{Structure, Value};

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const PENDING: u32 = 2;
const GENERAL_FAILURE: u32 = 0x0000_0100;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const DATA_TAG: u32 = 0x0042_00C2;
const IV_COUNTER_NONCE_TAG: u32 = 0x0042_003D;
const CORRELATION_VALUE_TAG: u32 = 0x0042_00D6;
const AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA_TAG: u32 = 0x0042_00FE;
const AUTHENTICATED_ENCRYPTION_TAG: u32 = 0x0042_00FF;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

fn payload(items: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    let mut payload = Structure::new();
    for (raw_tag, value) in items {
        payload
            .try_push(item(raw_tag, value))
            .expect("the Table 215 test payload stays within the TTLV depth limit");
    }
    payload
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("the fixture contains one validated Encrypt response batch item")
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
        .expect("the supplied Table 215 optional byte string is present")
        .with_bytes(|actual| assert_eq!(actual, expected));
}

#[test]
fn successful_response_preserves_table_215_fields_and_their_exact_bytes() {
    let identifier = UniqueIdentifier::TextString("encrypt-response-key".to_owned());
    let data = [0x00, 0x7F, 0x80, 0xFF];
    let iv_counter_nonce = [0x11, 0x22, 0x33, 0x44];
    let correlation_value = [0x55, 0x66, 0x77];
    let authenticated_encryption_tag = [0x88, 0x99, 0xAA, 0xBB];
    let payload = payload([
        (
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("encrypt-response-key".to_owned()),
        ),
        (DATA_TAG, Value::byte_string(data.to_vec())),
        (
            IV_COUNTER_NONCE_TAG,
            Value::byte_string(iv_counter_nonce.to_vec()),
        ),
        (
            CORRELATION_VALUE_TAG,
            Value::byte_string(correlation_value.to_vec()),
        ),
        (
            AUTHENTICATED_ENCRYPTION_TAG,
            Value::byte_string(authenticated_encryption_tag.to_vec()),
        ),
    ]);
    let message = response_message(ENCRYPT_OPERATION, SUCCESS, None, None, Some(payload));
    let item = response_item(&message);
    let response = EncryptResponse::try_from_response_item(item)
        .expect("Table 215 successful response retains its required and optional fields");

    assert_eq!(response.result().status(), ResultStatus::from_raw(SUCCESS));
    assert_eq!(response.unique_identifier(), Some(&identifier));
    assert_eq!(
        response_payload_tags(item),
        Some(vec![
            UNIQUE_IDENTIFIER_TAG,
            DATA_TAG,
            IV_COUNTER_NONCE_TAG,
            CORRELATION_VALUE_TAG,
            AUTHENTICATED_ENCRYPTION_TAG,
        ]),
        "the retained generic payload keeps Table 215 field order"
    );
    assert_secret_bytes(response.data(), &data);
    assert_secret_bytes(response.iv_counter_nonce(), &iv_counter_nonce);
    assert_secret_bytes(response.correlation_value(), &correlation_value);
    assert_secret_bytes(
        response.authenticated_encryption_tag(),
        &authenticated_encryption_tag,
    );
}

#[test]
fn successful_response_accepts_uid_only_table_215_payload() {
    let identifier = UniqueIdentifier::TextString("encrypt-uid-only".to_owned());
    let payload = payload([(
        UNIQUE_IDENTIFIER_TAG,
        Value::text_string("encrypt-uid-only".to_owned()),
    )]);
    let message = response_message(ENCRYPT_OPERATION, SUCCESS, None, None, Some(payload));
    let item = response_item(&message);
    let response = EncryptResponse::try_from_response_item(item)
        .expect("Table 215 optional fields may all be omitted on success");

    assert_eq!(response.unique_identifier(), Some(&identifier));
    assert_eq!(
        response_payload_tags(item),
        Some(vec![UNIQUE_IDENTIFIER_TAG])
    );
    assert!(response.data().is_none());
    assert!(response.iv_counter_nonce().is_none());
    assert!(response.correlation_value().is_none());
    assert!(response.authenticated_encryption_tag().is_none());
}

#[test]
fn successful_response_preserves_non_text_uid_encodings_and_unknown_fields() {
    let cases = [
        (
            UniqueIdentifier::Enumeration(u32::MAX),
            Value::enumeration(u32::MAX),
        ),
        (
            UniqueIdentifier::Integer(i32::MIN),
            Value::integer(i32::MIN),
        ),
    ];

    for (identifier, wire_value) in cases {
        let payload = payload([
            (UNIQUE_IDENTIFIER_TAG, wire_value),
            (UNKNOWN_VENDOR_TAG, Value::integer(-17)),
        ]);
        let message = response_message(ENCRYPT_OPERATION, SUCCESS, None, None, Some(payload));
        let item = response_item(&message);
        let response = EncryptResponse::try_from_response_item(item)
            .expect("Encrypt preserves supported UID encodings and tolerates future fields");

        assert_eq!(response.unique_identifier(), Some(&identifier));
        assert_eq!(
            response_payload_tags(item),
            Some(vec![UNIQUE_IDENTIFIER_TAG, UNKNOWN_VENDOR_TAG])
        );
    }
}

#[test]
fn successful_response_rejects_malformed_required_and_known_fields() {
    let cases = [
        payload([]),
        payload([(UNIQUE_IDENTIFIER_TAG, Value::boolean(true))]),
        payload([
            (
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("first-key".to_owned()),
            ),
            (
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("second-key".to_owned()),
            ),
        ]),
        payload([
            (
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("encrypt-key".to_owned()),
            ),
            (DATA_TAG, Value::integer(7)),
        ]),
        payload([
            (
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("encrypt-key".to_owned()),
            ),
            (
                AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA_TAG,
                Value::byte_string(vec![0xA5]),
            ),
        ]),
    ];

    for payload in cases {
        let message = response_message(ENCRYPT_OPERATION, SUCCESS, None, None, Some(payload));
        let Err(error) = EncryptResponse::try_from_response_item(response_item(&message)) else {
            panic!("malformed successful Encrypt payload is rejected");
        };

        assert_eq!(error, crate::EncryptError::MalformedSuccessPayload);
        assert_eq!(
            error.to_string(),
            "successful Encrypt response payload is malformed"
        );
    }
}

#[test]
fn response_converters_reject_items_for_a_different_operation() {
    let completed = response_message(
        DECRYPT_OPERATION,
        OPERATION_FAILED,
        Some(GENERAL_FAILURE),
        None,
        None,
    );
    let Err(completed_error) = EncryptResponse::try_from_response_item(response_item(&completed))
    else {
        panic!("the Encrypt converter rejects a Decrypt response item");
    };
    assert_eq!(completed_error, crate::EncryptError::UnexpectedOperation);
    assert_eq!(completed_error.to_string(), "response item is not Encrypt");

    let pending = response_message(
        DECRYPT_OPERATION,
        PENDING,
        None,
        Some(&[0xA1, 0xB2]),
        Some(Structure::new()),
    );
    let Err(pending_error) =
        EncryptResponse::try_from_pending_response_item(response_item(&pending))
    else {
        panic!("the Encrypt pending converter rejects a Decrypt response item");
    };
    assert_eq!(pending_error, crate::EncryptError::UnexpectedOperation);
}
