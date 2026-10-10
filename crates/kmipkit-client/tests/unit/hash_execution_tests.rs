//! Source-derived Hash client contracts from OASIS KMIP v2.1 §6.1.24,
//! Tables 235–237. These are not complete official Test Case executions.
//!
//! Traceability: FR-001–FR-003, FR-009–FR-012, and
//! `KMIPKIT-ELEM-OP-C2S-HASH`.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;

use kmipkit_protocol::{HashRequest, OperationData, SecretBytes};
use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;
use zeroize::Zeroizing;

use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientRequest};

const HASH_OPERATION: u32 = 0x0000_0027;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const GENERAL_FAILURE: u32 = 0x0000_0100;
const PENDING: u32 = 2;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const DATA: u32 = 0x0042_00C2;

#[derive(Default)]
struct ExchangeState {
    requests: Vec<Zeroizing<Vec<u8>>>,
    responses: VecDeque<Vec<u8>>,
    calls: usize,
    failure: Option<RequestDeliveryState>,
}

struct HashTransport(Rc<RefCell<ExchangeState>>);

impl Transport for HashTransport {
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

fn client(response: Vec<u8>) -> (Client, Rc<RefCell<ExchangeState>>) {
    let state = Rc::new(RefCell::new(ExchangeState {
        responses: VecDeque::from([response]),
        ..ExchangeState::default()
    }));
    (Client::for_test(HashTransport(Rc::clone(&state))), state)
}

fn request() -> ClientRequest {
    let parameters = test_structure([test_item(HASHING_ALGORITHM, Value::enumeration(6))]);
    ClientRequest::hash(
        HashRequest::new(parameters).with_data(OperationData::ByteString(SecretBytes::new(
            b"input-to-hash".to_vec(),
        ))),
    )
}

fn response(status: u32, reason: Option<u32>, correlation: Option<&[u8]>) -> Vec<u8> {
    let payload = if status == PENDING {
        Some(test_structure([]))
    } else if status == SUCCESS {
        Some(test_structure([test_item(
            DATA,
            Value::byte_string(vec![0x00, 0x80, 0xFF]),
        )]))
    } else {
        None
    };
    asynchronous_response_bytes(HASH_OPERATION, status, reason, correlation, payload)
}

#[test]
fn explicit_hash_executes_once_and_exposes_the_server_data() {
    let (mut client, state) = client(response(SUCCESS, None, None));
    let batch = ClientBatch::new(ClientBatchItem::new(request()));
    let result = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the server Hash result is valid");
    let item = result.items.first().expect("one Hash result is associated");
    let response = item
        .outcome()
        .hash_response()
        .expect("the result is a typed Hash response");

    assert_eq!(item.outcome().operation(), crate::ClientOperation::Hash);
    response
        .data()
        .expect("the server returned a digest")
        .with_bytes(|actual| {
            assert_eq!(actual, [0x00, 0x80, 0xFF]);
        });
    assert_eq!(
        state.borrow().calls,
        1,
        "Hash must not retry or continue implicitly"
    );
    assert_eq!(state.borrow().requests.len(), 1);
}

#[test]
fn pending_hash_result_preserves_the_correlation_value() {
    let correlation = [0xA1, 0xB2, 0xFF];
    let (mut client, state) = client(response(PENDING, None, Some(&correlation)));
    let result = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())).with_asynchronous_indicator(1),
            &CodecLimits::defaults(),
        )
        .expect("a valid Pending Hash result is returned to the caller");
    let item = result
        .items
        .first()
        .expect("one Pending item is associated");

    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("Operation Pending remains an explicit resumable outcome");
    };
    assert_eq!(pending.operation(), crate::ClientOperation::Hash);
    assert_eq!(pending.asynchronous_correlation_value(), correlation);
    assert_eq!(state.borrow().calls, 1);
}

#[test]
fn operation_failure_preserves_server_status_and_reason() {
    let (mut client, state) = client(response(OPERATION_FAILED, Some(GENERAL_FAILURE), None));
    let result = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect("operation failures remain results rather than transport errors");
    let outcome = result
        .items
        .first()
        .expect("one Hash result is associated")
        .outcome();

    assert_eq!(outcome.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        outcome
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(GENERAL_FAILURE)
    );
    assert!(
        outcome
            .response()
            .hash()
            .expect("typed Hash view")
            .data()
            .is_none()
    );
    assert_eq!(state.borrow().calls, 1);
}

#[test]
fn possibly_sent_transport_failure_exposes_delivery_state_without_retry() {
    let state = Rc::new(RefCell::new(ExchangeState {
        failure: Some(RequestDeliveryState::PossiblySent),
        ..ExchangeState::default()
    }));
    let mut client = Client::for_test(HashTransport(Rc::clone(&state)));
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect_err("the transport failure is surfaced with its delivery evidence");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(
        state.borrow().calls,
        1,
        "a possibly delivered Hash is never retried"
    );
    assert_eq!(state.borrow().requests.len(), 1);
}
