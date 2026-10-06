//! Client Poll behavior derived from OASIS KMIP v2.1 §6.1.38, Table 276, and
//! §8.6, Table 399; these are not official conformance vectors.
//!
//! Traceability: KMIPKIT-0009-FR-002, FR-003, FR-008, FR-009, FR-010.

use kmipkit_protocol::PollRequest;
use kmipkit_test_support::{ExchangeScript, ResponseDropObserver};
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;

use crate::ClientErrorCategory;
use crate::asynchronous_execution_test_support::{
    client_for, client_for_with_response_observer, request_contains,
};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};

const POLL: u32 = 0x0000_001A;
const CORRELATION: &[u8] = b"POLL_ASYNC_CORRELATION_SENTINEL";
const EXTENSION_TAG: u32 = 0x0042_0012;

#[test]
fn pending_poll_returns_without_repeating_and_zeroizes_request_and_response_copies() {
    let response = asynchronous_response_bytes(POLL, 2, None, Some(CORRELATION), None);
    let response_observer = ResponseDropObserver::new(response.len());
    let script = ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 3],
    };
    let (mut client, shared_fake, captured_request) =
        client_for_with_response_observer(script, response_observer.clone());

    let result = client.execute_poll(PollRequest::new(CORRELATION), &CodecLimits::defaults());
    let outcome = result.expect("§6.1.38 allows the original operation to remain Pending");

    assert_eq!(outcome.result().status().raw(), 2);
    assert!(outcome.is_pending());
    assert_eq!(
        outcome.with_asynchronous_correlation_value(<[u8]>::to_vec),
        Some(CORRELATION.to_vec())
    );
    assert_eq!(
        outcome.with_response_payload(|payload| payload.children().len()),
        None
    );
    assert_eq!(shared_fake.borrow().exchange_count(), 1);
    assert!(request_contains(&captured_request, CORRELATION));
    assert!(!format!("{outcome:?}").contains("POLL_ASYNC_CORRELATION"));
    assert!(response_observer.initialized_bytes_were_zeroized());
}

#[test]
fn completed_poll_keeps_original_payload_generic_and_failure_has_no_payload() {
    let success = asynchronous_response_bytes(
        POLL,
        0,
        None,
        None,
        Some(test_structure([test_item(
            EXTENSION_TAG,
            Value::byte_string(b"opaque-original-operation".to_vec()),
        )])),
    );
    let (mut client, success_fake, _) = client_for(ExchangeScript::Success {
        response: success,
        request_write_chunks: Vec::new(),
    });
    let outcome = client
        .execute_poll(PollRequest::new(CORRELATION), &CodecLimits::defaults())
        .expect("Poll success carries an opaque original-operation payload");
    assert_eq!(outcome.result().status().raw(), 0);
    assert_eq!(
        outcome.with_response_payload(|payload| { payload.children()[0].tag().raw() }),
        Some(EXTENSION_TAG)
    );
    assert_eq!(success_fake.borrow().exchange_count(), 1);

    let failure = asynchronous_response_bytes(POLL, 1, Some(1), None, None);
    let (mut client, failure_fake, _) = client_for(ExchangeScript::Success {
        response: failure,
        request_write_chunks: Vec::new(),
    });
    let outcome = client
        .execute_poll(PollRequest::new(CORRELATION), &CodecLimits::defaults())
        .expect("terminal Poll Failure uses the general failure response shape");
    assert_eq!(outcome.result().status().raw(), 1);
    assert_eq!(
        outcome
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(1)
    );
    assert_eq!(outcome.with_response_payload(|_| ()), None);
    assert_eq!(failure_fake.borrow().exchange_count(), 1);
}

#[test]
fn failed_poll_preserves_delivery_state_and_does_not_retry() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 2 });
    let error = client
        .execute_poll(PollRequest::new(CORRELATION), &CodecLimits::defaults())
        .expect_err("transport failure ends the one explicit Poll call");

    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
