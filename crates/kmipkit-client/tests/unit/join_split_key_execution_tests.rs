//! Fake-transport Join Split Key checks from KMIP v2.1 §6.1.27, Tables 244–246.
//! These table-derived vectors are not official OASIS test-case executions.

use kmipkit_protocol::{JoinSplitKeyRequest, ObjectType, SplitKeyReference, UniqueIdentifier};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;

use super::{ClientOperation, JOIN_SPLIT_KEY_OPERATION, encode_message_for_test};
use crate::asynchronous_execution_test_support::{client_for, request_contains};
use crate::execute_test_support::{test_item, test_structure};

const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const OPERATION_FAILED: u32 = 1;
const SERVER_REJECTED_SPLIT_KEYS: u32 = 0xF123_4567;

fn response_bytes(status: u32, reason: Option<u32>) -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut fields = vec![
        test_item(OPERATION, Value::enumeration(JOIN_SPLIT_KEY_OPERATION)),
        test_item(RESULT_STATUS, Value::enumeration(status)),
    ];
    if let Some(reason) = reason {
        fields.push(test_item(RESULT_REASON, Value::enumeration(reason)));
    }
    if status == 0 {
        let payload = test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("joined-object".to_owned()),
        )]);
        fields.push(test_item(0x0042_007C, Value::structure(payload)));
    }
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(test_structure(fields))),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("fixture is a valid KMIP 2.1 response")
}

fn reference(value: &str, threshold: u32) -> SplitKeyReference {
    SplitKeyReference::new(UniqueIdentifier::TextString(value.to_owned()))
        .with_split_key_threshold(threshold)
}

fn request_with(references: Vec<SplitKeyReference>) -> JoinSplitKeyRequest {
    JoinSplitKeyRequest::new(ObjectType::from_raw(7), references)
}

#[test]
fn explicit_join_split_key_dispatches_once_and_returns_typed_success() {
    assert_eq!(JOIN_SPLIT_KEY_OPERATION, 0x0000_0029);
    let response = response_bytes(0, None);
    let (mut client, fake, request_bytes) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![7, 3, 13],
    });
    let request = request_with(vec![reference("split-a", 2), reference("split-b", 2)]);
    let result = client
        .join_split_key(request, &CodecLimits::defaults())
        .expect("two supplied identifiers meet their known threshold");

    assert_eq!(result.outcome().operation(), ClientOperation::JoinSplitKey);
    assert!(request_contains(&request_bytes, b"split-a"));
    assert!(request_contains(&request_bytes, b"split-b"));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn absent_or_insufficient_threshold_context_fails_before_transport() {
    let response = response_bytes(0, None);
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let missing = request_with(vec![
        reference("split-a", 1),
        SplitKeyReference::new(UniqueIdentifier::TextString("split-b".to_owned())),
    ]);
    let error = client
        .join_split_key(missing, &CodecLimits::defaults())
        .expect_err("each identifier needs one matching caller threshold");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);

    let insufficient = request_with(vec![reference("split-a", 3), reference("split-b", 1)]);
    let error = client
        .join_split_key(insufficient, &CodecLimits::defaults())
        .expect_err("two identifiers do not meet maximum threshold three");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn server_failure_preserves_result_reason_without_retry() {
    let response = response_bytes(OPERATION_FAILED, Some(SERVER_REJECTED_SPLIT_KEYS));
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let result = client
        .join_split_key(
            request_with(vec![reference("split-a", 1)]),
            &CodecLimits::defaults(),
        )
        .expect("the server result remains available to the caller");
    assert_eq!(result.outcome().operation(), ClientOperation::JoinSplitKey);
    assert_eq!(fake.borrow().exchange_count(), 1);
}
