//! Completed Encrypt and Decrypt failure response tests.
//!
//! OASIS KMIP v2.1 §6.1 makes each Operation's Result Reason list non-exhaustive:
//! any Result Reason in the Message Data Structures may be used. §9.18 defines
//! the shared field and §11.46 its values, so General Failure (0x00000100) is
//! valid for Encrypt and Decrypt even though Tables 198 and 216 do not list it.
//! Result Message is defined by §9.17.
//!
//! Traceability: `KMIPKIT-REQ-SPEC-6.1-001-002`,
//! `KMIPKIT-ELEM-OP-C2S-ENCRYPT`, `KMIPKIT-ELEM-OP-C2S-DECRYPT`, and
//! `KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-GENERAL-FAILURE-00000100`.

use crate::async_operation_fixtures::{item, structure};
use crate::{
    DecryptResponse, EncryptResponse, ResponseBatchItemView, ResponseMessage, ResultMessage,
    ResultReason, ResultStatus,
};
use kmipkit_ttlv::Value;

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const OPERATION_FAILED: u32 = 1;
const GENERAL_FAILURE: u32 = 0x0000_0100;
const OPERATION: u32 = 0x0042_005C;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_STATUS: u32 = 0x0042_007F;

fn failure_response_message(operation: u32, result_message: &str) -> ResponseMessage {
    let version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(version)),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch_item = structure([
        item(OPERATION, Value::enumeration(operation)),
        item(RESULT_STATUS, Value::enumeration(OPERATION_FAILED)),
        item(RESULT_REASON, Value::enumeration(GENERAL_FAILURE)),
        item(
            RESULT_MESSAGE,
            Value::text_string(result_message.to_owned()),
        ),
    ]);
    let response = structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(batch_item)),
    ]);

    ResponseMessage::try_from_ttlv(response)
        .expect("a completed Failure may carry common result fields without a payload")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("the response message contains one operation batch item")
}

#[test]
fn encrypt_failure_preserves_common_result_without_success_payload_or_uid() {
    let message_text = "Encrypt was rejected by the server";
    let message = failure_response_message(ENCRYPT_OPERATION, message_text);
    let item = response_item(&message);
    let response = EncryptResponse::try_from_response_item(item)
        .expect("a completed Encrypt failure does not require the success payload");

    assert_eq!(item.with_response_payload(|_| ()), None);
    assert_eq!(response.unique_identifier(), None);
    assert_eq!(
        response.result().status(),
        ResultStatus::from_raw(OPERATION_FAILED)
    );
    assert_eq!(
        response.result().reason(),
        Some(ResultReason::from_raw(GENERAL_FAILURE))
    );
    assert_eq!(
        response
            .result()
            .reason()
            .and_then(ResultReason::known_name),
        Some("General Failure")
    );
    assert_eq!(
        response.result().message().map(ResultMessage::as_str),
        Some(message_text)
    );
}

#[test]
fn decrypt_failure_preserves_common_result_without_success_payload_or_uid() {
    let message_text = "Decrypt was rejected by the server";
    let message = failure_response_message(DECRYPT_OPERATION, message_text);
    let item = response_item(&message);
    let response = DecryptResponse::try_from_response_item(item)
        .expect("a completed Decrypt failure does not require the success payload");

    assert_eq!(item.with_response_payload(|_| ()), None);
    assert_eq!(response.unique_identifier(), None);
    assert_eq!(
        response.result().status(),
        ResultStatus::from_raw(OPERATION_FAILED)
    );
    assert_eq!(
        response.result().reason(),
        Some(ResultReason::from_raw(GENERAL_FAILURE))
    );
    assert_eq!(
        response
            .result()
            .reason()
            .and_then(ResultReason::known_name),
        Some("General Failure")
    );
    assert_eq!(
        response.result().message().map(ResultMessage::as_str),
        Some(message_text)
    );
}
