//! Shared operation-result parsing from OASIS KMIP v2.1 §§8.6 and 9.17–9.18.
//!
//! Result Status and Result Reason values are defined by §§11.44 and 11.46.
//! Unknown reasons and Result Message text remain lossless, while Pending stays
//! represented as its own status for the caller's shared `PendingOutcome` path.
//!
//! Traceability: `KMIPKIT-0019-FR-007` and `KMIPKIT-REQ-SPEC-6.1-001-002`.

use crate::async_operation_fixtures::{item, structure};
use crate::asynchronous::operation_result;
use crate::{
    DecryptResponse, EncryptResponse, KmipOperationResult, ResponseBatchItemView, ResponseMessage,
    ResultMessage, ResultReason, ResultStatus, ResultValidationError,
};
use kmipkit_ttlv::Value;

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const UNKNOWN_REASON: u32 = 0xF001_0001;

const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_STATUS: u32 = 0x0042_007F;

fn response_message(
    operation: u32,
    status: Option<u32>,
    reason: Option<u32>,
    result_message: Option<&str>,
    asynchronous_correlation_value: Option<&[u8]>,
) -> ResponseMessage {
    let version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(version)),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut batch_items = vec![item(OPERATION, Value::enumeration(operation))];
    if let Some(status) = status {
        batch_items.push(item(RESULT_STATUS, Value::enumeration(status)));
    }
    if let Some(reason) = reason {
        batch_items.push(item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(result_message) = result_message {
        batch_items.push(item(
            RESULT_MESSAGE,
            Value::text_string(result_message.to_owned()),
        ));
    }
    if let Some(correlation_value) = asynchronous_correlation_value {
        batch_items.push(item(
            0x0042_0006,
            Value::byte_string(correlation_value.to_vec()),
        ));
    }
    if status == Some(OPERATION_PENDING) {
        batch_items.push(item(
            RESPONSE_PAYLOAD,
            Value::structure(structure([item(
                0x0042_00D6,
                Value::byte_string(vec![0x11, 0x22, 0x33]),
            )])),
        ));
    }
    let response = structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(structure(batch_items))),
    ]);

    ResponseMessage::try_from_ttlv(response)
        .expect("the fixture is a structurally valid KMIP response message")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("the fixture contains one response batch item")
}

#[test]
fn shared_helper_and_encrypt_decrypt_preserve_unknown_reason_and_result_message() {
    let message_text = "server returned an unrecognized reason";
    for operation in [ENCRYPT_OPERATION, DECRYPT_OPERATION] {
        let message = response_message(
            operation,
            Some(OPERATION_FAILED),
            Some(UNKNOWN_REASON),
            Some(message_text),
            None,
        );
        let item = response_item(&message);
        let result =
            operation_result(item).expect("the shared result helper preserves server data");

        assert_eq!(result.status(), ResultStatus::from_raw(OPERATION_FAILED));
        assert_eq!(
            result.reason(),
            Some(ResultReason::from_raw(UNKNOWN_REASON))
        );
        assert_eq!(result.reason().and_then(ResultReason::known_name), None);
        assert_eq!(
            result.message().map(ResultMessage::as_str),
            Some(message_text)
        );

        let operation_result = if operation == ENCRYPT_OPERATION {
            EncryptResponse::try_from_response_item(item)
                .expect("completed Encrypt failures retain shared result fields")
                .result()
                .clone()
        } else {
            DecryptResponse::try_from_response_item(item)
                .expect("completed Decrypt failures retain shared result fields")
                .result()
                .clone()
        };
        assert_eq!(operation_result, result);
    }
}

#[test]
fn shared_helper_preserves_pending_and_completed_converters_reject_it() {
    for operation in [ENCRYPT_OPERATION, DECRYPT_OPERATION] {
        let message = response_message(
            operation,
            Some(OPERATION_PENDING),
            None,
            None,
            Some(&[0xA1, 0xB2, 0xC3]),
        );
        let item = response_item(&message);
        let result = operation_result(item).expect("Pending remains represented by its raw status");

        assert_eq!(result.status(), ResultStatus::from_raw(OPERATION_PENDING));
        assert_eq!(result.reason(), None);
        assert_eq!(result.message(), None);

        if operation == ENCRYPT_OPERATION {
            assert!(matches!(
                EncryptResponse::try_from_response_item(item),
                Err(crate::EncryptError::PendingOutcomeRequired)
            ));
        } else {
            assert!(matches!(
                DecryptResponse::try_from_response_item(item),
                Err(crate::DecryptError::PendingOutcomeRequired)
            ));
        }
    }
}

#[test]
fn common_result_model_preserves_failure_reason_invariant() {
    let error = KmipOperationResult::new(ResultStatus::from_raw(OPERATION_FAILED), None, None)
        .expect_err("a Failure result requires a Result Reason");

    assert_eq!(error, ResultValidationError::FailureRequiresReason);
}
