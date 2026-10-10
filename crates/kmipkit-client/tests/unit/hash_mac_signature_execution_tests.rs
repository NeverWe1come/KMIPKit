//! Shared execution contracts derived from OASIS KMIP v2.1 Tables 235, 259,
//! 262, 334, and 337. These tests are source-derived and do not claim an
//! official Test Case pass. Traceability: `KMIPKIT-ELEM-OP-C2S-HASH`,
//! `KMIPKIT-ELEM-OP-C2S-MAC`, `KMIPKIT-ELEM-OP-C2S-MAC-VERIFY`,
//! `KMIPKIT-ELEM-OP-C2S-SIGN`, and
//! `KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY`.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use kmipkit_protocol::{
    HashRequest, MacRequest, MacVerifyRequest, OperationData, SecretBytes, SignRequest,
    SignatureVerifyRequest,
};
use kmipkit_transport::{Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Value, ValueView};
use zeroize::Zeroizing;

use crate::execute::{BatchIdentity, Client, associate_batch_items};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{ClientBatch, ClientBatchItem, ClientOperation, ClientRequest};

const HASH: u32 = 0x0000_0027;
const MAC: u32 = 0x0000_0023;
const MAC_VERIFY: u32 = 0x0000_0024;
const SIGN: u32 = 0x0000_0021;
const SIGNATURE_VERIFY: u32 = 0x0000_0022;
const SUCCESS: u32 = 0;
const FAILURE: u32 = 1;

const OPERATION: u32 = 0x0042_005C;
const BATCH_ITEM: u32 = 0x0042_000F;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const HASHING_ALGORITHM: u32 = 0x0042_0038;

#[derive(Clone, Copy)]
struct OperationFixture {
    code: u32,
    operation: ClientOperation,
}

impl OperationFixture {
    const ALL: [Self; 5] = [
        Self {
            code: HASH,
            operation: ClientOperation::Hash,
        },
        Self {
            code: MAC,
            operation: ClientOperation::Mac,
        },
        Self {
            code: MAC_VERIFY,
            operation: ClientOperation::MacVerify,
        },
        Self {
            code: SIGN,
            operation: ClientOperation::Sign,
        },
        Self {
            code: SIGNATURE_VERIFY,
            operation: ClientOperation::SignatureVerify,
        },
    ];

    fn request(self) -> ClientRequest {
        let bytes = || OperationData::ByteString(SecretBytes::new(b"message".to_vec()));
        match self.code {
            HASH => ClientRequest::hash(
                HashRequest::new(test_structure([test_item(
                    HASHING_ALGORITHM,
                    Value::enumeration(6),
                )]))
                .with_data(bytes()),
            ),
            MAC => ClientRequest::mac(MacRequest::new().with_data(bytes())),
            MAC_VERIFY => ClientRequest::mac_verify(
                MacVerifyRequest::new().with_mac_data(SecretBytes::new(b"mac".to_vec())),
            ),
            SIGN => ClientRequest::sign(SignRequest::new().with_data(bytes())),
            SIGNATURE_VERIFY => ClientRequest::signature_verify(
                SignatureVerifyRequest::new()
                    .with_signature_data(SecretBytes::new(b"signature".to_vec())),
            ),
            _ => unreachable!("fixture only uses the five assigned operations"),
        }
    }

    fn success_payload(self) -> kmipkit_ttlv::Structure {
        let fields = match self.code {
            HASH => vec![test_item(
                0x0042_00C2,
                Value::byte_string(b"digest".to_vec()),
            )],
            MAC => vec![
                test_item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string("mac-object".to_owned()),
                ),
                test_item(0x0042_00C4, Value::byte_string(b"mac".to_vec())),
            ],
            MAC_VERIFY => vec![
                test_item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string("mac-object".to_owned()),
                ),
                test_item(0x0042_0128, Value::enumeration(1)),
            ],
            SIGN => vec![
                test_item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string("sign-object".to_owned()),
                ),
                test_item(0x0042_00C7, Value::byte_string(b"signature".to_vec())),
            ],
            SIGNATURE_VERIFY => vec![
                test_item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string("sign-object".to_owned()),
                ),
                test_item(0x0042_0128, Value::enumeration(1)),
            ],
            _ => unreachable!("fixture only uses the five assigned operations"),
        };
        test_structure(fields)
    }

    fn success_response(self) -> Vec<u8> {
        asynchronous_response_bytes(self.code, SUCCESS, None, None, Some(self.success_payload()))
    }
}

#[derive(Default)]
struct ExchangeState {
    requests: Vec<Zeroizing<Vec<u8>>>,
    responses: VecDeque<Zeroizing<Vec<u8>>>,
}

struct CountingTransport(Rc<RefCell<ExchangeState>>);

impl Transport for CountingTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        let mut state = self.0.borrow_mut();
        state.requests.push(Zeroizing::new(request.to_vec()));
        let response = state
            .responses
            .pop_front()
            .expect("the fixture queues one response for each explicit call");
        Ok(TransportResponse::new(response.to_vec()))
    }
}

fn client(responses: impl IntoIterator<Item = Vec<u8>>) -> (Client, Rc<RefCell<ExchangeState>>) {
    let state = Rc::new(RefCell::new(ExchangeState {
        requests: Vec::new(),
        responses: responses.into_iter().map(Zeroizing::new).collect(),
    }));
    (
        Client::for_test(CountingTransport(Rc::clone(&state))),
        state,
    )
}

#[test]
fn client_requests_map_to_the_five_kmip_operation_codes() {
    for fixture in OperationFixture::ALL {
        let request = fixture.request();
        assert_eq!(request.operation(), fixture.code);
    }
}

#[test]
fn each_explicit_operation_call_performs_one_exchange_without_retry() {
    let (mut client, state) = client(
        OperationFixture::ALL
            .into_iter()
            .map(|item| item.success_response()),
    );

    for fixture in OperationFixture::ALL {
        let response = client
            .execute(
                ClientBatch::new(ClientBatchItem::new(fixture.request())),
                &CodecLimits::defaults(),
            )
            .expect("the queued server response completes the explicit operation");
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].outcome().operation(), fixture.operation);
        assert_eq!(response.items[0].outcome().result().status().raw(), SUCCESS);
    }

    assert_eq!(state.borrow().requests.len(), OperationFixture::ALL.len());
}

#[test]
fn batch_item_ids_associate_out_of_order_responses_with_their_operations() {
    let requests = [
        BatchIdentity {
            operation: HASH,
            unique_batch_item_id: Some(b"hash-id".to_vec()),
            verification_response_context: None,
        },
        BatchIdentity {
            operation: SIGN,
            unique_batch_item_id: Some(b"sign-id".to_vec()),
            verification_response_context: None,
        },
    ];
    let responses = [
        BatchIdentity {
            operation: SIGN,
            unique_batch_item_id: Some(b"sign-id".to_vec()),
            verification_response_context: None,
        },
        BatchIdentity {
            operation: HASH,
            unique_batch_item_id: Some(b"hash-id".to_vec()),
            verification_response_context: None,
        },
    ];

    assert_eq!(associate_batch_items(&requests, &responses), Ok(vec![1, 0]));
}

#[test]
fn request_operation_codes_are_written_to_the_wire_and_server_failure_is_preserved() {
    let fixture = OperationFixture::ALL[0];
    let failure = asynchronous_response_bytes(fixture.code, FAILURE, Some(1), None, None);
    let (mut client, state) = client([failure]);
    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(fixture.request())),
            &CodecLimits::defaults(),
        )
        .expect("a valid KMIP failure remains an operation result");
    let item = response.items.first().expect("one result is associated");
    assert_eq!(item.outcome().result().status().raw(), FAILURE);
    assert_eq!(
        item.outcome().result().reason().map(|reason| reason.raw()),
        Some(1)
    );

    let captured = decode(state.borrow().requests[0].as_slice())
        .expect("the captured request is structurally valid TTLV");
    let encoded_operation = captured.with_value(|root| {
        let ValueView::Structure(root) = root else {
            panic!("the root is a KMIP Structure");
        };
        root.children()
            .iter()
            .find(|item| item.tag().raw() == BATCH_ITEM)
            .and_then(|batch| {
                batch.with_value(|value| {
                    let ValueView::Structure(batch) = value else {
                        return None;
                    };
                    batch
                        .children()
                        .iter()
                        .find(|item| item.tag().raw() == OPERATION)
                        .and_then(|field| {
                            field.with_value(|value| match value {
                                ValueView::Enumeration(value) => Some(*value),
                                _ => None,
                            })
                        })
                })
            })
    });
    assert_eq!(encoded_operation, Some(fixture.code));
    assert!(state.borrow().requests[0].len() > 0);
}
