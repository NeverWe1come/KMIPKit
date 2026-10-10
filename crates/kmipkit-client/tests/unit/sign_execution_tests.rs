//! Source-derived Sign client contracts from OASIS KMIP v2.1 §6.1.55,
//! Tables 334–336. These tests are not official Test Case executions.
//! Traceability: FR-001, FR-003, FR-004, FR-009–FR-013, FR-015, and
//! `KMIPKIT-ELEM-OP-C2S-SIGN`.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;

use kmipkit_protocol::{SecretBytes, SignRequest};
use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;
use zeroize::Zeroizing;

use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientRequest};

const SIGN_OPERATION: u32 = 0x0000_0021;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const GENERAL_FAILURE: u32 = 0x0000_0100;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const SIGNATURE_DATA: u32 = 0x0042_00C7;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;

#[derive(Default)]
struct ExchangeState {
    requests: Vec<Zeroizing<Vec<u8>>>,
    responses: VecDeque<Vec<u8>>,
    calls: usize,
    failure: Option<RequestDeliveryState>,
}

struct SignTransport(Rc<RefCell<ExchangeState>>);

impl Transport for SignTransport {
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
    (Client::for_test(SignTransport(Rc::clone(&state))), state)
}

fn request() -> ClientRequest {
    ClientRequest::sign(
        SignRequest::new().with_data(kmipkit_protocol::OperationData::ByteString(
            SecretBytes::new(b"input-to-sign".to_vec()),
        )),
    )
}

fn response(status: u32, reason: Option<u32>) -> Vec<u8> {
    let payload = (status == SUCCESS).then(|| {
        test_structure([
            test_item(
                UNIQUE_IDENTIFIER,
                Value::text_string("signing-key".to_owned()),
            ),
            test_item(SIGNATURE_DATA, Value::byte_string(vec![0x00, 0x80, 0xFF])),
        ])
    });
    asynchronous_response_bytes(SIGN_OPERATION, status, reason, None, payload)
}

fn batch_response_bytes() -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(2)),
    ]);
    let response_item = |id: &[u8], key: &str, output: &[u8]| {
        let payload = test_structure([
            test_item(UNIQUE_IDENTIFIER, Value::text_string(key.to_owned())),
            test_item(SIGNATURE_DATA, Value::byte_string(output.to_vec())),
        ]);
        test_item(
            BATCH_ITEM,
            Value::structure(test_structure([
                test_item(0x0042_005C, Value::enumeration(SIGN_OPERATION)),
                test_item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id.to_vec())),
                test_item(RESULT_STATUS, Value::enumeration(SUCCESS)),
                test_item(RESPONSE_PAYLOAD, Value::structure(payload)),
            ])),
        )
    };
    let tree = test_structure([
        test_item(RESPONSE_HEADER, Value::structure(header)),
        response_item(b"sign-1", "key-1", b"one"),
        response_item(b"sign-2", "key-2", b"two"),
    ]);
    crate::execute::encode_message_for_test(tree, &CodecLimits::defaults())
        .expect("two Sign batch responses form a valid message")
}

#[test]
fn explicit_sign_executes_once_and_exposes_server_output() {
    let (mut client, state) = client(response(SUCCESS, None));
    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect("the server Sign result is valid");
    let item = response
        .items
        .first()
        .expect("one Sign result is associated");
    let typed = item
        .outcome()
        .sign_response()
        .expect("the result is a typed Sign response");

    assert_eq!(item.outcome().operation(), crate::ClientOperation::Sign);
    typed
        .unique_identifier()
        .expect("the server returned its managed key identifier");
    typed
        .signature_data()
        .expect("the server produced Signature Data")
        .with_bytes(|actual| assert_eq!(actual, [0x00, 0x80, 0xFF]));
    assert_eq!(
        state.borrow().calls,
        1,
        "an explicit Sign makes one exchange"
    );
    assert_eq!(state.borrow().requests.len(), 1);
}

#[test]
fn server_operation_failure_remains_an_operation_result() {
    let (mut client, state) = client(response(OPERATION_FAILED, Some(GENERAL_FAILURE)));
    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect("a server Operation Failed result is not a transport error");
    let outcome = response
        .items
        .first()
        .expect("one failed Sign result is associated")
        .outcome();

    assert_eq!(outcome.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        outcome.result().reason().map(|reason| reason.raw()),
        Some(GENERAL_FAILURE)
    );
    assert!(matches!(outcome, ClientBatchOutcome::Completed(_)));
    assert_eq!(state.borrow().calls, 1);
}

#[test]
fn sign_batch_results_preserve_each_unique_batch_item_id() {
    let (mut client, state) = client(batch_response_bytes());
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(request()).with_unique_batch_item_id(b"sign-1".to_vec()),
        ClientBatchItem::new(request()).with_unique_batch_item_id(b"sign-2".to_vec()),
    ]);
    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("Sign responses are associated with their explicit batch identifiers");

    assert_eq!(response.items.len(), 2);
    assert_eq!(
        response.items[0].unique_batch_item_id(),
        Some(&b"sign-1"[..])
    );
    assert_eq!(
        response.items[1].unique_batch_item_id(),
        Some(&b"sign-2"[..])
    );
    assert_eq!(
        response.items[0].outcome().operation(),
        crate::ClientOperation::Sign
    );
    assert_eq!(
        response.items[1].outcome().operation(),
        crate::ClientOperation::Sign
    );
    assert_eq!(state.borrow().calls, 1, "a Sign batch is one exchange");
}

#[test]
fn possibly_sent_sign_transport_failure_reports_delivery_and_is_not_retried() {
    let state = Rc::new(RefCell::new(ExchangeState {
        failure: Some(RequestDeliveryState::PossiblySent),
        ..ExchangeState::default()
    }));
    let mut client = Client::for_test(SignTransport(Rc::clone(&state)));
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect_err("transport failure preserves delivery evidence");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(
        state.borrow().calls,
        1,
        "Sign is never retried automatically"
    );
    assert_eq!(state.borrow().requests.len(), 1);
}
