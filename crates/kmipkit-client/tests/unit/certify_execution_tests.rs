#![cfg(test)]

//! Fake-transport Certify execution contracts from OASIS KMIP v2.1 §6.1.6,
//! Tables 179–181. Stable requirement coverage:
//! `KMIPKIT-REQ-SPEC-6.1.6-001`, `-002-001`, `-002-002`, `-002-003`,
//! `-003`, and `-007`. Shared Pending and delivery behavior follows KMIP
//! v2.1 §§8.6, 9.1, and 9.2. These are derived tests, not official OASIS
//! Test Case executions.

use kmipkit_protocol::{CertifyRequest, GetRequest, SecretBytes, UniqueIdentifier};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Item, Structure, StructureView, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const CERTIFY_OPERATION: u32 = 0x0000_0006;
const GET_OPERATION: u32 = 0x0000_000A;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CERTIFICATE_REQUEST_TYPE: u32 = 0x0042_0019;
const CERTIFICATE_REQUEST_VALUE: u32 = 0x0042_0140;
const ATTRIBUTES: u32 = 0x0042_0125;
const CERTIFICATE_TYPE: u32 = 0x0042_001D;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const INVALID_CSR: u32 = 0x0000_002F;
const PENDING_CORRELATION: &[u8] = b"CERTIFY_PENDING\x00CORRELATION\xFF";
const GENERIC_RESPONSE_BYTES: &[u8] = b"certify-generic-response-value";

fn certify_request() -> CertifyRequest {
    CertifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("public-key".to_owned()))
        .with_certificate_request_type(1)
        .with_certificate_request_value(SecretBytes::new(b"single-certificate-request".to_vec()))
        .with_attributes(certificate_type_attributes())
}

fn certificate_type_attributes() -> Structure {
    test_structure([test_item(CERTIFICATE_TYPE, Value::enumeration(1))])
}

fn response_payload(identifier: &str) -> Structure {
    test_structure([
        test_item(UNIQUE_IDENTIFIER, Value::text_string(identifier.to_owned())),
        test_item(
            UNKNOWN_VENDOR_TAG,
            Value::byte_string(GENERIC_RESPONSE_BYTES.to_vec()),
        ),
    ])
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

fn assert_certify_request(bytes: &[u8], expected_tags: &[u32]) {
    let message = decode(bytes).expect("the captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("the KMIP message is a Structure");
        };
        let mut batch_items = message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM);
        let batch_item = batch_items
            .next()
            .expect("the request contains one Batch Item");
        assert!(
            batch_items.next().is_none(),
            "the convenience call sends one batch item"
        );
        batch_item.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("the Batch Item is a Structure");
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)
                .expect("the Batch Item has an Operation")
                .with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                });
            assert_eq!(operation, Some(CERTIFY_OPERATION));

            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("the Batch Item has a Request Payload");
            let actual_tags = payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("the Request Payload is a Structure");
                };
                payload
                    .children()
                    .iter()
                    .map(|field| field.tag().raw())
                    .collect::<Vec<_>>()
            });
            assert_eq!(actual_tags.as_slice(), expected_tags);
            assert_eq!(
                actual_tags
                    .iter()
                    .filter(|tag| **tag == CERTIFICATE_REQUEST_VALUE)
                    .count(),
                1,
                "one Certify request carries one Certificate Request Value"
            );
        });
    });
}

fn assert_batch_operations_and_payload_tags(bytes: &[u8]) -> Vec<(u32, Vec<u32>)> {
    let message = decode(bytes).expect("the captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("the KMIP message is a Structure");
        };
        message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM)
            .map(|field| {
                field.with_value(|value| {
                    let ValueView::Structure(batch) = value else {
                        panic!("each Batch Item is a Structure");
                    };
                    let operation = batch
                        .children()
                        .iter()
                        .find(|field| field.tag().raw() == OPERATION)
                        .and_then(enumeration_value)
                        .expect("each Batch Item has an Enumeration Operation");
                    let payload = batch
                        .children()
                        .iter()
                        .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                        .expect("each Batch Item has a Request Payload");
                    let tags = payload.with_value(|value| {
                        let ValueView::Structure(payload) = value else {
                            panic!("each Request Payload is a Structure");
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
            .collect()
    })
}

fn enumeration_value(item: &Item) -> Option<u32> {
    item.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    })
}

#[test]
fn certify_sends_table_179_request_once_and_exposes_identifier_and_generic_response() {
    crate::execute_test_support::assert_lifecycle_request_debug(
        &crate::ClientRequest::certify(certify_request()),
        "Certify",
    );
    let response = asynchronous_response_bytes(
        CERTIFY_OPERATION,
        SUCCESS,
        None,
        None,
        Some(response_payload("generated-certificate")),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![5, 9],
    });

    let item = client
        .certify(certify_request(), &CodecLimits::defaults())
        .expect("the Table 180 Certify success response is valid");
    let outcome = item.outcome();
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        outcome,
        ClientOperation::Certify,
        "Certify(",
        "Certify(",
    );
    let typed = outcome
        .response()
        .certify()
        .expect("the result retains its typed Certify response");

    assert_eq!(outcome.operation(), ClientOperation::Certify);
    assert_eq!(outcome.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.unique_identifier(),
        Some(&UniqueIdentifier::TextString(
            "generated-certificate".to_owned()
        ))
    );
    assert_eq!(
        typed.with_ttlv(|view| structure_contains_tag(&view, UNKNOWN_VENDOR_TAG)),
        Some(true)
    );
    assert_certify_request(
        captured
            .borrow()
            .as_deref()
            .expect("the fake transport captured the Certify request"),
        &[
            UNIQUE_IDENTIFIER,
            CERTIFICATE_REQUEST_TYPE,
            CERTIFICATE_REQUEST_VALUE,
            ATTRIBUTES,
        ],
    );
    assert_eq!(
        fake.borrow().exchange_count(),
        1,
        "Certify is never retried"
    );
}

#[test]
fn certify_failure_preserves_table_181_result_without_retry() {
    let response = asynchronous_response_bytes(
        CERTIFY_OPERATION,
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
        .certify(certify_request(), &CodecLimits::defaults())
        .expect("a valid Table 181 Operation Failed response remains a KMIP result");
    assert_eq!(item.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        item.outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(INVALID_CSR)
    );
    assert!(
        item.outcome()
            .response()
            .certify()
            .is_some_and(|response| response.with_ttlv(|_| ()).is_none())
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn certify_pending_preserves_exact_correlation_without_poll_or_retry() {
    let response = asynchronous_response_bytes(
        CERTIFY_OPERATION,
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Certify(
        certify_request(),
    )))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("caller-enabled typed batch preserves Certify Pending");
    let item = response.get(0).expect("one Certify result is associated");
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        item.outcome(),
        ClientOperation::Certify,
        "Certify(",
        "Pending(",
    );
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("the server's Operation Pending result remains Pending");
    };

    assert_eq!(pending.operation(), ClientOperation::Certify);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(
        fake.borrow().exchange_count(),
        1,
        "no implicit Poll or retry occurs"
    );
}

#[test]
fn certify_transport_error_preserves_delivery_state_after_one_exchange() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });

    let error = client
        .certify(certify_request(), &CodecLimits::defaults())
        .expect_err("the fake transport deliberately fails after part of the request is written");

    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(
        fake.borrow().exchange_count(),
        1,
        "possibly-sent requests are not retried"
    );
}

#[test]
fn certify_can_produce_the_placeholder_for_a_get_in_the_same_batch() {
    let (mut client, fake, captured) = client_for(ExchangeScript::FailBeforeWrite);
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Certify(
            CertifyRequest::new()
                .with_certificate_request_type(1)
                .with_certificate_request_value(SecretBytes::new(b"request".to_vec())),
        ))
        .with_unique_batch_item_id(b"certify".to_vec()),
        ClientBatchItem::new(ClientRequest::Get(GetRequest::new()))
            .with_unique_batch_item_id(b"get".to_vec()),
    ]);

    let error = client
        .execute(batch, &CodecLimits::defaults())
        .expect_err("the fake transport stops the exchange before any bytes are sent");

    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let captured_request = captured.borrow();
    let request = captured_request
        .as_deref()
        .expect("the caller's two-operation batch reaches the fake transport");
    assert_eq!(
        assert_batch_operations_and_payload_tags(request),
        vec![
            (
                CERTIFY_OPERATION,
                vec![CERTIFICATE_REQUEST_TYPE, CERTIFICATE_REQUEST_VALUE]
            ),
            (GET_OPERATION, vec![]),
        ],
        "§6.1.6 permits a later Get with omitted Unique Identifier to use Certify's ID Placeholder"
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
