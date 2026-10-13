#![cfg(test)]

//! Fake-transport Obtain Lease contracts from OASIS KMIP v2.1 §6.1.35,
//! Tables 268–270. Stable requirement coverage:
//! `KMIPKIT-REQ-SPEC-6.1.35-001-001`, `-001-002`, `-002`, and `-005`.
//! Shared Pending and delivery behavior follows KMIP v2.1 §§8.6, 9.1, and
//! 9.2. These are derived tests, not official OASIS Test Case executions.

use kmipkit_protocol::{
    AttributeSet, CreateRequest, ObjectType, ObtainLeaseRequest, UniqueIdentifier,
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

const OBTAIN_LEASE_OPERATION: u32 = 0x0000_0010;
const CREATE_OPERATION: u32 = 0x0000_0001;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const OBJECT_TYPE: u32 = 0x0042_0057;
const LEASE_TIME: u32 = 0x0042_0049;
const LAST_CHANGE_DATE: u32 = 0x0042_0048;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const OBJECT_NOT_FOUND: u32 = 0x0000_0037;
const PENDING_CORRELATION: &[u8] = b"OBTAIN_LEASE_PENDING\x00CORRELATION\xFF";

fn obtain_lease_request(identifier: &str) -> ObtainLeaseRequest {
    ObtainLeaseRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
}

fn response_payload(identifier: &str, lease_time: u32) -> Structure {
    test_structure([
        test_item(UNIQUE_IDENTIFIER, Value::text_string(identifier.to_owned())),
        test_item(LEASE_TIME, Value::interval(lease_time)),
        test_item(LAST_CHANGE_DATE, Value::date_time(1_700_000_000)),
        test_item(UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ])
}

fn obtain_lease_response_bytes(
    status: u32,
    reason: Option<u32>,
    correlation: Option<&[u8]>,
    payload: Option<Structure>,
) -> Vec<u8> {
    asynchronous_response_bytes(OBTAIN_LEASE_OPERATION, status, reason, correlation, payload)
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
fn obtain_lease_sends_table_268_and_exposes_required_table_269_values_once() {
    crate::execute_test_support::assert_lifecycle_request_debug(
        &crate::ClientRequest::obtain_lease(obtain_lease_request("debug-lease")),
        "ObtainLease",
    );
    let response = obtain_lease_response_bytes(
        SUCCESS,
        None,
        None,
        Some(response_payload("leased-object", 0)),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![4, 7],
    });

    let item = client
        .obtain_lease(
            obtain_lease_request("leased-object"),
            &CodecLimits::defaults(),
        )
        .expect("a Table 269 success returns the required lease metadata");
    let outcome = item.outcome();
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        outcome,
        ClientOperation::ObtainLease,
        "ObtainLease(",
        "ObtainLease(",
    );
    let typed = outcome
        .response()
        .obtain_lease()
        .expect("the result retains its typed Obtain Lease response");

    assert_eq!(outcome.operation(), ClientOperation::ObtainLease);
    assert_eq!(outcome.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.unique_identifier(),
        Some(&UniqueIdentifier::TextString("leased-object".to_owned()))
    );
    assert_eq!(typed.lease_time(), Some(0));
    assert_eq!(typed.last_change_date(), Some(1_700_000_000));
    assert!(typed.with_ttlv(|view| structure_contains_tag(&view, UNKNOWN_VENDOR_TAG)));
    assert_eq!(
        request_fields(
            captured
                .borrow()
                .as_deref()
                .expect("fake transport captured the Obtain Lease request"),
            0,
        ),
        (Some(OBTAIN_LEASE_OPERATION), vec![UNIQUE_IDENTIFIER])
    );
    assert_eq!(
        fake.borrow().exchange_count(),
        1,
        "Obtain Lease is never retried"
    );
}

#[test]
fn obtain_lease_failure_preserves_table_270_reason_without_retry() {
    let response =
        obtain_lease_response_bytes(OPERATION_FAILED, Some(OBJECT_NOT_FOUND), None, None);
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let item = client
        .obtain_lease(
            obtain_lease_request("missing-object"),
            &CodecLimits::defaults(),
        )
        .expect("Table 270 Operation Failed remains a KMIP result");
    assert_eq!(item.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        item.outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(OBJECT_NOT_FOUND)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn obtain_lease_pending_preserves_correlation_without_implicit_poll() {
    let response = obtain_lease_response_bytes(
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::ObtainLease(
        obtain_lease_request("leased-object"),
    )))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("caller-enabled typed batch preserves Obtain Lease Pending");
    let item = response
        .get(0)
        .expect("one Obtain Lease result is associated");
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        item.outcome(),
        ClientOperation::ObtainLease,
        "ObtainLease(",
        "Pending(",
    );
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("the server's Operation Pending result remains Pending");
    };

    assert_eq!(pending.operation(), ClientOperation::ObtainLease);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn obtain_lease_can_use_a_preceding_successful_create_placeholder() {
    let response = operation_batch_response_bytes([
        (
            CREATE_OPERATION,
            b"create-item".to_vec(),
            test_structure([
                test_item(OBJECT_TYPE, Value::enumeration(7)),
                test_item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string("created-object".to_owned()),
                ),
            ]),
        ),
        (
            OBTAIN_LEASE_OPERATION,
            b"lease-item".to_vec(),
            response_payload("created-object", 30),
        ),
    ]);
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Create(CreateRequest::new(
            ObjectType::from_raw(7),
            AttributeSet::new(),
        )))
        .with_unique_batch_item_id(b"create-item".to_vec()),
        ClientBatchItem::new(ClientRequest::ObtainLease(ObtainLeaseRequest::new()))
            .with_unique_batch_item_id(b"lease-item".to_vec()),
    ])
    .with_batch_order_option(true)
    .with_batch_error_continuation_option(1);

    client
        .execute(batch, &CodecLimits::defaults())
        .expect("Obtain Lease can use the preceding Create response ID Placeholder");

    let request = captured.borrow();
    let request = request
        .as_deref()
        .expect("the ordered Create and Obtain Lease batch reaches the fake transport");
    assert_eq!(
        request_fields(request, 1),
        (Some(OBTAIN_LEASE_OPERATION), vec![])
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn obtain_lease_transport_error_preserves_delivery_state_after_one_exchange() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });

    let error = client
        .obtain_lease(
            obtain_lease_request("leased-object"),
            &CodecLimits::defaults(),
        )
        .expect_err("the fake transport deliberately fails after a partial write");

    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
