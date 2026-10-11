#![cfg(test)]

//! Fake-transport Re-key Key Pair contracts from OASIS KMIP v2.1 §6.1.47,
//! Tables 308–312. These source-derived tests do not claim official fixture
//! execution; the pinned catalog has no direct Re-key Key Pair test case.

use kmipkit_protocol::{
    AttributeSet, CheckRequest, CreateRequest, ObjectType, RekeyKeyPairRequest, UniqueIdentifier,
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

const REKEY_KEY_PAIR_OPERATION: u32 = 0x0000_001d;
const CREATE_OPERATION: u32 = 0x0000_0001;
const CHECK_OPERATION: u32 = 0x0000_0009;
const BATCH_ITEM: u32 = 0x0042_000f;
const OPERATION: u32 = 0x0042_005c;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const PRIVATE_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_0066;
const PUBLIC_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_006f;
const OFFSET: u32 = 0x0042_0058;
const COMMON_ATTRIBUTES: u32 = 0x0042_0126;
const PRIVATE_KEY_ATTRIBUTES: u32 = 0x0042_0127;
const PUBLIC_KEY_ATTRIBUTES: u32 = 0x0042_0128;
const COMMON_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0163;
const PRIVATE_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0164;
const PUBLIC_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0165;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const KEY_VALUE_NOT_PRESENT: u32 = 0x0000_0013;
const PENDING_CORRELATION: &[u8] = b"REKEY_KEY_PAIR_PENDING_CORRELATION\xFF";

fn rekey_key_pair_request() -> RekeyKeyPairRequest {
    RekeyKeyPairRequest::new()
        .with_private_key_unique_identifier(UniqueIdentifier::TextString(
            "existing-private-key".to_owned(),
        ))
        .with_offset(86_400)
        .with_common_attributes(AttributeSet::new())
        .with_private_key_attributes(AttributeSet::new())
        .with_public_key_attributes(AttributeSet::new())
        .with_common_protection_storage_masks(test_structure([test_item(
            0x0042_0160,
            Value::integer(1),
        )]))
        .with_private_protection_storage_masks(test_structure([test_item(
            0x0042_0160,
            Value::integer(2),
        )]))
        .with_public_protection_storage_masks(test_structure([test_item(
            0x0042_0160,
            Value::integer(3),
        )]))
}

fn response_payload() -> Structure {
    test_structure([
        test_item(
            PRIVATE_KEY_UNIQUE_IDENTIFIER,
            Value::text_string("replacement-private-key".to_owned()),
        ),
        test_item(
            PUBLIC_KEY_UNIQUE_IDENTIFIER,
            Value::text_string("replacement-public-key".to_owned()),
        ),
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
fn rekey_key_pair_sends_table_310_and_returns_both_identifiers_once() {
    let response = asynchronous_response_bytes(
        REKEY_KEY_PAIR_OPERATION,
        SUCCESS,
        None,
        None,
        Some(response_payload()),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![5, 9],
    });

    let item = client
        .rekey_key_pair(rekey_key_pair_request(), &CodecLimits::defaults())
        .expect("Table 311 success returns both replacement key identifiers");
    let outcome = item.outcome();
    let typed = outcome
        .response()
        .rekey_key_pair()
        .expect("the result retains its typed Re-key Key Pair response");

    assert_eq!(outcome.operation(), ClientOperation::RekeyKeyPair);
    assert_eq!(outcome.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.private_key_unique_identifier(),
        Some(&UniqueIdentifier::TextString(
            "replacement-private-key".to_owned()
        ))
    );
    assert_eq!(
        typed.public_key_unique_identifier(),
        Some(&UniqueIdentifier::TextString(
            "replacement-public-key".to_owned()
        ))
    );
    assert!(typed.with_ttlv(|view| structure_contains_tag(&view, UNKNOWN_VENDOR_TAG)));
    assert_eq!(
        request_fields(
            captured
                .borrow()
                .as_deref()
                .expect("the fake transport captured the Re-key Key Pair request"),
            0,
        ),
        (
            Some(REKEY_KEY_PAIR_OPERATION),
            vec![
                PRIVATE_KEY_UNIQUE_IDENTIFIER,
                OFFSET,
                COMMON_ATTRIBUTES,
                PRIVATE_KEY_ATTRIBUTES,
                PUBLIC_KEY_ATTRIBUTES,
                COMMON_PROTECTION_STORAGE_MASKS,
                PRIVATE_PROTECTION_STORAGE_MASKS,
                PUBLIC_PROTECTION_STORAGE_MASKS,
            ]
        )
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn rekey_key_pair_failure_preserves_table_312_result_without_retry() {
    let response = asynchronous_response_bytes(
        REKEY_KEY_PAIR_OPERATION,
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
        .rekey_key_pair(rekey_key_pair_request(), &CodecLimits::defaults())
        .expect("Table 312 failure remains a KMIP operation result");
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
fn rekey_key_pair_pending_preserves_correlation_without_implicit_poll() {
    let response = asynchronous_response_bytes(
        REKEY_KEY_PAIR_OPERATION,
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::RekeyKeyPair(
        RekeyKeyPairRequest::new().with_private_key_unique_identifier(
            UniqueIdentifier::TextString("existing-private-key".to_owned()),
        ),
    )))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("caller-enabled typed batch preserves Re-key Key Pair Pending");
    let item = response
        .get(0)
        .expect("one Re-key Key Pair result is associated");
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("the server's Operation Pending result remains Pending");
    };

    assert_eq!(pending.operation(), ClientOperation::RekeyKeyPair);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn rekey_key_pair_can_consume_a_preceding_create_id_placeholder() {
    let response = operation_batch_response_bytes([
        (
            CREATE_OPERATION,
            b"create-item".to_vec(),
            test_structure([
                test_item(OBJECT_TYPE, Value::enumeration(2)),
                test_item(
                    0x0042_0094,
                    Value::text_string("existing-private-key".to_owned()),
                ),
            ]),
        ),
        (
            REKEY_KEY_PAIR_OPERATION,
            b"rekey-pair-item".to_vec(),
            response_payload(),
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
        ClientBatchItem::new(ClientRequest::RekeyKeyPair(RekeyKeyPairRequest::new()))
            .with_unique_batch_item_id(b"rekey-pair-item".to_vec()),
    ])
    .with_batch_order_option(true)
    .with_batch_error_continuation_option(1);

    client
        .execute(batch, &CodecLimits::defaults())
        .expect("Re-key Key Pair can use a preceding Create ID Placeholder");
    assert_eq!(
        request_fields(captured.borrow().as_deref().expect("request captured"), 1),
        (Some(REKEY_KEY_PAIR_OPERATION), Vec::new())
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn rekey_key_pair_response_produces_private_key_id_placeholder_for_check() {
    let (mut client, fake, captured) = client_for(ExchangeScript::FailBeforeWrite);
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::RekeyKeyPair(rekey_key_pair_request()))
            .with_unique_batch_item_id(b"rekey-pair-item".to_vec()),
        ClientBatchItem::new(ClientRequest::Check(CheckRequest::new()))
            .with_unique_batch_item_id(b"check-item".to_vec()),
    ]);

    let error = client
        .execute(batch, &CodecLimits::defaults())
        .expect_err("the fake transport fails before writing any bytes");
    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(
        request_fields(captured.borrow().as_deref().expect("request captured"), 1),
        (Some(CHECK_OPERATION), Vec::new()),
        "§4.1.2 specifies Re-key Key Pair's private replacement UID as the ID Placeholder"
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn rekey_key_pair_transport_error_preserves_partial_write_delivery_without_retry() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = client
        .rekey_key_pair(rekey_key_pair_request(), &CodecLimits::defaults())
        .expect_err("the fake transport fails after a partial write");

    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
