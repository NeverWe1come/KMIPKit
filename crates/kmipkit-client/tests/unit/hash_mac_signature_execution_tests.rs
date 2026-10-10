//! Shared execution contracts derived from OASIS KMIP v2.1 Tables 235, 259,
//! 262, 334, and 337. These tests are source-derived and do not claim an
//! official Test Case pass. Traceability: `KMIPKIT-ELEM-OP-C2S-HASH`,
//! `KMIPKIT-ELEM-OP-C2S-MAC`, `KMIPKIT-ELEM-OP-C2S-MAC-VERIFY`,
//! `KMIPKIT-ELEM-OP-C2S-SIGN`, and
//! `KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY`. Response output cardinality is
//! derived from §6.1.24 Table 236, §6.1.32 Table 260, and §6.1.55 Table 335.

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
use crate::execute_test_support::{
    asynchronous_response_bytes, operation_batch_response_bytes, test_item, test_structure,
};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchItemResponse, ClientOperation, ClientRequest,
};
use crate::{ClientResponseView, RequestOptions};

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
const HASH_DATA: u32 = 0x0042_00C2;
const MAC_DATA: u32 = 0x0042_00C4;
const SIGNATURE_DATA: u32 = 0x0042_00C7;

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

    fn multipart_request(self) -> ClientRequest {
        match self.code {
            HASH => ClientRequest::hash(
                HashRequest::new(test_structure([test_item(
                    HASHING_ALGORITHM,
                    Value::enumeration(6),
                )]))
                .with_init_indicator(true),
            ),
            MAC => ClientRequest::mac(MacRequest::new().with_init_indicator(true)),
            SIGN => ClientRequest::sign(SignRequest::new().with_init_indicator(true)),
            _ => unreachable!("only Hash, MAC, and Sign have output cardinality here"),
        }
    }

    fn final_multipart_request(self) -> ClientRequest {
        let correlation = || SecretBytes::new(b"multipart-correlation".to_vec());
        match self.code {
            HASH => ClientRequest::hash(
                HashRequest::new(test_structure([test_item(
                    HASHING_ALGORITHM,
                    Value::enumeration(6),
                )]))
                .with_correlation_value(correlation())
                .with_final_indicator(true),
            ),
            MAC => ClientRequest::mac(
                MacRequest::new()
                    .with_correlation_value(correlation())
                    .with_final_indicator(true),
            ),
            SIGN => ClientRequest::sign(
                SignRequest::new()
                    .with_correlation_value(correlation())
                    .with_final_indicator(true),
            ),
            _ => unreachable!("only Hash, MAC, and Sign have output cardinality here"),
        }
    }

    fn success_payload(self) -> kmipkit_ttlv::Structure {
        self.success_payload_with_output(true)
    }

    fn success_payload_with_output(self, include_output: bool) -> kmipkit_ttlv::Structure {
        let mut fields = match self.code {
            HASH => Vec::new(),
            MAC => vec![test_item(
                UNIQUE_IDENTIFIER,
                Value::text_string("mac-object".to_owned()),
            )],
            MAC_VERIFY => vec![
                test_item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string("mac-object".to_owned()),
                ),
                test_item(0x0042_0128, Value::enumeration(1)),
            ],
            SIGN => vec![test_item(
                UNIQUE_IDENTIFIER,
                Value::text_string("sign-object".to_owned()),
            )],
            SIGNATURE_VERIFY => vec![
                test_item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string("sign-object".to_owned()),
                ),
                test_item(0x0042_0128, Value::enumeration(1)),
            ],
            _ => unreachable!("fixture only uses the five assigned operations"),
        };
        if include_output {
            match self.code {
                HASH => fields.push(test_item(HASH_DATA, Value::byte_string(b"digest".to_vec()))),
                MAC => fields.push(test_item(MAC_DATA, Value::byte_string(b"mac".to_vec()))),
                SIGN => fields.push(test_item(
                    SIGNATURE_DATA,
                    Value::byte_string(b"signature".to_vec()),
                )),
                _ => {}
            }
        }
        test_structure(fields)
    }

    fn success_response(self) -> Vec<u8> {
        asynchronous_response_bytes(self.code, SUCCESS, None, None, Some(self.success_payload()))
    }

    fn pending_response(self) -> Vec<u8> {
        asynchronous_response_bytes(
            self.code,
            2,
            None,
            Some(b"pending-correlation"),
            Some(test_structure([])),
        )
    }
}

fn output_operation_fixtures() -> [OperationFixture; 3] {
    [
        OperationFixture::ALL[0],
        OperationFixture::ALL[1],
        OperationFixture::ALL[3],
    ]
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
            .map(OperationFixture::success_response),
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

fn invoke_typed_convenience(
    client: &mut Client,
    fixture: OperationFixture,
    with_options: bool,
    limits: &CodecLimits,
) -> ClientBatchItemResponse {
    let options = RequestOptions::default();
    let result = match fixture.request() {
        ClientRequest::Hash(request) if with_options => {
            client.hash_with_options(request, limits, &options)
        }
        ClientRequest::Hash(request) => client.hash(request, limits),
        ClientRequest::Mac(request) if with_options => {
            client.mac_with_options(request, limits, &options)
        }
        ClientRequest::Mac(request) => client.mac(request, limits),
        ClientRequest::MacVerify(request) if with_options => {
            client.mac_verify_with_options(request, limits, &options)
        }
        ClientRequest::MacVerify(request) => client.mac_verify(request, limits),
        ClientRequest::Sign(request) if with_options => {
            client.sign_with_options(request, limits, &options)
        }
        ClientRequest::Sign(request) => client.sign(request, limits),
        ClientRequest::SignatureVerify(request) if with_options => {
            client.signature_verify_with_options(request, limits, &options)
        }
        ClientRequest::SignatureVerify(request) => client.signature_verify(request, limits),
        _ => unreachable!("the fixture only constructs the five assigned operations"),
    };
    result.expect("the queued response completes the typed convenience call")
}

fn response_accessor_presence(view: &ClientResponseView<'_>) -> [bool; 5] {
    [
        view.hash().is_some(),
        view.mac().is_some(),
        view.mac_verify().is_some(),
        view.sign().is_some(),
        view.signature_verify().is_some(),
    ]
}

fn outcome_accessor_presence(outcome: &crate::ClientBatchOutcome) -> [bool; 5] {
    [
        outcome.hash_response().is_some(),
        outcome.mac_response().is_some(),
        outcome.mac_verify_response().is_some(),
        outcome.sign_response().is_some(),
        outcome.signature_verify_response().is_some(),
    ]
}

fn expected_accessor_presence(operation: ClientOperation) -> [bool; 5] {
    [
        operation == ClientOperation::Hash,
        operation == ClientOperation::Mac,
        operation == ClientOperation::MacVerify,
        operation == ClientOperation::Sign,
        operation == ClientOperation::SignatureVerify,
    ]
}

#[test]
fn public_convenience_methods_views_accessors_and_formatters_cover_all_operations() {
    let queued_responses = OperationFixture::ALL
        .into_iter()
        .flat_map(|fixture| [fixture.success_response(), fixture.success_response()]);
    let (mut client, state) = client(queued_responses);
    let limits = CodecLimits::defaults();

    for fixture in OperationFixture::ALL {
        let request = fixture.request();
        let request_debug = format!("{request:?}");
        assert!(!request_debug.contains("message"));

        for with_options in [false, true] {
            let item = invoke_typed_convenience(&mut client, fixture, with_options, &limits);
            let outcome = item.outcome();
            assert_eq!(outcome.operation(), fixture.operation);
            assert_eq!(outcome.result().status().raw(), SUCCESS);

            let expected = expected_accessor_presence(fixture.operation);
            let view = outcome.response();
            assert_eq!(response_accessor_presence(&view), expected);
            assert_eq!(outcome_accessor_presence(outcome), expected);

            let rendered = format!("{item:?} {outcome:?} {view:?} {outcome}");
            assert!(!rendered.is_empty());
        }
    }

    assert_eq!(state.borrow().requests.len(), 10);
}

#[test]
fn pending_views_preserve_each_operation_response_type() {
    let responses = OperationFixture::ALL
        .into_iter()
        .map(OperationFixture::pending_response);
    let (mut client, state) = client(responses);
    let limits = CodecLimits::defaults();

    for fixture in OperationFixture::ALL {
        let outcome = client
            .execute(
                ClientBatch::new(ClientBatchItem::new(fixture.request()))
                    .with_asynchronous_indicator(1),
                &limits,
            )
            .expect("the explicitly asynchronous Pending response is accepted");
        let item = outcome.items.first().expect("one item is associated");
        let crate::ClientBatchOutcome::Pending(pending) = item.outcome() else {
            panic!("a Pending response remains an explicit resumable outcome");
        };
        assert_eq!(pending.operation(), fixture.operation);
        assert_eq!(
            pending.asynchronous_correlation_value(),
            b"pending-correlation"
        );
        assert_eq!(pending.response().result().status().raw(), 2);
        assert_eq!(
            response_accessor_presence(&pending.response()),
            expected_accessor_presence(fixture.operation)
        );
        assert!(!format!("{pending:?}").contains("pending-correlation"));
    }

    assert_eq!(state.borrow().requests.len(), OperationFixture::ALL.len());
}

#[test]
fn batch_item_ids_associate_out_of_order_responses_with_their_operations() {
    let requests = [
        BatchIdentity {
            operation: HASH,
            unique_batch_item_id: Some(b"hash-id".to_vec()),
            response_context: None,
        },
        BatchIdentity {
            operation: SIGN,
            unique_batch_item_id: Some(b"sign-id".to_vec()),
            response_context: None,
        },
    ];
    let responses = [
        BatchIdentity {
            operation: SIGN,
            unique_batch_item_id: Some(b"sign-id".to_vec()),
            response_context: None,
        },
        BatchIdentity {
            operation: HASH,
            unique_batch_item_id: Some(b"hash-id".to_vec()),
            response_context: None,
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
        item.outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
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
    assert!(!state.borrow().requests[0].is_empty());
}

#[test]
fn successful_single_part_hash_mac_and_sign_responses_require_output_data() {
    let rejected = output_operation_fixtures().map(|fixture| {
        let response = asynchronous_response_bytes(
            fixture.code,
            SUCCESS,
            None,
            None,
            Some(fixture.success_payload_with_output(false)),
        );
        let (mut client, _) = client([response]);
        client
            .execute(
                ClientBatch::new(ClientBatchItem::new(fixture.request())),
                &CodecLimits::defaults(),
            )
            .is_err()
    });
    assert_eq!(rejected, [true; 3]);
}

#[test]
fn multipart_hash_mac_and_sign_responses_reject_output_data() {
    let rejected = output_operation_fixtures().map(|fixture| {
        let response = asynchronous_response_bytes(
            fixture.code,
            SUCCESS,
            None,
            None,
            Some(fixture.success_payload_with_output(true)),
        );
        let (mut client, _) = client([response]);
        client
            .execute(
                ClientBatch::new(ClientBatchItem::new(fixture.multipart_request())),
                &CodecLimits::defaults(),
            )
            .is_err()
    });
    assert_eq!(rejected, [true; 3]);
}

#[test]
fn multipart_hash_mac_and_sign_responses_accept_absent_output_data() {
    for fixture in output_operation_fixtures() {
        let response = asynchronous_response_bytes(
            fixture.code,
            SUCCESS,
            None,
            None,
            Some(fixture.success_payload_with_output(false)),
        );
        let (mut client, _) = client([response]);
        let result = client
            .execute(
                ClientBatch::new(ClientBatchItem::new(fixture.multipart_request())),
                &CodecLimits::defaults(),
            )
            .expect("multipart response without output conforms to the operation table");
        let view = result.items[0].outcome().response();
        match fixture.code {
            HASH => assert!(
                view.hash()
                    .is_some_and(|response| response.data().is_none())
            ),
            MAC => assert!(
                view.mac()
                    .is_some_and(|response| response.mac_data().is_none())
            ),
            SIGN => assert!(
                view.sign()
                    .is_some_and(|response| response.signature_data().is_none())
            ),
            _ => unreachable!("fixture loop contains only Hash, MAC, and Sign"),
        }
    }
}

#[test]
fn final_multipart_hash_mac_and_sign_responses_omit_output_data() {
    let conforms = output_operation_fixtures().map(|fixture| {
        let without_output = asynchronous_response_bytes(
            fixture.code,
            SUCCESS,
            None,
            None,
            Some(fixture.success_payload_with_output(false)),
        );
        let (mut first_client, _) = client([without_output]);
        let absent_output_accepted = first_client
            .execute(
                ClientBatch::new(ClientBatchItem::new(fixture.final_multipart_request())),
                &CodecLimits::defaults(),
            )
            .is_ok();

        let with_output = asynchronous_response_bytes(
            fixture.code,
            SUCCESS,
            None,
            None,
            Some(fixture.success_payload_with_output(true)),
        );
        let (mut second_client, _) = client([with_output]);
        let present_output_rejected = second_client
            .execute(
                ClientBatch::new(ClientBatchItem::new(fixture.final_multipart_request())),
                &CodecLimits::defaults(),
            )
            .is_err();

        absent_output_accepted && present_output_rejected
    });
    assert_eq!(conforms, [true; 3]);
}

#[test]
fn reordered_hash_batch_responses_keep_the_original_request_context() {
    let response = operation_batch_response_bytes([
        (
            HASH,
            b"multipart-id".to_vec(),
            OperationFixture::ALL[0].success_payload_with_output(false),
        ),
        (
            HASH,
            b"single-id".to_vec(),
            OperationFixture::ALL[0].success_payload_with_output(true),
        ),
    ]);
    let (mut client, _) = client([response]);
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(OperationFixture::ALL[0].request())
            .with_unique_batch_item_id(b"single-id".to_vec()),
        ClientBatchItem::new(OperationFixture::ALL[0].multipart_request())
            .with_unique_batch_item_id(b"multipart-id".to_vec()),
    ]);

    let result = client
        .execute(batch, &CodecLimits::defaults())
        .expect("responses reordered by ID retain their matching request framing");

    assert_eq!(
        result.items[0].unique_batch_item_id(),
        Some(&b"single-id"[..])
    );
    assert_eq!(
        result.items[1].unique_batch_item_id(),
        Some(&b"multipart-id"[..])
    );
    assert!(
        result.items[0]
            .outcome()
            .response()
            .hash()
            .is_some_and(|response| response.data().is_some())
    );
    assert!(
        result.items[1]
            .outcome()
            .response()
            .hash()
            .is_some_and(|response| response.data().is_none())
    );
}

#[test]
fn reordered_hash_batch_rejects_outputs_mismatched_to_request_context() {
    let response = operation_batch_response_bytes([
        (
            HASH,
            b"multipart-id".to_vec(),
            OperationFixture::ALL[0].success_payload_with_output(true),
        ),
        (
            HASH,
            b"single-id".to_vec(),
            OperationFixture::ALL[0].success_payload_with_output(false),
        ),
    ]);
    let (mut client, _) = client([response]);
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(OperationFixture::ALL[0].request())
            .with_unique_batch_item_id(b"single-id".to_vec()),
        ClientBatchItem::new(OperationFixture::ALL[0].multipart_request())
            .with_unique_batch_item_id(b"multipart-id".to_vec()),
    ]);

    assert!(
        client.execute(batch, &CodecLimits::defaults()).is_err(),
        "the response parser must use request context after batch-ID association"
    );
}
