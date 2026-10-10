#![cfg(test)]

//! Client-level ID Placeholder contracts derived from KMIP v2.1 §6.1,
//! §6.1.8/Table 187, §6.1.11/Table 196, §6.1.17/Table 214, and §9.8.
//! Traceability: `KMIPKIT-REQ-SPEC-6.1-003-002`, `KMIPKIT-0019-FR-002`, and
//! `KMIPKIT-REQ-SPEC-6.1-001-003`. These tests exercise ID Placeholder
//! eligibility and result association through the typed client path.

use kmipkit_protocol::{
    AttributeSet, CreateRequest, CreateSplitKeyRequest, DecryptRequest, EncryptRequest, ObjectType,
    OperationData, ResultReason, SplitKeyMethod,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::{client_for, request_contains};
use crate::execute::encode_message_for_test;
use crate::execute_test_support::{test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientCauseCategory, ClientErrorCategory, ClientRequest,
};

const CREATE_OPERATION: u32 = 0x0000_0001;
const PING_OPERATION: u32 = 0x0000_003B;
const ENCRYPT_OPERATION: u32 = 0x0000_001F;

const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;

const REQUEST_HEADER: u32 = 0x0042_0077;
const BATCH_ORDER_OPTION: u32 = 0x0042_0010;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;

const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const RESPONSE_TOO_LARGE: u32 = 0x02;
const INVALID_FIELD: u32 = 0x07;
const OBJECT_NOT_FOUND: u32 = 0x37;

const CREATE_BATCH_ID: &[u8] = b"id-placeholder-create";
const ENCRYPT_BATCH_ID: &[u8] = b"id-placeholder-encrypt";
const DECRYPT_BATCH_ID: &[u8] = b"id-placeholder-decrypt";

fn create_request() -> CreateRequest {
    CreateRequest::new(ObjectType::from_raw(7), AttributeSet::new())
}

fn encrypt_without_identifier() -> ClientRequest {
    ClientRequest::Encrypt(EncryptRequest::new(
        None,
        Some(OperationData::Enumeration(19)),
    ))
}

fn decrypt_without_identifier() -> ClientRequest {
    ClientRequest::Decrypt(DecryptRequest::new(
        None,
        Some(OperationData::Enumeration(23)),
    ))
}

fn batch_item(request: ClientRequest, id: &[u8]) -> ClientBatchItem {
    ClientBatchItem::new(request).with_unique_batch_item_id(id.to_vec())
}

fn assert_rejected_before_send(batch: ClientBatch) {
    let (mut client, fake, request) = client_for(ExchangeScript::Success {
        response: response_bytes([ResponseItem::failure(
            PING_OPERATION,
            b"unused-response",
            INVALID_FIELD,
        )]),
        request_write_chunks: Vec::new(),
    });

    let error = client
        .execute(batch, &CodecLimits::defaults())
        .expect_err("an ineligible ID Placeholder request fails before transport");

    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(
        error.cause_category(),
        Some(ClientCauseCategory::InvalidInput)
    );
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(
        error.to_string(),
        "client validation failure (invalid input, NotSent)"
    );
    assert_eq!(fake.borrow().exchange_count(), 0);
    assert!(request.borrow().is_none());
}

#[test]
fn encrypt_and_decrypt_without_a_preceding_batch_item_are_rejected_before_send() {
    for request in [encrypt_without_identifier(), decrypt_without_identifier()] {
        assert_rejected_before_send(ClientBatch::new(ClientBatchItem::new(request)));
    }
}

#[test]
fn omitted_identifier_validation_diagnostic_is_static() {
    let error = crate::execute::BatchValidationError::IneligibleIdPlaceholder;

    assert_eq!(
        error.to_string(),
        "omitted Unique Identifier has no eligible preceding ID Placeholder producer"
    );
}

#[test]
fn omitted_identifier_after_ping_is_rejected_before_send() {
    let batch = ClientBatch::from_items([
        batch_item(ClientRequest::ping(), b"ping-before-encrypt"),
        batch_item(encrypt_without_identifier(), ENCRYPT_BATCH_ID),
    ])
    .with_batch_order_option(true);

    assert_rejected_before_send(batch);
}

#[test]
fn omitted_identifier_is_rejected_when_batch_order_is_false() {
    for (request, consumer_id) in [
        (encrypt_without_identifier(), ENCRYPT_BATCH_ID),
        (decrypt_without_identifier(), DECRYPT_BATCH_ID),
    ] {
        let batch = ClientBatch::from_items([
            batch_item(ClientRequest::Create(create_request()), CREATE_BATCH_ID),
            batch_item(request, consumer_id),
        ])
        .with_batch_order_option(false);

        assert_rejected_before_send(batch);
    }
}

#[test]
fn later_encrypt_omits_identifier_with_batch_order_enabled() {
    let response = response_bytes([
        ResponseItem::failure(ENCRYPT_OPERATION, ENCRYPT_BATCH_ID, OBJECT_NOT_FOUND),
        ResponseItem::create_success(CREATE_BATCH_ID, "created-object"),
    ]);
    let (mut client, fake, captured_request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![5, 9],
    });
    let batch = ClientBatch::from_items([
        batch_item(ClientRequest::Create(create_request()), CREATE_BATCH_ID),
        batch_item(encrypt_without_identifier(), ENCRYPT_BATCH_ID),
    ])
    .with_batch_order_option(true);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("an eligible later Encrypt item is sent and receives its server result");

    assert_eq!(fake.borrow().exchange_count(), 1);
    assert!(request_contains(&captured_request, CREATE_BATCH_ID));
    assert!(request_contains(&captured_request, ENCRYPT_BATCH_ID));
    let captured_request = captured_request.borrow();
    let request = captured_request
        .as_ref()
        .expect("the eligible ordered batch reached the fake transport");
    assert_eq!(request_batch_order_option(request), Some(true));
    assert_eq!(request_item_operation(request, 0), Some(CREATE_OPERATION));
    assert_eq!(request_item_operation(request, 1), Some(ENCRYPT_OPERATION));
    assert!(!request_payload_has_identifier(request, 0));
    assert!(!request_payload_has_identifier(request, 1));
    assert_eq!(response.len(), 2);
    assert_eq!(
        response.get(0).and_then(|item| item.unique_batch_item_id()),
        Some(CREATE_BATCH_ID)
    );
    assert_eq!(
        response.get(1).and_then(|item| item.unique_batch_item_id()),
        Some(ENCRYPT_BATCH_ID)
    );
}

#[test]
fn later_encrypt_uses_id_placeholder_when_batch_order_option_is_unspecified() {
    let response = response_bytes([
        ResponseItem::failure(ENCRYPT_OPERATION, ENCRYPT_BATCH_ID, OBJECT_NOT_FOUND),
        ResponseItem::create_success(CREATE_BATCH_ID, "created-object"),
    ]);
    let (mut client, fake, captured_request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![5, 9],
    });
    let batch = ClientBatch::from_items([
        batch_item(ClientRequest::Create(create_request()), CREATE_BATCH_ID),
        batch_item(encrypt_without_identifier(), ENCRYPT_BATCH_ID),
    ]);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("an omitted Batch Order Option has the KMIP effective value True");

    assert_eq!(fake.borrow().exchange_count(), 1);
    let captured_request = captured_request.borrow();
    let request = captured_request
        .as_ref()
        .expect("the eligible ordered batch reached the fake transport");
    assert_eq!(request_batch_order_option(request), None);
    assert_eq!(request_item_operation(request, 0), Some(CREATE_OPERATION));
    assert_eq!(request_item_operation(request, 1), Some(ENCRYPT_OPERATION));
    assert!(!request_payload_has_identifier(request, 1));
    assert_eq!(response.len(), 2);
    assert_eq!(
        response.get(0).and_then(|item| item.unique_batch_item_id()),
        Some(CREATE_BATCH_ID)
    );
    assert_eq!(
        response.get(1).and_then(|item| item.unique_batch_item_id()),
        Some(ENCRYPT_BATCH_ID)
    );
    assert_eq!(
        response
            .get(0)
            .map(|item| item.outcome().result().status().raw()),
        Some(SUCCESS)
    );
    assert_eq!(
        response
            .get(1)
            .map(|item| item.outcome().result().status().raw()),
        Some(OPERATION_FAILED)
    );
}

#[test]
fn create_split_key_does_not_make_a_later_identifier_eligible() {
    let create_split_key = CreateSplitKeyRequest::new(
        ObjectType::from_raw(7),
        3,
        2,
        SplitKeyMethod::XOR,
        AttributeSet::new(),
    );
    let batch = ClientBatch::from_items([
        batch_item(
            ClientRequest::CreateSplitKey(create_split_key),
            CREATE_BATCH_ID,
        ),
        batch_item(encrypt_without_identifier(), ENCRYPT_BATCH_ID),
    ])
    .with_batch_order_option(true);

    assert_rejected_before_send(batch);
}

#[test]
fn server_failure_of_placeholder_producer_preserves_each_batch_item_result() {
    let response = response_bytes([
        ResponseItem::failure(ENCRYPT_OPERATION, ENCRYPT_BATCH_ID, OBJECT_NOT_FOUND),
        ResponseItem::failure(CREATE_OPERATION, CREATE_BATCH_ID, RESPONSE_TOO_LARGE),
    ]);
    let (mut client, fake, captured_request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::from_items([
        batch_item(ClientRequest::Create(create_request()), CREATE_BATCH_ID),
        batch_item(encrypt_without_identifier(), ENCRYPT_BATCH_ID),
    ])
    .with_batch_order_option(true)
    .with_batch_error_continuation_option(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the client exposes both server results without predicting placeholder state");

    assert_eq!(fake.borrow().exchange_count(), 1);
    assert!(captured_request.borrow().is_some());
    assert_eq!(response.len(), 2);
    let first = response.get(0).expect("Create remains request item zero");
    let second = response.get(1).expect("Encrypt remains request item one");
    assert_eq!(first.unique_batch_item_id(), Some(CREATE_BATCH_ID));
    assert_eq!(second.unique_batch_item_id(), Some(ENCRYPT_BATCH_ID));
    assert_eq!(first.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        first.outcome().result().reason().map(ResultReason::raw),
        Some(RESPONSE_TOO_LARGE)
    );
    assert_eq!(second.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        second.outcome().result().reason().map(ResultReason::raw),
        Some(OBJECT_NOT_FOUND)
    );
}

fn request_batch_order_option(request: &[u8]) -> Option<bool> {
    let document = decode(request).ok()?;
    document.with_value(|message| {
        let ValueView::Structure(message) = message else {
            return None;
        };
        let header = message
            .children()
            .iter()
            .find(|item| item.tag().raw() == REQUEST_HEADER)?;
        header.with_value(|header| {
            let ValueView::Structure(header) = header else {
                return None;
            };
            let option = header
                .children()
                .iter()
                .find(|item| item.tag().raw() == BATCH_ORDER_OPTION)?;
            option.with_value(|value| match value {
                ValueView::Boolean(value) => Some(*value),
                _ => None,
            })
        })
    })
}

fn request_item_operation(request: &[u8], index: usize) -> Option<u32> {
    let document = decode(request).ok()?;
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
                .find(|item| item.tag().raw() == OPERATION)?;
            operation.with_value(|value| match value {
                ValueView::Enumeration(value) => Some(*value),
                _ => None,
            })
        })
    })
}

fn request_payload_has_identifier(request: &[u8], index: usize) -> bool {
    let Some(document) = decode(request).ok() else {
        return false;
    };
    document.with_value(|message| {
        let ValueView::Structure(message) = message else {
            return false;
        };
        let Some(batch_item) = message
            .children()
            .iter()
            .filter(|item| item.tag().raw() == BATCH_ITEM)
            .nth(index)
        else {
            return false;
        };
        batch_item.with_value(|batch_item| {
            let ValueView::Structure(batch_item) = batch_item else {
                return false;
            };
            let Some(payload) = batch_item
                .children()
                .iter()
                .find(|item| item.tag().raw() == REQUEST_PAYLOAD)
            else {
                return false;
            };
            payload.with_value(|payload| {
                let ValueView::Structure(payload) = payload else {
                    return false;
                };
                payload
                    .children()
                    .iter()
                    .any(|item| item.tag().raw() == UNIQUE_IDENTIFIER)
            })
        })
    })
}

struct ResponseItem {
    operation: u32,
    id: &'static [u8],
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
}

impl ResponseItem {
    fn failure(operation: u32, id: &'static [u8], reason: u32) -> Self {
        Self {
            operation,
            id,
            status: OPERATION_FAILED,
            reason: Some(reason),
            payload: None,
        }
    }

    fn create_success(id: &'static [u8], identifier: &str) -> Self {
        Self {
            operation: CREATE_OPERATION,
            id,
            status: SUCCESS,
            reason: None,
            payload: Some(test_structure([
                test_item(OBJECT_TYPE, Value::enumeration(7)),
                test_item(UNIQUE_IDENTIFIER, Value::text_string(identifier.to_owned())),
            ])),
        }
    }
}

fn response_bytes(items: impl IntoIterator<Item = ResponseItem>) -> Vec<u8> {
    let items = items.into_iter().collect::<Vec<_>>();
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(
            BATCH_COUNT,
            Value::integer(i32::try_from(items.len()).expect("fixture count fits i32")),
        ),
    ]);
    let mut message = vec![test_item(RESPONSE_HEADER, Value::structure(header))];
    for item in items {
        let mut fields = vec![
            test_item(OPERATION, Value::enumeration(item.operation)),
            test_item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(item.id.to_vec())),
            test_item(RESULT_STATUS, Value::enumeration(item.status)),
        ];
        if let Some(reason) = item.reason {
            fields.push(test_item(RESULT_REASON, Value::enumeration(reason)));
        }
        if let Some(payload) = item.payload {
            fields.push(test_item(RESPONSE_PAYLOAD, Value::structure(payload)));
        }
        message.push(test_item(
            BATCH_ITEM,
            Value::structure(test_structure(fields)),
        ));
    }
    encode_message_for_test(test_structure(message), &CodecLimits::defaults())
        .expect("the response fixture is a valid KMIP 2.1 message")
}
