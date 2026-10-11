#![cfg(test)]

//! Fake-transport Register contracts from OASIS KMIP v2.1 §6.1.43,
//! Tables 291–294. Traceability: all `KMIPKIT-REQ-SPEC-6.1.43-*` rows.
//! These are derived execution tests, not official fixture passes.

use kmipkit_protocol::{AttributeSet, CheckRequest, ObjectType, RegisterRequest};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const REGISTER_OPERATION: u32 = 0x0000_0003;
const CHECK_OPERATION: u32 = 0x0000_0009;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_LENGTH: u32 = 0x0042_00AD;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const INVALID_OBJECT_TYPE: u32 = 0x0000_0038;
const PENDING_CORRELATION: &[u8] = b"REGISTER_PENDING_CORRELATION\xFF";

fn register_request() -> RegisterRequest {
    RegisterRequest::new(
        ObjectType::from_raw(1),
        AttributeSet::try_new([test_item(CERTIFICATE_LENGTH, Value::integer(512))])
            .expect("Table 293 certificate attribute is correctly typed"),
        test_item(
            CERTIFICATE,
            Value::structure(test_structure([test_item(
                UNKNOWN_VENDOR_TAG,
                Value::byte_string(b"certificate-value".to_vec()),
            )])),
        ),
    )
}

fn request_fields(bytes: &[u8], item_index: usize) -> (Option<u32>, Vec<u32>, Vec<Vec<u32>>) {
    let message = decode(bytes).expect("captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("KMIP request message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM)
            .nth(item_index)
            .expect("request contains the selected Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("Batch Item is a Structure");
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)
                .and_then(|field| {
                    field.with_value(|value| match value {
                        ValueView::Enumeration(raw) => Some(*raw),
                        _ => None,
                    })
                });
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("request contains a Request Payload");
            let (tags, nested) = payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                let tags = payload
                    .children()
                    .iter()
                    .map(|field| field.tag().raw())
                    .collect();
                let nested = payload
                    .children()
                    .iter()
                    .filter_map(|field| {
                        field.with_value(|value| match value {
                            ValueView::Structure(structure) => Some(
                                structure
                                    .children()
                                    .iter()
                                    .map(|child| child.tag().raw())
                                    .collect(),
                            ),
                            _ => None,
                        })
                    })
                    .collect();
                (tags, nested)
            });
            (operation, tags, nested)
        })
    })
}

#[test]
fn register_sends_table_291_and_exposes_table_292_identifier_once() {
    crate::execute_test_support::assert_lifecycle_request_debug(
        &crate::ClientRequest::register(register_request()),
        "Register",
    );
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("registered-certificate".to_owned()),
        )])),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![5, 9],
    });

    let item = client
        .register(register_request(), &CodecLimits::defaults())
        .expect("valid Table 291 request receives typed Table 292 success");
    let outcome = item.outcome();
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        outcome,
        ClientOperation::Register,
        "Register(",
        "Register(",
    );
    let typed = outcome
        .response()
        .register()
        .expect("result retains its typed Register response");
    assert_eq!(outcome.operation(), ClientOperation::Register);
    assert_eq!(outcome.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.unique_identifier(),
        Some(&kmipkit_protocol::UniqueIdentifier::TextString(
            "registered-certificate".to_owned()
        ))
    );
    let request = captured.borrow();
    let request = request.as_deref().expect("fake transport captured request");
    let (operation, fields, nested) = request_fields(request, 0);
    assert_eq!(
        operation,
        Some(REGISTER_OPERATION),
        "§11.36 assigns Register Operation 0x00000003"
    );
    assert_eq!(fields, [OBJECT_TYPE, ATTRIBUTES, CERTIFICATE]);
    assert_eq!(nested, [vec![CERTIFICATE_LENGTH], vec![UNKNOWN_VENDOR_TAG]]);
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_produces_id_placeholder_for_a_later_identifier_omitting_operation() {
    let (mut client, fake, captured) = client_for(ExchangeScript::FailBeforeWrite);
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Register(register_request()))
            .with_unique_batch_item_id(b"register-item".to_vec()),
        ClientBatchItem::new(ClientRequest::Check(CheckRequest::new()))
            .with_unique_batch_item_id(b"check-item".to_vec()),
    ]);

    let error = client
        .execute(batch, &CodecLimits::defaults())
        .expect_err("fake transport stops before any request bytes are sent");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let request = captured.borrow();
    let request = request.as_deref().expect("request reaches fake transport");
    assert_eq!(
        request_fields(request, 1),
        (Some(CHECK_OPERATION), Vec::new(), Vec::new()),
        "§4.1.2 permits Check to consume Register's returned ID Placeholder"
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_failure_preserves_table_294_reason_without_retry() {
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        OPERATION_FAILED,
        Some(INVALID_OBJECT_TYPE),
        None,
        None,
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let item = client
        .register(register_request(), &CodecLimits::defaults())
        .expect("Table 294 Operation Failed is preserved as a KMIP result");
    assert_eq!(item.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        item.outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(INVALID_OBJECT_TYPE)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_pending_preserves_correlation_without_implicit_poll() {
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Register(
        register_request(),
    )))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("caller-enabled typed batch preserves Register Pending");
    let item = response.get(0).expect("one Register result is associated");
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        item.outcome(),
        ClientOperation::Register,
        "Register(",
        "Pending(",
    );
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("server Operation Pending remains Pending");
    };
    assert_eq!(pending.operation(), ClientOperation::Register);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_transport_failure_preserves_partial_write_delivery_without_retry() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = client
        .register(register_request(), &CodecLimits::defaults())
        .expect_err("fake transport fails after partial request write");
    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
