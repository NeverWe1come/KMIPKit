//! Fake-transport Logout contracts from OASIS KMIP v2.1 §6.1.31,
//! Tables 256–258. These are project-derived tests, not official OASIS cases.

use kmipkit_protocol::{LogoutRequest, SecretBytes, Ticket, TicketType};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::CodecLimits;

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};

const LOGOUT: u32 = 0x0000_002E;
const SUCCESS: u32 = 0;
const FAILURE: u32 = 1;
const PENDING: u32 = 2;
const INVALID_TICKET: u32 = 0x0000_000D;
const TICKET: u32 = 0x0042_0149;
const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;
const SECRET_SENTINEL: &[u8] = b"logout-ticket-secret-sentinel";

fn request() -> ClientRequest {
    let ticket = Ticket::new(
        TicketType::LOGIN,
        SecretBytes::new(SECRET_SENTINEL.to_vec()),
    )
    .expect("the explicit Ticket is structurally valid");
    ClientRequest::logout(LogoutRequest::new(ticket))
}

fn response(
    status: u32,
    reason: Option<u32>,
    correlation: Option<&[u8]>,
    payload: Option<kmipkit_ttlv::Structure>,
) -> Vec<u8> {
    asynchronous_response_bytes(LOGOUT, status, reason, correlation, payload)
}

#[test]
fn logout_sends_one_ticket_and_exposes_empty_success_without_leaking_it() {
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response: response(SUCCESS, None, None, Some(test_structure([]))),
        request_write_chunks: vec![2, 4],
    });
    let outcome = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect("the server returns empty Table 257 success");
    let item = outcome.get(0).expect("one Logout result exists");
    assert_eq!(item.outcome().operation(), ClientOperation::Logout);
    let response = item
        .outcome()
        .logout_response()
        .expect("typed Logout response is available");
    assert_eq!(response.result().status().raw(), SUCCESS);
    assert!(!format!("{outcome:?}").contains("logout-ticket-secret-sentinel"));

    let request_bytes = captured.borrow();
    let request_bytes = request_bytes.as_deref().expect("request captured");
    assert!(
        request_bytes
            .windows(SECRET_SENTINEL.len())
            .any(|w| w == SECRET_SENTINEL)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn table_258_failure_preserves_reason_without_retry() {
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response: response(FAILURE, Some(INVALID_TICKET), None, None),
        request_write_chunks: Vec::new(),
    });
    let outcome = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect("Operation Failed remains a server result");
    let result = outcome.get(0).expect("one Logout result exists").outcome();
    assert_eq!(result.result().status().raw(), FAILURE);
    assert_eq!(
        result.result().reason().map(|reason| reason.raw()),
        Some(INVALID_TICKET)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn pending_preserves_correlation_without_implicit_poll() {
    const CORRELATION: &[u8] = b"logout-pending-correlation";
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response: response(PENDING, None, Some(CORRELATION), Some(test_structure([]))),
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(request())).with_asynchronous_indicator(1);
    let outcome = client
        .execute(batch, &CodecLimits::defaults())
        .expect("caller explicitly opted into asynchronous responses");
    let ClientBatchOutcome::Pending(pending) = outcome.get(0).expect("one result").outcome() else {
        panic!("Logout Pending remains explicit and resumable");
    };
    assert_eq!(pending.operation(), ClientOperation::Logout);
    assert_eq!(pending.asynchronous_correlation_value(), CORRELATION);
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn malformed_success_payload_is_redacted_after_one_exchange() {
    let malformed = test_structure([test_item(TICKET_TYPE, kmipkit_ttlv::Value::enumeration(1))]);
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response: response(SUCCESS, None, None, Some(malformed)),
        request_write_chunks: Vec::new(),
    });
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect_err("Table 257 success payload must be empty");
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert!(!error.to_string().contains("logout-ticket-secret-sentinel"));
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn local_codec_limit_failure_is_not_sent_and_never_retries() {
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response: response(SUCCESS, None, None, Some(test_structure([]))),
        request_write_chunks: Vec::new(),
    });
    let limits = CodecLimits::new(1024, 8, 1).expect("positive codec limits");
    let error = client
        .execute(ClientBatch::new(ClientBatchItem::new(request())), &limits)
        .expect_err("the local message exceeds the caller's element limit");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
}
