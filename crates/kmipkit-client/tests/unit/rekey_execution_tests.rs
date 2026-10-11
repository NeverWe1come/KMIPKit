#![cfg(test)]

//! Fake-transport Re-key contracts from OASIS KMIP v2.1 §6.1.46, Tables
//! 303–307. Catalog cases `KMIPKIT-TEST-CN01-2-76` through `-87` have no
//! pinned fixtures, so these are source-derived tests, not official fixture
//! executions. Shared Pending and delivery behavior follows §§8.6, 9.1, 9.2.

use kmipkit_protocol::{
    AttributeSet, CheckRequest, CreateRequest, ObjectType, RekeyRequest, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Structure, StructureView, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{
    asynchronous_response_bytes, operation_batch_response_bytes, test_item, test_structure,
};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const REKEY_OPERATION: u32 = 0x0000_0004;
const CREATE_OPERATION: u32 = 0x0000_0001;
const CHECK_OPERATION: u32 = 0x0000_0009;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const OFFSET: u32 = 0x0042_0058;
const ATTRIBUTES: u32 = 0x0042_0125;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const KEY_VALUE_NOT_PRESENT: u32 = 0x0000_0013;
const PENDING_CORRELATION: &[u8] = b"REKEY_PENDING\x00CORRELATION\xFF";

fn rekey_request() -> RekeyRequest {
    RekeyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("existing-key".to_owned()))
        .with_offset(86_400)
        .with_attributes(test_structure([test_item(
            0x0042_001D,
            Value::enumeration(1),
        )]))
        .with_protection_storage_masks(test_structure([test_item(0x0042_0160, Value::integer(3))]))
}

fn response_payload(identifier: &str) -> Structure {
    test_structure([
        test_item(UNIQUE_IDENTIFIER, Value::text_string(identifier.to_owned())),
        test_item(UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ])
}

fn request_fields(bytes: &[u8], batch_index: usize) -> (Option<u32>, Vec<u32>) {
    let message = decode(bytes).expect("the captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("the KMIP request message is a Structure");
        };
        let batch_items = message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM)
            .collect::<Vec<_>>();
        batch_items[batch_index].with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("the Batch Item is a Structure");
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)
                .and_then(|field| {
                    field.with_value(|value| match value {
                        ValueView::Enumeration(value) => Some(*value),
                        _ => None,
                    })
                });
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("the Batch Item contains a Request Payload");
            let tags = payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("the Request Payload is a Structure");
                };
                payload
                    .children()
                    .iter()
                    .map(|field| field.tag().raw())
                    .collect()
            });
            (operation, tags)
        })
    })
}

fn structure_contains_tag(view: &StructureView<'_>, raw_tag: u32) -> bool {
    view.children().iter().any(|field| {
        field.tag().raw() == raw_tag
            || field.with_value(|value| match value {
                ValueView::Structure(nested) => structure_contains_tag(&nested, raw_tag),
                _ => false,
            })
    })
}

#[test]
fn rekey_sends_one_table_305_request_and_exposes_replacement_identifier() {
    let response = asynchronous_response_bytes(
        REKEY_OPERATION,
        SUCCESS,
        None,
        None,
        Some(response_payload("replacement-key")),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![5, 9],
    });

    let item = client
        .rekey(rekey_request(), &CodecLimits::defaults())
        .expect("Table 306 Re-key success returns the replacement identifier");
    let outcome = item.outcome();
    let typed = outcome
        .response()
        .rekey()
        .expect("the result retains its typed Re-key response");

    assert_eq!(outcome.operation(), ClientOperation::Rekey);
    assert_eq!(outcome.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.unique_identifier(),
        Some(&UniqueIdentifier::TextString("replacement-key".to_owned()))
    );
    assert!(typed.with_ttlv(|view| structure_contains_tag(&view, UNKNOWN_VENDOR_TAG)));
    assert_eq!(
        request_fields(
            captured
                .borrow()
                .as_deref()
                .expect("the fake transport captured the Re-key request"),
            0,
        ),
        (
            Some(REKEY_OPERATION),
            vec![
                UNIQUE_IDENTIFIER,
                OFFSET,
                ATTRIBUTES,
                PROTECTION_STORAGE_MASKS
            ]
        )
    );
    assert_eq!(fake.borrow().exchange_count(), 1, "Re-key is never retried");
}

#[test]
fn rekey_failure_preserves_table_307_result_without_retry() {
    let response = asynchronous_response_bytes(
        REKEY_OPERATION,
        OPERATION_FAILED,
        Some(KEY_VALUE_NOT_PRESENT),
        None,
        None,
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let item = client
        .rekey(rekey_request(), &CodecLimits::defaults())
        .expect("Table 307 Operation Failed remains a KMIP result");
    assert_eq!(item.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        item.outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(KEY_VALUE_NOT_PRESENT)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn rekey_pending_preserves_correlation_without_implicit_poll() {
    let response = asynchronous_response_bytes(
        REKEY_OPERATION,
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Rekey(
        RekeyRequest::new()
            .with_unique_identifier(UniqueIdentifier::TextString("existing-key".to_owned())),
    )))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("caller-enabled typed batch preserves Re-key Pending");
    let item = response.get(0).expect("one Re-key result is associated");
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("the server's Operation Pending result remains Pending");
    };

    assert_eq!(pending.operation(), ClientOperation::Rekey);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn rekey_can_consume_a_preceding_successful_create_placeholder() {
    let response = operation_batch_response_bytes([
        (
            CREATE_OPERATION,
            b"create-item".to_vec(),
            test_structure([
                test_item(OBJECT_TYPE, Value::enumeration(2)),
                test_item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string("existing-key".to_owned()),
                ),
            ]),
        ),
        (
            REKEY_OPERATION,
            b"rekey-item".to_vec(),
            response_payload("replacement-key"),
        ),
    ]);
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Create(CreateRequest::new(
            ObjectType::from_raw(2),
            AttributeSet::new(),
        )))
        .with_unique_batch_item_id(b"create-item".to_vec()),
        ClientBatchItem::new(ClientRequest::Rekey(RekeyRequest::new()))
            .with_unique_batch_item_id(b"rekey-item".to_vec()),
    ])
    .with_batch_order_option(true)
    .with_batch_error_continuation_option(1);

    client
        .execute(batch, &CodecLimits::defaults())
        .expect("Re-key can use the preceding Create response ID Placeholder");

    let request = captured.borrow();
    let request = request
        .as_deref()
        .expect("the ordered Create and Re-key batch reaches the fake transport");
    assert_eq!(
        request_fields(request, 1),
        (Some(REKEY_OPERATION), Vec::new())
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn rekey_can_produce_the_placeholder_for_a_later_check() {
    let (mut client, fake, captured) = client_for(ExchangeScript::FailBeforeWrite);
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Rekey(rekey_request()))
            .with_unique_batch_item_id(b"rekey-item".to_vec()),
        ClientBatchItem::new(ClientRequest::Check(CheckRequest::new()))
            .with_unique_batch_item_id(b"check-item".to_vec()),
    ]);

    let error = client
        .execute(batch, &CodecLimits::defaults())
        .expect_err("the fake transport stops the exchange before any bytes are sent");

    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let captured_request = captured.borrow();
    let request = captured_request
        .as_deref()
        .expect("the Re-key and Check batch reaches the fake transport");
    assert_eq!(
        request_fields(request, 1),
        (Some(CHECK_OPERATION), Vec::new()),
        "§4.1.2 allows the next operation to use Re-key's returned ID Placeholder"
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn rekey_transport_error_preserves_delivery_state_after_one_exchange() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });

    let error = client
        .rekey(rekey_request(), &CodecLimits::defaults())
        .expect_err("the fake transport deliberately fails after a partial write");

    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
