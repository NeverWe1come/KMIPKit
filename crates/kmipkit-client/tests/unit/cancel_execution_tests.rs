//! Client Cancel behavior derived from OASIS KMIP v2.1 §6.1.5, Tables 176–178,
//! and §11.7, Tables 437–438; these are not official conformance vectors.
//!
//! Traceability: KMIPKIT-0009-FR-004, FR-008, FR-009, FR-010.

use kmipkit_protocol::{CancelRequest, CancellationResult};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;

use crate::asynchronous_execution_test_support::{client_for, request_contains};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{ClientErrorCategory, ClientOperation};

const CANCEL: u32 = 0x0000_0019;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const CANCELLATION_RESULT: u32 = 0x0042_0012;
const CORRELATION: &[u8] = b"CANCEL_ASYNC_CORRELATION_SENTINEL";

#[test]
fn cancel_exposes_matching_echo_and_assigned_cancellation_result() {
    let payload = test_structure([
        test_item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(CORRELATION.to_vec()),
        ),
        test_item(CANCELLATION_RESULT, Value::enumeration(3)),
    ]);
    let response = asynchronous_response_bytes(CANCEL, 0, None, None, Some(payload));
    let (mut client, fake, request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![3, 2],
    });
    let request_model = CancelRequest::new(CORRELATION);
    assert!(!format!("{request_model:?}").contains("CANCEL_ASYNC_CORRELATION"));

    let result = client
        .execute_cancel(request_model, &CodecLimits::defaults())
        .expect("Table 177 successful Cancel response echoes the capability");

    assert_eq!(result.result().status().raw(), 0);
    assert_eq!(
        result.cancellation_result(),
        Some(CancellationResult::Completed)
    );
    assert_eq!(
        result.with_cancel_echo(<[u8]>::to_vec),
        Some(CORRELATION.to_vec())
    );
    assert!(request_contains(&request, CORRELATION));
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert!(!format!("{result:?}").contains("CANCEL_ASYNC_CORRELATION"));
}

#[test]
fn mismatched_cancel_echo_is_a_redacted_protocol_error_after_one_exchange() {
    let payload = test_structure([
        test_item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(b"different-capability".to_vec()),
        ),
        test_item(CANCELLATION_RESULT, Value::enumeration(1)),
    ]);
    let response = asynchronous_response_bytes(CANCEL, 0, None, None, Some(payload));
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let error = client
        .execute_cancel(CancelRequest::new(CORRELATION), &CodecLimits::defaults())
        .expect_err("the response echo must match the request capability");

    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert!(!error.to_string().contains("different-capability"));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn cancel_rejects_pending_even_with_a_valid_general_pending_shape() {
    let response = asynchronous_response_bytes(
        CANCEL,
        2,
        None,
        Some(b"pending-value"),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let error = client
        .execute_cancel(CancelRequest::new(CORRELATION), &CodecLimits::defaults())
        .expect_err("§6.1.5 states that Cancel responses are not asynchronous");

    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn cancel_failure_does_not_expose_an_echo_or_cancellation_result() {
    let response = asynchronous_response_bytes(CANCEL, 1, Some(1), None, None);
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let outcome = client
        .execute_cancel(CancelRequest::new(CORRELATION), &CodecLimits::defaults())
        .expect("a synchronous Cancel failure follows the general Failure shape");

    assert_eq!(outcome.operation(), ClientOperation::Cancel);
    assert_eq!(outcome.result().status().raw(), 1);
    assert_eq!(outcome.cancellation_result(), None);
    assert_eq!(outcome.with_cancel_echo(<[u8]>::len), None);
    assert_eq!(fake.borrow().exchange_count(), 1);
}
