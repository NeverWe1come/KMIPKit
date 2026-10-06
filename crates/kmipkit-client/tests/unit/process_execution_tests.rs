//! Client Process behavior derived from OASIS KMIP v2.1 §6.1.39, Tables 278–280,
//! and §8.6, Table 399; these are not official conformance vectors.
//!
//! Traceability: KMIPKIT-0009-FR-005, FR-008, FR-009, FR-010. The missing
//! catalog requirement ID for Table 278 remains open under OD-002.

use crate::asynchronous_execution_test_support::{client_for, request_contains};
use crate::execute_test_support::{asynchronous_response_bytes, test_structure};
use kmipkit_protocol::ProcessRequest;
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::codec::CodecLimits;

const PROCESS: u32 = 0x0000_003A;
const CORRELATION: &[u8] = b"PROCESS_ASYNC_CORRELATION_SENTINEL";

#[test]
fn process_pending_is_caller_selected_and_performs_one_exchange() {
    let response = asynchronous_response_bytes(
        PROCESS,
        2,
        None,
        Some(CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let result = client
        .execute_process(
            ProcessRequest::new(CORRELATION),
            Some(2),
            &CodecLimits::defaults(),
        )
        .expect("the caller selected an Asynchronous Indicator that permits Pending");

    assert!(result.is_pending());
    assert_eq!(result.result().status().raw(), 2);
    assert_eq!(result.payload_member_count(), Some(0));
    assert_eq!(
        result.with_asynchronous_correlation_value(<[u8]>::to_vec),
        Some(CORRELATION.to_vec())
    );
    assert!(request_contains(&request, CORRELATION));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn process_success_requires_the_empty_table_279_payload() {
    let response = asynchronous_response_bytes(PROCESS, 0, None, None, Some(test_structure([])));
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let result = client
        .execute_process(
            ProcessRequest::new(CORRELATION),
            None,
            &CodecLimits::defaults(),
        )
        .expect("successful Process has an empty response payload");

    assert!(!result.is_pending());
    assert_eq!(result.result().status().raw(), 0);
    assert_eq!(result.payload_member_count(), Some(0));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn process_rejects_pending_when_the_request_did_not_permit_it() {
    let response = asynchronous_response_bytes(
        PROCESS,
        2,
        None,
        Some(CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let error = client
        .execute_process(
            ProcessRequest::new(CORRELATION),
            None,
            &CodecLimits::defaults(),
        )
        .expect_err("Pending requires a caller-selected permissive indicator");
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::ResponseStarted)
    );
}
