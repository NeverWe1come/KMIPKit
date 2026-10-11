//! Fake-transport Log behavior derived from OASIS KMIP v2.1 §6.1.29,
//! Tables 250–252. These tests are not official fixture passes.

use kmipkit_protocol::{LogRequest, SecretText};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::ValueView;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};

const LOG: u32 = 0x0000_002c;
const BATCH_ITEM: u32 = 0x0042_000f;
const OPERATION: u32 = 0x0042_005c;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;

fn request() -> LogRequest {
    LogRequest::new(SecretText::new("log-private-sentinel".to_owned()))
}

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
fn log_sends_required_fields_and_returns_empty_success() {
    let response = asynchronous_response_bytes(LOG, 0, None, None, Some(test_structure([])));
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });

    let outcome = client
        .log(request(), &CodecLimits::defaults())
        .expect("the fake server returns a successful empty Log response");
    let ClientBatchOutcome::Log(log) = outcome.outcome() else {
        panic!("the typed outcome is Log");
    };

    assert_eq!(log.result().status().raw(), 0);
    assert_eq!(outcome.outcome().operation(), ClientOperation::Log);
    assert_eq!(outcome.outcome().result().status().raw(), 0);
    let response = outcome.outcome().response();
    assert!(response.log().is_some());
    assert!(response.query().is_none());
    assert_eq!(response.result().status().raw(), 0);
    assert_eq!(
        request_shape(captured.borrow().as_deref().expect("request was captured")),
        Some((LOG, 1))
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn log_failure_preserves_transport_delivery_and_does_not_retry() {
    let (mut client, transport, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });

    let error = client
        .log(request(), &CodecLimits::defaults())
        .expect_err("the scripted transport fails after a partial write");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn log_pending_preserves_typed_response_and_correlation_without_follow_up() {
    const CORRELATION: &[u8] = b"log-pending-correlation";
    let response = asynchronous_response_bytes(
        LOG,
        2,
        None,
        Some(CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::log(request())))
        .with_asynchronous_indicator(1);

    let result = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the request permits an asynchronous Pending response");
    let outcome = result.get(0).expect("one result exists").outcome();
    let ClientBatchOutcome::Pending(pending) = outcome else {
        panic!("Log Pending remains explicitly resumable");
    };

    assert_eq!(pending.operation(), ClientOperation::Log);
    assert_eq!(pending.result().status().raw(), 2);
    assert_eq!(pending.asynchronous_correlation_value(), CORRELATION);
    assert_eq!(
        pending
            .response()
            .log()
            .map(|response| response.result().status().raw()),
        Some(2)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn malformed_log_response_is_rejected_after_one_exchange() {
    let malformed_payload = test_structure([test_item(
        0x0042_0012,
        kmipkit_ttlv::Value::text_string("unexpected".to_owned()),
    )]);
    let response = asynchronous_response_bytes(LOG, 0, None, None, Some(malformed_payload));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let error = client
        .log(request(), &CodecLimits::defaults())
        .expect_err("Log success has an empty response payload");

    assert!(error.delivery_state().is_some());
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn failure_reason_message_and_diagnostics_are_lossless_and_redacted() {
    let response = crate::execute_test_support::one_item_response_bytes(
        LOG,
        1,
        Some(0xf1234567),
        Some("log-private-sentinel"),
        None,
        None,
        None,
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let outcome = client.log(request(), &CodecLimits::defaults()).unwrap();
    assert_eq!(
        outcome.outcome().result().reason().unwrap().raw(),
        0xf1234567
    );
    assert_eq!(
        outcome.outcome().result().message().unwrap().as_str(),
        "log-private-sentinel"
    );
    assert!(
        !format!(
            "{:?} {} {:?}",
            outcome.outcome(),
            outcome.outcome(),
            outcome.outcome().response()
        )
        .contains("log-private-sentinel")
    );
    assert!(
        !format!("{:?}", ClientRequest::log(request())).contains("log-private-sentinel")
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn every_transport_delivery_state_is_preserved_without_retry() {
    for (script, expected) in [
        (
            ExchangeScript::FailBeforeWrite,
            RequestDeliveryState::NotSent,
        ),
        (
            ExchangeScript::FailAfterPartialWrite { written_bytes: 1 },
            RequestDeliveryState::PossiblySent,
        ),
        (
            ExchangeScript::FailAfterPartialRead {
                written_bytes: 1,
                response_bytes: b"private".to_vec(),
            },
            RequestDeliveryState::ResponseStarted,
        ),
    ] {
        let (mut client, transport, _) = client_for(script);
        let error = client
            .log(request(), &CodecLimits::defaults())
            .unwrap_err();
        assert_eq!(error.delivery_state(), Some(expected));
        assert!(!format!("{error:?} {error}").contains("log-private-sentinel"));
        assert_eq!(transport.borrow().exchange_count(), 1);
    }
}

#[test]
fn codec_failure_is_not_sent_and_operation_mismatch_is_response_started() {
    let response = asynchronous_response_bytes(LOG, 0, None, None, Some(test_structure([])));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let error = client
        .log(request(), &CodecLimits::new(1024, 8, 1).unwrap())
        .unwrap_err();
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
    let response = asynchronous_response_bytes(0x3b, 0, None, None, Some(test_structure([])));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let error = client
        .log(request(), &CodecLimits::defaults())
        .unwrap_err();
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}
