//! Source-derived fake-transport contracts for OASIS KMIP v2.1 §6.1.12,
//! Tables 199–201. These tests do not claim official fixture execution.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;

use kmipkit_protocol::DelegatedLoginRequest;
use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Structure, Value, ValueView};
use zeroize::Zeroizing;

use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{
    Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};

const DELEGATED_LOGIN: u32 = 0x0000_002F;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const PENDING: u32 = 2;
const PERMISSION_DENIED: u32 = 0x0000_000C;
const TICKET: u32 = 0x0042_0149;
const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;
const RIGHT: u32 = 0x0042_014E;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;
const SECRET_SENTINEL: &[u8] = b"delegated-login-ticket-secret";

#[derive(Default)]
struct ExchangeState {
    requests: Vec<Zeroizing<Vec<u8>>>,
    responses: VecDeque<Vec<u8>>,
    calls: usize,
    failure: Option<RequestDeliveryState>,
}

struct DelegatedLoginTransport(Rc<RefCell<ExchangeState>>);

impl Transport for DelegatedLoginTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        let mut state = self.0.borrow_mut();
        state.calls += 1;
        state.requests.push(Zeroizing::new(request.to_vec()));
        if let Some(delivery_state) = state.failure {
            return Err(TransportError::new(
                delivery_state,
                TransportCauseCategory::Io,
                io::Error::other("test transport failure"),
            ));
        }
        let response = state.responses.pop_front().ok_or_else(|| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("unexpected additional exchange"),
            )
        })?;
        Ok(TransportResponse::new(response))
    }
}

fn client(responses: Vec<Vec<u8>>) -> (Client, Rc<RefCell<ExchangeState>>) {
    let state = Rc::new(RefCell::new(ExchangeState {
        responses: VecDeque::from(responses),
        ..ExchangeState::default()
    }));
    (
        Client::for_test(DelegatedLoginTransport(Rc::clone(&state))),
        state,
    )
}

fn request() -> ClientRequest {
    let rights = test_structure([test_item(RIGHT, Value::structure(Structure::new()))]);
    ClientRequest::delegated_login(DelegatedLoginRequest::new(rights))
}

fn response(
    status: u32,
    reason: Option<u32>,
    correlation: Option<&[u8]>,
    payload: Option<Structure>,
) -> Vec<u8> {
    asynchronous_response_bytes(DELEGATED_LOGIN, status, reason, correlation, payload)
}

fn ticket_payload() -> Structure {
    test_structure([test_item(
        TICKET,
        Value::structure(test_structure([
            test_item(TICKET_TYPE, Value::enumeration(0x8000_0042)),
            test_item(TICKET_VALUE, Value::byte_string(SECRET_SENTINEL.to_vec())),
            test_item(UNKNOWN_VENDOR_TAG, Value::enumeration(0xF123_4567)),
        ])),
    )])
}

fn structure_contains_tag(tree: &kmipkit_ttlv::StructureView<'_>, raw_tag: u32) -> bool {
    tree.children().iter().any(|item| {
        item.tag().raw() == raw_tag
            || item.with_value(|value| match value {
                ValueView::Structure(nested) => structure_contains_tag(&nested, raw_tag),
                _ => false,
            })
    })
}

fn request_contains_tag(request: &[u8], raw_tag: u32) -> bool {
    let decoded = decode_with_limits(request, &CodecLimits::defaults())
        .expect("captured request is valid TTLV");
    decoded.with_value(|value| match value {
        ValueView::Structure(root) => structure_contains_tag(&root, raw_tag),
        _ => false,
    })
}

#[test]
fn delegated_login_sends_one_typed_request_and_exposes_secret_safe_ticket() {
    let (mut client, state) = client(vec![response(SUCCESS, None, None, Some(ticket_payload()))]);
    let result = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect("the server success result contains a valid Ticket");
    let item = result.items.first().expect("one response item");
    let typed = item
        .outcome()
        .delegated_login_response()
        .expect("the explicit request returns a typed Delegated Login result");
    let ticket = typed.ticket().expect("success exposes its Ticket");
    ticket.with_ticket_value(|actual| assert_eq!(actual, SECRET_SENTINEL));

    assert_eq!(item.outcome().operation(), ClientOperation::DelegatedLogin);
    assert!(format!("{:?}", item.outcome()).contains("DelegatedLogin"));
    assert!(format!("{:?}", item.outcome().response()).contains("DelegatedLogin"));
    assert!(format!("{}", item.outcome()).contains("DelegatedLogin("));
    assert_eq!(state.borrow().calls, 1, "a KMIP request is never retried");
    assert_eq!(state.borrow().requests.len(), 1);
    let debug = format!("{result:?}");
    assert!(!debug.contains("delegated-login-ticket-secret"));
}

#[test]
fn pending_result_keeps_exact_correlation_without_implicit_poll() {
    let correlation = [0xA1, 0x00, 0xB2, 0xFF];
    let pending_payload = test_structure([test_item(
        UNKNOWN_VENDOR_TAG,
        Value::byte_string(b"pending opaque payload".to_vec()),
    )]);
    let (mut client, state) = client(vec![response(
        PENDING,
        None,
        Some(&correlation),
        Some(pending_payload),
    )]);
    let result = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())).with_asynchronous_indicator(1),
            &CodecLimits::defaults(),
        )
        .expect("Pending is returned as an explicit caller-controlled outcome");
    let item = result.items.first().expect("one response item");
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("Pending remains an explicit outcome");
    };

    assert_eq!(pending.operation(), ClientOperation::DelegatedLogin);
    assert_eq!(pending.asynchronous_correlation_value(), correlation);
    assert_eq!(
        state.borrow().calls,
        1,
        "the client does not Poll implicitly"
    );
}

#[test]
fn operation_failure_preserves_table_201_reason_and_does_not_retry() {
    let (mut client, state) = client(vec![response(
        OPERATION_FAILED,
        Some(PERMISSION_DENIED),
        None,
        None,
    )]);
    let result = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect("operation failure is a KMIP result, not a transport error");
    let outcome = result.items.first().expect("one response item").outcome();

    assert_eq!(outcome.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        outcome
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(PERMISSION_DENIED)
    );
    assert_eq!(state.borrow().calls, 1);
}

#[test]
fn malformed_ticket_response_is_sanitized_after_one_exchange() {
    let malformed = test_structure([test_item(
        TICKET,
        Value::structure(test_structure([test_item(
            TICKET_TYPE,
            Value::enumeration(1),
        )])),
    )]);
    let (mut client, state) = client(vec![response(SUCCESS, None, None, Some(malformed))]);
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect_err("a malformed successful Ticket response is rejected");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert!(!error.to_string().contains("delegated-login-ticket-secret"));
    assert_eq!(state.borrow().calls, 1);
}

#[test]
fn returned_ticket_does_not_mutate_the_next_request_authentication_state() {
    let (mut client, state) = client(vec![
        response(SUCCESS, None, None, Some(ticket_payload())),
        asynchronous_response_bytes(0x0000_003B, SUCCESS, None, None, Some(test_structure([]))),
    ]);
    client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect("Delegated Login returns the Ticket to the caller");
    client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::ping())),
            &CodecLimits::defaults(),
        )
        .expect("a later Ping is an independent explicit request");

    let state = state.borrow();
    assert_eq!(state.calls, 2);
    assert_eq!(state.requests.len(), 2);
    assert!(!request_contains_tag(&state.requests[1], TICKET));
    assert!(
        !state.requests[1]
            .windows(SECRET_SENTINEL.len())
            .any(|window| window == SECRET_SENTINEL)
    );
}

#[test]
fn possibly_sent_transport_failure_is_reported_without_retry() {
    let state = Rc::new(RefCell::new(ExchangeState {
        failure: Some(RequestDeliveryState::PossiblySent),
        ..ExchangeState::default()
    }));
    let mut client = Client::for_test(DelegatedLoginTransport(Rc::clone(&state)));
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect_err("transport failure exposes delivery evidence");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(state.borrow().calls, 1);
}
