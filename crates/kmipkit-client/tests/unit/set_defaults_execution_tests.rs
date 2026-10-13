//! Project-derived execution tests: OASIS KMIP v2.1 §6.1.53, Tables 328–330.
use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest};
use kmipkit_protocol::SetDefaultsRequest;
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Value, ValueView};

#[test]
fn set_defaults_one_exchange_returns_typed_success() {
    let response = asynchronous_response_bytes(54, 0, None, None, Some(test_structure([])));
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });
    let response = client
        .set_defaults(SetDefaultsRequest::new(None), &CodecLimits::defaults())
        .expect("success");
    assert_eq!(response.outcome().operation(), ClientOperation::SetDefaults);
    assert_eq!(response.outcome().result().status().raw(), 0);
    assert!(response.outcome().response().set_defaults().is_some());
    let decoded = decode_with_limits(
        captured.borrow().as_deref().expect("captured"),
        &CodecLimits::defaults(),
    )
    .expect("TTLV");
    decoded.with_value(|v| {
        let ValueView::Structure(root) = v else {
            panic!("root")
        };
        let batch = root
            .children()
            .iter()
            .find(|f| f.tag().raw() == 0x42000f)
            .expect("batch");
        batch.with_value(|v| {
            let ValueView::Structure(batch) = v else {
                panic!("batch")
            };
            assert!(batch.children().iter().any(|f| f.tag().raw() == 0x42005c
                && f.with_value(|v| matches!(v,ValueView::Enumeration(v) if *v==54))));
        });
    });
    assert_eq!(transport.borrow().exchange_count(), 1);
}
#[test]
fn set_defaults_delivery_is_preserved_without_retry() {
    for (script, delivery) in [
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
                response_bytes: vec![1],
            },
            RequestDeliveryState::ResponseStarted,
        ),
    ] {
        let (mut client, transport, _) = client_for(script);
        let error = client
            .set_defaults(SetDefaultsRequest::new(None), &CodecLimits::defaults())
            .expect_err("transport error");
        assert_eq!(error.delivery_state(), Some(delivery));
        assert_eq!(transport.borrow().exchange_count(), 1);
    }
}
#[test]
fn set_defaults_pending_retains_correlation_and_response_without_poll() {
    let response = asynchronous_response_bytes(
        54,
        2,
        None,
        Some(b"policy-pending"),
        Some(test_structure([])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![],
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::SetDefaults(
        SetDefaultsRequest::new(None),
    )))
    .with_asynchronous_indicator(1);
    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("Pending");
    let ClientBatchOutcome::Pending(pending) = response.get(0).expect("one").outcome() else {
        panic!("Pending")
    };
    assert_eq!(pending.operation(), ClientOperation::SetDefaults);
    assert_eq!(pending.asynchronous_correlation_value(), b"policy-pending");
    assert!(pending.response().set_defaults().is_some());
    assert_eq!(transport.borrow().exchange_count(), 1);
}
#[test]
fn set_defaults_server_failure_remains_a_result() {
    let response = asynchronous_response_bytes(54, 1, Some(0x80000001), None, None);
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![],
    });
    let response = client
        .set_defaults(SetDefaultsRequest::new(None), &CodecLimits::defaults())
        .expect("server result");
    assert_eq!(
        response
            .outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(0x80000001)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}
#[test]
fn set_defaults_malformed_response_retains_response_started() {
    let response = asynchronous_response_bytes(
        54,
        0,
        None,
        None,
        Some(test_structure([test_item(0x420012, Value::integer(1))])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![],
    });
    let error = client
        .set_defaults(SetDefaultsRequest::new(None), &CodecLimits::defaults())
        .expect_err("malformed payload");
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert!(error.response_ttlv().is_some());
    assert_eq!(transport.borrow().exchange_count(), 1);
}
