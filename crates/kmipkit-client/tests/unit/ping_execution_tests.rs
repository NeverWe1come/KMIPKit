//! Fake-transport Ping behavior derived from OASIS KMIP v2.1 §6.1.36,
//! Tables 271–272. These tests are not official fixture passes.

use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::ValueView;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{ClientBatchOutcome, ClientOperation};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};

const PING: u32 = 0x0000_003b;
const BATCH_ITEM: u32 = 0x0042_000f;
const OPERATION: u32 = 0x0042_005c;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;

fn request_shape(request: &[u8]) -> Option<(u32, usize)> {
    let decoded = decode_with_limits(request, &CodecLimits::defaults()).ok()?;
    decoded.with_value(|value| {
        let ValueView::Structure(root) = value else {
            return None;
        };
        let batch = root
            .children()
            .iter()
            .find(|field| field.tag().raw() == BATCH_ITEM)?;
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                return None;
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)?
                .with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                })?;
            let payload_len = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)?
                .with_value(|value| match value {
                    ValueView::Structure(payload) => Some(payload.children().len()),
                    _ => None,
                })?;
            Some((operation, payload_len))
        })
    })
}

#[test]
fn ping_sends_one_empty_payload_and_returns_success() {
    let response = asynchronous_response_bytes(PING, 0, None, None, Some(test_structure([])));
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });

    let outcome = client
        .ping(&CodecLimits::defaults())
        .expect("the fake server returns a successful empty Ping response");
    let ClientBatchOutcome::Ping(ping) = outcome.outcome() else {
        panic!("the typed outcome is Ping");
    };

    assert_eq!(ping.result().status().raw(), 0);
    assert_eq!(outcome.outcome().operation(), ClientOperation::Ping);
    assert_eq!(outcome.outcome().result().status().raw(), 0);
    let response = outcome.outcome().response();
    assert!(response.ping().is_some());
    assert!(response.query().is_none());
    assert_eq!(response.result().status().raw(), 0);
    assert_eq!(
        request_shape(captured.borrow().as_deref().expect("request was captured")),
        Some((PING, 0))
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn ping_failure_preserves_transport_delivery_and_does_not_retry() {
    let (mut client, transport, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });

    let error = client
        .ping(&CodecLimits::defaults())
        .expect_err("the scripted transport fails after a partial write");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn malformed_ping_response_is_rejected_after_one_exchange() {
    let malformed_payload = test_structure([test_item(
        0x0042_0012,
        kmipkit_ttlv::Value::text_string("unexpected".to_owned()),
    )]);
    let response = asynchronous_response_bytes(PING, 0, None, None, Some(malformed_payload));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let error = client
        .ping(&CodecLimits::defaults())
        .expect_err("Ping success has an empty response payload");

    assert!(error.delivery_state().is_some());
    assert_eq!(transport.borrow().exchange_count(), 1);
}
