//! Fake-transport Login behavior from OASIS KMIP v2.1 §6.1.30,
//! Tables 253–255. The request values are the pinned CN01 §2.48 fixture values.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;

use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use kmipkit_ttlv::ValueView;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use zeroize::Zeroizing;

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use kmipkit_protocol::LoginRequest;

const LOGIN: u32 = 0x0000_002D;
const SUCCESS: u32 = 0;
const FAILURE: u32 = 1;
const PENDING: u32 = 2;
const PERMISSION_DENIED: u32 = 5;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const LEASE_TIME: u32 = 0x0042_0049;
const REQUEST_COUNT: u32 = 0x0042_014C;
const TICKET: u32 = 0x0042_0149;
const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;
const SECRET_SENTINEL: &[u8] = b"login-ticket-secret-sentinel";
const PING: u32 = 0x0000_003B;

fn response(
    status: u32,
    reason: Option<u32>,
    correlation: Option<&[u8]>,
    payload: Option<kmipkit_ttlv::Structure>,
) -> Vec<u8> {
    asynchronous_response_bytes(LOGIN, status, reason, correlation, payload)
}

fn ticket_payload() -> kmipkit_ttlv::Structure {
    test_structure([test_item(
        TICKET,
        kmipkit_ttlv::Value::structure(test_structure([
            test_item(TICKET_TYPE, kmipkit_ttlv::Value::enumeration(1)),
            test_item(
                TICKET_VALUE,
                kmipkit_ttlv::Value::byte_string(SECRET_SENTINEL.to_vec()),
            ),
        ])),
    )])
}

fn fixture_request() -> ClientRequest {
    ClientRequest::login(
        LoginRequest::new()
            .with_lease_time_interval(0)
            .with_request_count(3),
    )
}

fn operation_and_payload_tags(request: &[u8]) -> Option<(u32, Vec<u32>)> {
    const RESULT_BATCH_ITEM: u32 = 0x0042_000F;
    let decoded = decode_with_limits(request, &CodecLimits::defaults()).ok()?;
    decoded.with_value(|value| {
        let ValueView::Structure(root) = value else {
            return None;
        };
        let batch = root
            .children()
            .iter()
            .find(|field| field.tag().raw() == RESULT_BATCH_ITEM)?;
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                return None;
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)?
                .with_value(|value| match value {
                    ValueView::Enumeration(raw) => Some(*raw),
                    _ => None,
                })?;
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)?
                .with_value(|value| match value {
                    ValueView::Structure(payload) => Some(
                        payload
                            .children()
                            .iter()
                            .map(|field| field.tag().raw())
                            .collect(),
                    ),
                    _ => None,
                })?;
            Some((operation, payload))
        })
    })
}

#[derive(Default)]
struct SequenceState {
    calls: usize,
    requests: Vec<Zeroizing<Vec<u8>>>,
    responses: VecDeque<Vec<u8>>,
}

struct SequenceTransport(Rc<RefCell<SequenceState>>);

impl Transport for SequenceTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<kmipkit_transport::TransportResponse, kmipkit_transport::TransportError> {
        let mut state = self.0.borrow_mut();
        state.calls += 1;
        state.requests.push(Zeroizing::new(request.to_vec()));
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

fn sequence_client(responses: Vec<Vec<u8>>) -> (crate::Client, Rc<RefCell<SequenceState>>) {
    let state = Rc::new(RefCell::new(SequenceState {
        responses: VecDeque::from(responses),
        ..SequenceState::default()
    }));
    (
        crate::Client::for_test(SequenceTransport(Rc::clone(&state))),
        state,
    )
}

fn structure_contains_tag(tree: &kmipkit_ttlv::StructureView<'_>, raw_tag: u32) -> bool {
    tree.children().iter().any(|field| {
        field.tag().raw() == raw_tag
            || field.with_value(|value| match value {
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
fn login_matches_pinned_fixture_request_and_returns_one_secret_safe_ticket() {
    let response = response(SUCCESS, None, None, Some(ticket_payload()));
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });
    let outcome = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(fixture_request())),
            &CodecLimits::defaults(),
        )
        .expect("the server returns the Login fixture success shape");
    let item = outcome.get(0).expect("one Login result exists");
    assert_eq!(item.outcome().operation(), ClientOperation::Login);
    let ticket = item
        .outcome()
        .login_response()
        .expect("typed Login response is available")
        .ticket()
        .expect("successful Login returns Ticket");
    ticket.with_ticket_value(|value| assert_eq!(value, SECRET_SENTINEL));
    assert!(!format!("{outcome:?}").contains("login-ticket-secret-sentinel"));
    assert_eq!(
        operation_and_payload_tags(captured.borrow().as_deref().expect("request captured")),
        Some((LOGIN, vec![LEASE_TIME, REQUEST_COUNT]))
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn failure_preserves_table_255_reason_without_retry() {
    let response = response(FAILURE, Some(PERMISSION_DENIED), None, None);
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let outcome = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(fixture_request())),
            &CodecLimits::defaults(),
        )
        .expect("Operation Failed is a server result, not a transport error");
    let result = outcome.get(0).expect("one Login result exists").outcome();
    assert_eq!(result.result().status().raw(), FAILURE);
    assert_eq!(
        result.result().reason().map(|reason| reason.raw()),
        Some(PERMISSION_DENIED)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn pending_preserves_exact_correlation_without_implicit_poll() {
    const CORRELATION: &[u8] = b"login-pending-correlation";
    let response = response(PENDING, None, Some(CORRELATION), Some(test_structure([])));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch =
        ClientBatch::new(ClientBatchItem::new(fixture_request())).with_asynchronous_indicator(1);
    let outcome = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the caller opted into asynchronous results");
    let ClientBatchOutcome::Pending(pending) = outcome.get(0).expect("one result exists").outcome()
    else {
        panic!("Login Pending remains explicitly resumable");
    };
    assert_eq!(pending.operation(), ClientOperation::Login);
    assert_eq!(pending.asynchronous_correlation_value(), CORRELATION);
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn partial_write_failure_preserves_possibly_sent_delivery_without_retry() {
    let (mut client, transport, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(fixture_request())),
            &CodecLimits::defaults(),
        )
        .expect_err("the scripted transport fails after a partial write");
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn malformed_success_ticket_is_sanitized_after_one_exchange() {
    let malformed = test_structure([test_item(
        TICKET,
        kmipkit_ttlv::Value::structure(test_structure([test_item(
            TICKET_TYPE,
            kmipkit_ttlv::Value::enumeration(1),
        )])),
    )]);
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response: response(SUCCESS, None, None, Some(malformed)),
        request_write_chunks: Vec::new(),
    });
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(fixture_request())),
            &CodecLimits::defaults(),
        )
        .expect_err("malformed successful Ticket is rejected");
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert!(!error.to_string().contains("login-ticket-secret-sentinel"));
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn returned_ticket_does_not_change_the_next_explicit_request_authentication() {
    let (mut client, state) = sequence_client(vec![
        response(SUCCESS, None, None, Some(ticket_payload())),
        asynchronous_response_bytes(PING, SUCCESS, None, None, Some(test_structure([]))),
    ]);
    client
        .execute(
            ClientBatch::new(ClientBatchItem::new(fixture_request())),
            &CodecLimits::defaults(),
        )
        .expect("Login returns Ticket to the caller");
    client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::ping())),
            &CodecLimits::defaults(),
        )
        .expect("the next Ping is a separate explicit request");

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
