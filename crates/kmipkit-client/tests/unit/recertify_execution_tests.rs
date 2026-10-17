#![cfg(test)]

//! Fake-transport Re-certify contracts from OASIS KMIP v2.1 §6.1.45,
//! Tables 298–302. Stable requirement coverage: `KMIPKIT-REQ-SPEC-6.1.45-001`,
//! `-002-001`, `-002-002`, `-004`, `-006-001`, `-006-002`, and `-008`.
//! Shared Pending and delivery behavior follows KMIP v2.1 §§8.6, 9.1, and
//! 9.2. These are derived tests, not official OASIS Test Case executions.

use kmipkit_protocol::{
    AttributeSet, CreateRequest, ObjectType, RecertifyRequest, SecretBytes, UniqueIdentifier,
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

const RECERTIFY_OPERATION: u32 = 0x0000_0007;
const CREATE_OPERATION: u32 = 0x0000_0001;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CERTIFICATE_REQUEST_UNIQUE_IDENTIFIER: u32 = 0x0042_0139;
const CERTIFICATE_REQUEST_TYPE: u32 = 0x0042_0019;
const CERTIFICATE_REQUEST_VALUE: u32 = 0x0042_0140;
const OFFSET: u32 = 0x0042_0058;
const ATTRIBUTES: u32 = 0x0042_0125;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const OBJECT_TYPE: u32 = 0x0042_0057;
const CERTIFICATE_TYPE: u32 = 0x0042_001D;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const INVALID_CSR: u32 = 0x0000_002F;
const PENDING_CORRELATION: &[u8] = b"RECERTIFY_PENDING\x00CORRELATION\xFF";
const CERTIFICATE_REQUEST_BYTES: &[u8] = b"opaque-certificate-request";

fn recertify_request() -> RecertifyRequest {
    RecertifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(
            "existing-certificate".to_owned(),
        ))
        .with_certificate_request_unique_identifier(UniqueIdentifier::Integer(17))
        .with_certificate_request_type(1)
        .with_certificate_request_value(SecretBytes::new(CERTIFICATE_REQUEST_BYTES.to_vec()))
        .with_offset(86_400)
        .with_attributes(certificate_attributes())
        .with_protection_storage_masks(test_structure([test_item(0x0042_0160, Value::integer(3))]))
}

fn certificate_attributes() -> Structure {
    test_structure([test_item(CERTIFICATE_TYPE, Value::enumeration(1))])
}

fn response_payload(identifier: &str) -> Structure {
    test_structure([
        test_item(UNIQUE_IDENTIFIER, Value::text_string(identifier.to_owned())),
        test_item(UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ])
}

fn request_fields(bytes: &[u8], batch_index: usize) -> (Option<u32>, Vec<u32>, usize) {
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
            let (tags, request_values) = payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("the Request Payload is a Structure");
                };
                let tags = payload
                    .children()
                    .iter()
                    .map(|field| field.tag().raw())
                    .collect();
                let request_values = payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == CERTIFICATE_REQUEST_VALUE)
                    .count();
                (tags, request_values)
            });
            (operation, tags, request_values)
        })
    })
}

fn request_unique_identifier(bytes: &[u8], batch_index: usize) -> Option<String> {
    let message = decode(bytes).expect("the captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return None;
        };
        let batch = message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM)
            .nth(batch_index)?;
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                return None;
            };
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)?;
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    return None;
                };
                let identifier = payload
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == UNIQUE_IDENTIFIER)?;
                identifier.with_value(|value| match value {
                    ValueView::TextString(value) => Some(value.to_owned()),
                    _ => None,
                })
            })
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
fn recertify_sends_one_table_300_request_and_exposes_table_301_identifier() {
    crate::execute_test_support::assert_lifecycle_request_debug(
        &crate::ClientRequest::recertify(recertify_request()),
        "Recertify",
    );
    let response = asynchronous_response_bytes(
        RECERTIFY_OPERATION,
        SUCCESS,
        None,
        None,
        Some(response_payload("replacement-certificate")),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![5, 9],
    });

    let item = client
        .recertify(recertify_request(), &CodecLimits::defaults())
        .expect("Table 301 Re-certify success returns the new certificate identifier");
    let outcome = item.outcome();
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        outcome,
        ClientOperation::Recertify,
        "Recertify(",
        "Re-certify(",
    );
    let typed = outcome
        .response()
        .recertify()
        .expect("the result retains its typed Re-certify response");

    assert_eq!(outcome.operation(), ClientOperation::Recertify);
    assert_eq!(outcome.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.unique_identifier(),
        Some(&UniqueIdentifier::TextString(
            "replacement-certificate".to_owned()
        ))
    );
    assert!(typed.with_ttlv(|view| structure_contains_tag(&view, UNKNOWN_VENDOR_TAG)));
    assert_eq!(
        request_fields(
            captured
                .borrow()
                .as_deref()
                .expect("the fake transport captured the Re-certify request"),
            0,
        ),
        (
            Some(RECERTIFY_OPERATION),
            vec![
                UNIQUE_IDENTIFIER,
                CERTIFICATE_REQUEST_UNIQUE_IDENTIFIER,
                CERTIFICATE_REQUEST_TYPE,
                CERTIFICATE_REQUEST_VALUE,
                OFFSET,
                ATTRIBUTES,
                PROTECTION_STORAGE_MASKS,
            ],
            1,
        )
    );
    assert_eq!(
        fake.borrow().exchange_count(),
        1,
        "Re-certify is never retried"
    );
}

#[test]
fn recertify_sends_explicit_unique_identifier_without_certificate_request_fields() {
    let request = RecertifyRequest::new().with_unique_identifier(UniqueIdentifier::TextString(
        "existing-certificate".to_owned(),
    ));
    let response = asynchronous_response_bytes(
        RECERTIFY_OPERATION,
        SUCCESS,
        None,
        None,
        Some(response_payload("replacement-certificate")),
    );
    let (mut client, _fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let _ = client
        .recertify(request, &CodecLimits::defaults())
        .expect("the response fixture completes the client exchange");

    let captured = captured.borrow();
    let captured = captured
        .as_deref()
        .expect("the fake transport captured the Re-certify request");
    assert_eq!(
        request_fields(captured, 0),
        (Some(RECERTIFY_OPERATION), vec![UNIQUE_IDENTIFIER], 0),
        "the request contains the explicit Unique Identifier and omits the Certificate Request fields"
    );
    assert_eq!(
        request_unique_identifier(captured, 0),
        Some("existing-certificate".to_owned())
    );
}

#[test]
fn recertify_failure_preserves_table_302_result_without_retry() {
    let response = asynchronous_response_bytes(
        RECERTIFY_OPERATION,
        OPERATION_FAILED,
        Some(INVALID_CSR),
        None,
        None,
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let item = client
        .recertify(recertify_request(), &CodecLimits::defaults())
        .expect("Table 302 Operation Failed remains a KMIP result");
    assert_eq!(item.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        item.outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(INVALID_CSR)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn recertify_pending_preserves_correlation_without_implicit_poll() {
    let response = asynchronous_response_bytes(
        RECERTIFY_OPERATION,
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Recertify(
        recertify_request(),
    )))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("caller-enabled typed batch preserves Re-certify Pending");
    let item = response
        .get(0)
        .expect("one Re-certify result is associated");
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        item.outcome(),
        ClientOperation::Recertify,
        "Recertify(",
        "Pending(",
    );
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("the server's Operation Pending result remains Pending");
    };

    assert_eq!(pending.operation(), ClientOperation::Recertify);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn recertify_can_consume_a_preceding_successful_create_placeholder() {
    let response = operation_batch_response_bytes([
        (
            CREATE_OPERATION,
            b"create-item".to_vec(),
            test_structure([
                test_item(OBJECT_TYPE, Value::enumeration(7)),
                test_item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string("existing-certificate".to_owned()),
                ),
            ]),
        ),
        (
            RECERTIFY_OPERATION,
            b"recertify-item".to_vec(),
            response_payload("replacement-certificate"),
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
        ClientBatchItem::new(ClientRequest::Recertify(RecertifyRequest::new()))
            .with_unique_batch_item_id(b"recertify-item".to_vec()),
    ])
    .with_batch_order_option(true)
    .with_batch_error_continuation_option(1);

    client
        .execute(batch, &CodecLimits::defaults())
        .expect("Re-certify can use the preceding Create response ID Placeholder");

    let request = captured.borrow();
    let request = request
        .as_deref()
        .expect("the ordered Create and Re-certify batch reaches the fake transport");
    assert_eq!(request_fields(request, 1).0, Some(RECERTIFY_OPERATION));
    assert_eq!(request_fields(request, 1).1, Vec::<u32>::new());
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn recertify_transport_error_preserves_delivery_state_after_one_exchange() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });

    let error = client
        .recertify(recertify_request(), &CodecLimits::defaults())
        .expect_err("the fake transport deliberately fails after a partial write");

    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
