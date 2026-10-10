//! Fake-transport Get execution contracts derived from OASIS KMIP v2.1
//! §6.1.19, Tables 220–222; §§8.6 and 9.1, Tables 399–400; §§9.2 and 11.3,
//! Tables 431–432; §§11.46–11.47, Tables 479–480; and ID Placeholder behavior
//! in §§6.1, 6.1.8/Table 187, and 9.8. Traceability: KMIPKIT-0017-FR-001,
//! FR-003, FR-010, FR-011, and SC-002. These are client execution contracts,
//! not claims of official OASIS case passes.

use kmipkit_protocol::{AttributeSet, CreateRequest, GetRequest, ObjectType};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::ValueView;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Structure, Value};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};
use crate::execute_test_support::{
    asynchronous_response_bytes, one_item_response_bytes, operation_batch_response_bytes,
    test_item, test_structure,
};

const CREATE_OPERATION: u32 = 0x0000_0001;
const GET_OPERATION: u32 = 0x0000_000A;
const RESPONSE_MESSAGE: &str = "GET_RESULT_MESSAGE_SENTINEL";
const OBJECT_NOT_FOUND: u32 = 0x0000_0037;
const PENDING_CORRELATION: &[u8] = b"GET_PENDING_CORRELATION_EXACT";

const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0043;

const CREATE_BATCH_ID: &[u8] = b"get-placeholder-create";
const GET_BATCH_ID: &[u8] = b"get-placeholder-read";

fn empty_create_request() -> CreateRequest {
    CreateRequest::new(ObjectType::from_raw(7), AttributeSet::new())
}

fn successful_get_payload(identifier: &str) -> Structure {
    test_structure([
        test_item(OBJECT_TYPE, Value::enumeration(2)),
        test_item(UNIQUE_IDENTIFIER, Value::text_string(identifier.to_owned())),
        test_item(
            SYMMETRIC_KEY,
            Value::structure(test_structure([test_item(
                KEY_BLOCK,
                Value::structure(test_structure([
                    test_item(KEY_FORMAT_TYPE, Value::enumeration(1)),
                    test_item(
                        KEY_VALUE,
                        Value::structure(test_structure([test_item(
                            KEY_MATERIAL,
                            Value::byte_string(b"opaque-get-response".to_vec()),
                        )])),
                    ),
                ])),
            )])),
        ),
    ])
}

fn successful_create_payload(identifier: &str) -> Structure {
    test_structure([
        test_item(OBJECT_TYPE, Value::enumeration(7)),
        test_item(UNIQUE_IDENTIFIER, Value::text_string(identifier.to_owned())),
    ])
}

fn request_item_shape(request: &[u8], index: usize) -> Option<(u32, bool)> {
    let document = decode_with_limits(request, &CodecLimits::defaults()).ok()?;
    document.with_value(|message| {
        let ValueView::Structure(message) = message else {
            return None;
        };
        let batch_item = message
            .children()
            .iter()
            .filter(|item| item.tag().raw() == BATCH_ITEM)
            .nth(index)?;
        batch_item.with_value(|batch_item| {
            let ValueView::Structure(batch_item) = batch_item else {
                return None;
            };
            let operation = batch_item
                .children()
                .iter()
                .find(|item| item.tag().raw() == OPERATION)?
                .with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                })?;
            let payload_has_identifier = batch_item
                .children()
                .iter()
                .find(|item| item.tag().raw() == REQUEST_PAYLOAD)?
                .with_value(|payload| match payload {
                    ValueView::Structure(payload) => payload
                        .children()
                        .iter()
                        .any(|item| item.tag().raw() == UNIQUE_IDENTIFIER),
                    _ => false,
                });
            Some((operation, payload_has_identifier))
        })
    })
}

#[test]
fn get_success_preserves_completed_object_data_in_one_exchange() {
    let response = one_item_response_bytes(
        GET_OPERATION,
        0,
        None,
        None,
        None,
        None,
        Some(successful_get_payload("server-assigned-get-id")),
    );
    let (mut client, fake, captured_request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![3, 7],
    });

    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::Get(GetRequest::new()))),
            &CodecLimits::defaults(),
        )
        .expect("the server returns a completed Get response");

    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(
        request_item_shape(
            captured_request
                .borrow()
                .as_deref()
                .expect("the request reached the fake transport"),
            0,
        ),
        Some((GET_OPERATION, false)),
        "an omitted request identifier remains omitted on the wire"
    );
    let outcome = response.get(0).expect("one result is returned").outcome();
    assert_eq!(outcome.operation(), ClientOperation::Get);
    assert_eq!(outcome.result().status().raw(), 0);
    let get_response = outcome
        .get_response()
        .expect("a completed Get exposes its typed response");
    assert_eq!(get_response.result().status().raw(), 0);
    assert_eq!(get_response.object_type().raw(), 2);
    assert_eq!(
        get_response.unique_identifier(),
        Some("server-assigned-get-id")
    );
    assert_eq!(
        get_response.object().map(|object| object.tag().raw()),
        Some(SYMMETRIC_KEY)
    );
}

#[test]
fn get_failure_preserves_result_status_reason_and_message_without_retry() {
    let response = one_item_response_bytes(
        GET_OPERATION,
        1,
        Some(OBJECT_NOT_FOUND),
        Some(RESPONSE_MESSAGE),
        None,
        None,
        None,
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::Get(GetRequest::new()))),
            &CodecLimits::defaults(),
        )
        .expect("the operation failure remains a typed Get result");
    let outcome = response.get(0).expect("one result is returned").outcome();
    let get_response = outcome
        .get_response()
        .expect("a completed failed Get retains its operation result");

    assert_eq!(outcome.operation(), ClientOperation::Get);
    assert_eq!(get_response.result().status().raw(), 1);
    assert_eq!(
        get_response.result().reason().map(|reason| reason.raw()),
        Some(OBJECT_NOT_FOUND)
    );
    assert_eq!(
        get_response
            .result()
            .message()
            .map(kmipkit_protocol::ResultMessage::as_str),
        Some(RESPONSE_MESSAGE)
    );
    assert!(get_response.object().is_none());
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn get_pending_preserves_correlation_and_typed_result_without_follow_up() {
    let response = asynchronous_response_bytes(
        GET_OPERATION,
        2,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Get(GetRequest::new())))
        .with_asynchronous_indicator(2);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the requested asynchronous mode permits Pending");
    let outcome = response.get(0).expect("one result is returned").outcome();
    let ClientBatchOutcome::Pending(pending) = outcome else {
        panic!("a valid Get Pending result remains an explicit Pending outcome");
    };

    assert_eq!(pending.operation(), ClientOperation::Get);
    assert_eq!(pending.result().status().raw(), 2);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert!(outcome.get_response().is_none());
    assert_eq!(
        outcome
            .response()
            .get()
            .map(|get| get.result().status().raw()),
        Some(2),
        "the unified response view preserves the Get Pending result"
    );
    assert_eq!(
        fake.borrow().exchange_count(),
        1,
        "Pending is not polled again"
    );
}

#[test]
fn get_transport_failures_preserve_delivery_state_without_retry() {
    for (script, expected_state) in [
        (
            ExchangeScript::FailBeforeWrite,
            RequestDeliveryState::NotSent,
        ),
        (
            ExchangeScript::FailAfterPartialWrite { written_bytes: 3 },
            RequestDeliveryState::PossiblySent,
        ),
        (
            ExchangeScript::FailAfterPartialRead {
                written_bytes: 3,
                response_bytes: vec![0x42, 0x00, 0x7A, 0x01],
            },
            RequestDeliveryState::ResponseStarted,
        ),
    ] {
        let (mut client, fake, _) = client_for(script);
        let error = client
            .execute(
                ClientBatch::new(ClientBatchItem::new(ClientRequest::Get(GetRequest::new()))),
                &CodecLimits::defaults(),
            )
            .expect_err("the scripted transport failure is returned to the caller");

        assert_eq!(error.delivery_state(), Some(expected_state));
        assert_eq!(
            fake.borrow().exchange_count(),
            1,
            "delivery {expected_state:?}"
        );
    }
}

#[test]
fn get_without_identifier_follows_create_and_keeps_the_batch_order() {
    let response = operation_batch_response_bytes([
        (
            CREATE_OPERATION,
            CREATE_BATCH_ID.to_vec(),
            successful_create_payload("server-created-id"),
        ),
        (
            GET_OPERATION,
            GET_BATCH_ID.to_vec(),
            successful_get_payload("server-created-id"),
        ),
    ]);
    let (mut client, fake, captured_request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![5, 11],
    });
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Create(empty_create_request()))
            .with_unique_batch_item_id(CREATE_BATCH_ID.to_vec()),
        ClientBatchItem::new(ClientRequest::Get(GetRequest::new()))
            .with_unique_batch_item_id(GET_BATCH_ID.to_vec()),
    ]);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("Create establishes server-side ID Placeholder for the following Get");

    let captured_request = captured_request.borrow();
    let request = captured_request
        .as_deref()
        .expect("the unchanged batch reaches the fake transport");
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(
        request_item_shape(request, 0),
        Some((CREATE_OPERATION, false))
    );
    assert_eq!(
        request_item_shape(request, 1),
        Some((GET_OPERATION, false)),
        "the Get payload omits Unique Identifier instead of synthesizing one"
    );
    let first = response.get(0).expect("Create remains batch item zero");
    let second = response.get(1).expect("Get remains batch item one");
    assert_eq!(first.unique_batch_item_id(), Some(CREATE_BATCH_ID));
    assert_eq!(second.unique_batch_item_id(), Some(GET_BATCH_ID));
    assert_eq!(first.outcome().operation(), ClientOperation::Create);
    assert_eq!(second.outcome().operation(), ClientOperation::Get);
    assert_eq!(
        second
            .outcome()
            .get_response()
            .and_then(|get| get.unique_identifier()),
        Some("server-created-id")
    );
}
