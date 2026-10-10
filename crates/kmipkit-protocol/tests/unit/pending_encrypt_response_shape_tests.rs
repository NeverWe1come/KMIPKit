//! Pending Encrypt and Decrypt response items must stay out of completed types.
//!
//! OASIS KMIP v2.1 §§8.6–8.8 (including Table 399) place the asynchronous
//! correlation value in the response batch item. The multipart Correlation
//! Value remains in the operation Response Payload under §7.8.
//!
//! Traceability: `KMIPKIT-REQ-SPEC-6.1-001-002`,
//! `KMIPKIT-ELEM-MESSAGE-FIELD-8-6-ASYNCHRONOUS-CORRELATION-VALUE`,
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE`, and
//! `KMIPKIT-ELEM-OP-C2S-ENCRYPT`, and `KMIPKIT-ELEM-OP-C2S-DECRYPT`.

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{
    DecryptResponse, EncryptResponse, ResponseBatchItemView, ResponseMessage, ResultStatus,
};
use kmipkit_ttlv::{Value, ValueView};

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const PENDING: u32 = 2;
const CORRELATION_VALUE: u32 = 0x0042_00D6;

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("the response message contains one operation batch item")
}

fn payload_correlation_value(item: ResponseBatchItemView<'_>) -> Option<Vec<u8>> {
    item.with_response_payload(|payload| {
        payload
            .children()
            .iter()
            .find(|child| child.tag().raw() == CORRELATION_VALUE)
            .and_then(|child| {
                child.with_value(|value| match value {
                    ValueView::ByteString(bytes) => Some(bytes.to_vec()),
                    _ => None,
                })
            })
    })
    .flatten()
}

#[test]
fn pending_encrypt_keeps_async_and_multipart_correlation_values_distinct() {
    let asynchronous_correlation = [0xA1, 0xB2, 0xC3, 0xD4];
    let multipart_correlation = [0x11, 0x22, 0x33];
    let payload = structure([item(
        CORRELATION_VALUE,
        Value::byte_string(multipart_correlation.to_vec()),
    )]);
    let message = response_message(
        ENCRYPT_OPERATION,
        PENDING,
        None,
        Some(&asynchronous_correlation),
        Some(payload),
    );
    let item = response_item(&message);

    assert_eq!(item.result_status(), Some(ResultStatus::from_raw(PENDING)));
    assert_eq!(
        item.with_asynchronous_correlation_value(<[u8]>::to_vec),
        Some(asynchronous_correlation.to_vec())
    );
    assert_eq!(
        payload_correlation_value(item),
        Some(multipart_correlation.to_vec())
    );
    assert_ne!(
        asynchronous_correlation.as_slice(),
        multipart_correlation.as_slice()
    );
}

#[test]
fn pending_encrypt_item_is_not_accepted_as_a_completed_response() {
    let asynchronous_correlation = [0xA1, 0xB2, 0xC3, 0xD4];
    let multipart_payload = structure([item(
        CORRELATION_VALUE,
        Value::byte_string(vec![0x11, 0x22, 0x33]),
    )]);
    let message = response_message(
        ENCRYPT_OPERATION,
        PENDING,
        None,
        Some(&asynchronous_correlation),
        Some(multipart_payload),
    );
    let item = response_item(&message);

    assert_eq!(item.result_status(), Some(ResultStatus::from_raw(PENDING)));
    assert_eq!(
        item.with_asynchronous_correlation_value(<[u8]>::to_vec),
        Some(asynchronous_correlation.to_vec())
    );
    assert!(
        EncryptResponse::try_from_response_item(item).is_err(),
        "Pending must be routed through the shared PendingOutcome path"
    );
}

#[test]
fn pending_decrypt_item_is_not_accepted_as_a_completed_response() {
    let asynchronous_correlation = [0xD4, 0xC3, 0xB2, 0xA1];
    let multipart_payload = structure([item(
        CORRELATION_VALUE,
        Value::byte_string(vec![0x33, 0x22, 0x11]),
    )]);
    let message = response_message(
        DECRYPT_OPERATION,
        PENDING,
        None,
        Some(&asynchronous_correlation),
        Some(multipart_payload),
    );
    let item = response_item(&message);

    assert_eq!(item.result_status(), Some(ResultStatus::from_raw(PENDING)));
    assert_eq!(
        item.with_asynchronous_correlation_value(<[u8]>::to_vec),
        Some(asynchronous_correlation.to_vec())
    );
    assert!(
        DecryptResponse::try_from_response_item(item).is_err(),
        "Pending must be routed through the shared PendingOutcome path"
    );
}
