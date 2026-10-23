//! Source-derived Sign client contracts from OASIS KMIP v2.1 §6.1.55,
//! Tables 334–336. These tests are not official Test Case executions.
//! Traceability: FR-001, FR-003–FR-005, FR-009–FR-013, FR-015, and
//! `KMIPKIT-ELEM-OP-C2S-SIGN`.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;

use kmipkit_protocol::{OperationData, SecretBytes, SignRequest, UniqueIdentifier};
use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use kmipkit_ttlv::Value;
use kmipkit_ttlv::ValueView;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use zeroize::Zeroizing;

use crate::execute_test_support::{
    asynchronous_response_bytes, structure_contains_byte_string, test_item, test_structure,
};
use crate::{Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientRequest};

const SIGN_OPERATION: u32 = 0x0000_0021;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const GENERAL_FAILURE: u32 = 0x0000_0100;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const DATA: u32 = 0x0042_00C2;
const DIGESTED_DATA: u32 = 0x0042_0107;
const SIGNATURE_DATA: u32 = 0x0042_00C3;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const INIT_INDICATOR: u32 = 0x0042_00D7;
const FINAL_INDICATOR: u32 = 0x0042_00D8;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const GENERIC_RESPONSE_TAG: u32 = 0x0042_00D6;
const GENERIC_RESPONSE_BYTES: &[u8] = b"sign-client-generic-ttlv";
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
        SignRequest::new()
            .with_unique_identifier(UniqueIdentifier::TextString("signing-key".to_owned()))
            .with_data(kmipkit_protocol::OperationData::ByteString(
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
            test_item(
                GENERIC_RESPONSE_TAG,
                Value::byte_string(GENERIC_RESPONSE_BYTES.to_vec()),
            ),
        ])
    });
    asynchronous_response_bytes(SIGN_OPERATION, status, reason, None, payload)
}

fn multipart_response() -> Vec<u8> {
    let payload = test_structure([test_item(
        UNIQUE_IDENTIFIER,
        Value::text_string("signing-key".to_owned()),
    )]);
    asynchronous_response_bytes(SIGN_OPERATION, SUCCESS, None, None, Some(payload))
}

#[derive(Debug, Eq, PartialEq)]
enum CapturedSignField {
    Text(String),
    Bytes(Vec<u8>),
    Boolean(bool),
    Other,
}

fn request_payload_fields(bytes: &[u8]) -> (Option<u32>, Vec<(u32, CapturedSignField)>) {
    let message = decode(bytes).expect("the captured Sign request is valid TTLV");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("the KMIP request message is a Structure");
        };
        let batch_item = message
            .children()
            .iter()
            .find(|field| field.tag().raw() == BATCH_ITEM)
            .expect("the Sign request contains a Batch Item");
        batch_item.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("the Batch Item is a Structure");
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)
                .and_then(|field| {
                    field.with_value(|value| match value {
                        ValueView::Enumeration(value) => Some(*value),
                        _ => None,
                    })
                });
            let request_payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("the Sign Batch Item contains a Request Payload");
            let fields = request_payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("the Request Payload is a Structure");
                };
                payload
                    .children()
                    .iter()
                    .map(|field| {
                        let snapshot = field.with_value(|value| match value {
                            ValueView::TextString(value) => {
                                CapturedSignField::Text(value.to_owned())
                            }
                            ValueView::ByteString(value) => {
                                CapturedSignField::Bytes(value.to_vec())
                            }
                            ValueView::Boolean(value) => CapturedSignField::Boolean(*value),
                            _ => CapturedSignField::Other,
                        });
                        (field.tag().raw(), snapshot)
                    })
                    .collect()
            });
            (operation, fields)
        })
    })
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
    assert!(
        item.outcome()
            .response()
            .sign()
            .expect("the common client response view exposes Sign")
            .with_ttlv(|tree| {
                structure_contains_byte_string(&tree, GENERIC_RESPONSE_TAG, GENERIC_RESPONSE_BYTES)
            })
    );
    assert_eq!(
        state.borrow().calls,
        1,
        "an explicit Sign makes one exchange"
    );
    assert_eq!(state.borrow().requests.len(), 1);
}

#[test]
fn request_table_fields_reach_the_fake_transport_unchanged() {
    let data = [0x00, 0x80, 0xFF];
    let digest = [0x01, 0x81, 0xFE];
    let (mut single_part_client, single_part_state) = client(response(SUCCESS, None));
    single_part_client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::sign(
                SignRequest::new()
                    .with_unique_identifier(UniqueIdentifier::TextString("signing-key".to_owned()))
                    .with_data(OperationData::ByteString(SecretBytes::new(data.to_vec())))
                    .with_digested_data(SecretBytes::new(digest.to_vec())),
            ))),
            &CodecLimits::defaults(),
        )
        .expect("single-part Sign input fields form a valid client request");
    let captured_single_part = single_part_state.borrow();
    let (single_part_operation, single_part_fields) =
        request_payload_fields(captured_single_part.requests[0].as_slice());
    assert_eq!(single_part_operation, Some(SIGN_OPERATION));
    assert_eq!(
        single_part_fields,
        [
            (
                UNIQUE_IDENTIFIER,
                CapturedSignField::Text("signing-key".to_owned()),
            ),
            (DATA, CapturedSignField::Bytes(data.to_vec())),
            (DIGESTED_DATA, CapturedSignField::Bytes(digest.to_vec())),
        ]
    );
    drop(captured_single_part);

    let correlation = [0x02, 0x82, 0xFD];
    let (mut multipart_client, multipart_state) = client(multipart_response());
    multipart_client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::sign(
                SignRequest::new()
                    .with_unique_identifier(UniqueIdentifier::TextString("signing-key".to_owned()))
                    .with_correlation_value(SecretBytes::new(correlation.to_vec()))
                    .with_init_indicator(false)
                    .with_final_indicator(true),
            ))),
            &CodecLimits::defaults(),
        )
        .expect("multipart framing fields form a valid client request");
    let captured_multipart = multipart_state.borrow();
    let (multipart_operation, multipart_fields) =
        request_payload_fields(captured_multipart.requests[0].as_slice());
    assert_eq!(multipart_operation, Some(SIGN_OPERATION));
    assert_eq!(
        multipart_fields,
        [
            (
                UNIQUE_IDENTIFIER,
                CapturedSignField::Text("signing-key".to_owned()),
            ),
            (
                CORRELATION_VALUE,
                CapturedSignField::Bytes(correlation.to_vec()),
            ),
            (INIT_INDICATOR, CapturedSignField::Boolean(false)),
            (FINAL_INDICATOR, CapturedSignField::Boolean(true)),
        ]
    );
}

#[test]
fn sign_response_shape_error_retains_the_generic_ttlv_item() {
    let payload = test_structure([
        test_item(SIGNATURE_DATA, Value::byte_string(vec![0x00, 0x80, 0xFF])),
        test_item(
            GENERIC_RESPONSE_TAG,
            Value::byte_string(GENERIC_RESPONSE_BYTES.to_vec()),
        ),
    ]);
    let response = asynchronous_response_bytes(SIGN_OPERATION, SUCCESS, None, None, Some(payload));
    let (mut client, _) = client(response);
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect_err("Sign success requires exactly one well-formed Unique Identifier");

    assert_eq!(error.category(), crate::ClientErrorCategory::Protocol);
    assert!(
        error
            .response_ttlv()
            .expect("typed response shape errors retain the generic response")
            .with_ttlv(|tree| {
                structure_contains_byte_string(&tree, GENERIC_RESPONSE_TAG, GENERIC_RESPONSE_BYTES)
            })
    );
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
        outcome
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(GENERAL_FAILURE)
    );
    assert!(matches!(outcome, ClientBatchOutcome::Sign(_)));
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
