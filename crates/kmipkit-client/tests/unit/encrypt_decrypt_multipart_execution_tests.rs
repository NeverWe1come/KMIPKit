#![cfg(test)]

//! T027 Red client-contract tests derived from the OASIS KMIP Specification v2.1 §§6.1,
//! 6.1.11 Table 196, 6.1.17 Table 214, and §§7.3, 7.4, 7.8, 7.14, and 7.17.
//! Traceability: `KMIPKIT-ELEM-OP-C2S-ENCRYPT`,
//! `KMIPKIT-ELEM-OP-C2S-DECRYPT`,
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-3-AUTHENTICATED-ENCRYPTION-ADDITIONAL-DATA`,
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-4-AUTHENTICATED-ENCRYPTION-TAG`, and
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE`. Requirement IDs:
//! `KMIPKIT-REQ-SPEC-6.1-001-001`, `KMIPKIT-REQ-SPEC-6.1.11-006`,
//! `KMIPKIT-REQ-SPEC-6.1.17-006`, and `KMIPKIT-REQ-SPEC-7.8-001`. These tests
//! keep each part caller-driven: one typed execute call means one exchange.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use kmipkit_protocol::{
    DecryptRequest, EncryptRequest, OperationData, SecretBytes, UniqueIdentifier,
};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{ItemType, Value, ValueView};
use zeroize::Zeroizing;

use crate::execute::Client;
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchResponse, ClientCauseCategory, ClientErrorCategory,
    ClientRequest,
};

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const SUCCESS: u32 = 0;

const BATCH_ITEM: u32 = 0x0042_000F;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const INIT_INDICATOR: u32 = 0x0042_00D7;
const FINAL_INDICATOR: u32 = 0x0042_00D8;
const AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA: u32 = 0x0042_00FE;
const AUTHENTICATED_ENCRYPTION_TAG: u32 = 0x0042_00FF;
const DATA: u32 = 0x0042_00C2;

const SERVER_CORRELATION: &[u8] = b"server-correlation\x00\xff";
const FRAMED_SINGLE_DATA: &[u8] = b"framed-single-part-data";
const ENCRYPT_AAD: &[u8] = b"encrypt-aad-initial-only";
const DECRYPT_AAD: &[u8] = b"decrypt-aad-initial-only";
const DECRYPT_TAG: &[u8] = b"decrypt-tag-initial-only";

type SharedTransportState = Rc<RefCell<QueuedTransportState>>;

struct QueuedTransportState {
    responses: VecDeque<Zeroizing<Vec<u8>>>,
    requests: Vec<Zeroizing<Vec<u8>>>,
}

struct QueuedMultipartTransport {
    state: SharedTransportState,
}

impl Transport for QueuedMultipartTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        let mut state = self.state.borrow_mut();
        state.requests.push(Zeroizing::new(request.to_vec()));
        let response = state
            .responses
            .pop_front()
            .expect("the multipart test queues one response per explicit call");
        assert!(
            response.len() <= max_response_bytes,
            "the queued response fixture fits the client's response limit"
        );
        Ok(TransportResponse::new(response.to_vec()))
    }
}

struct CapturedField {
    tag: u32,
    item_type: ItemType,
    byte_string: Option<Zeroizing<Vec<u8>>>,
    boolean: Option<bool>,
}

fn client_with_responses(responses: [Vec<u8>; 3]) -> (Client, SharedTransportState) {
    let state = Rc::new(RefCell::new(QueuedTransportState {
        responses: responses.into_iter().map(Zeroizing::new).collect(),
        requests: Vec::new(),
    }));
    let client = Client::for_test(QueuedMultipartTransport {
        state: Rc::clone(&state),
    });
    (client, state)
}

fn success_response(operation: u32, correlation_value: Option<&[u8]>) -> Vec<u8> {
    let mut fields = vec![test_item(
        UNIQUE_IDENTIFIER,
        Value::text_string("multipart-object".to_owned()),
    )];
    if let Some(correlation_value) = correlation_value {
        fields.push(test_item(
            CORRELATION_VALUE,
            Value::byte_string(correlation_value.to_vec()),
        ));
    }
    asynchronous_response_bytes(operation, SUCCESS, None, None, Some(test_structure(fields)))
}

fn execute_one(client: &mut Client, request: ClientRequest) -> ClientBatchResponse {
    client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request)),
            &CodecLimits::defaults(),
        )
        .expect("one valid multipart request receives one typed response")
}

fn response_correlation(response: &ClientBatchResponse, operation: u32) -> Zeroizing<Vec<u8>> {
    let item = response
        .get(0)
        .expect("the one-item request has one corresponding result");
    let value = match operation {
        ENCRYPT_OPERATION => item
            .outcome()
            .response()
            .encrypt()
            .expect("the result is a typed Encrypt response")
            .correlation_value()
            .expect("the initial Encrypt response supplies Correlation Value"),
        DECRYPT_OPERATION => item
            .outcome()
            .response()
            .decrypt()
            .expect("the result is a typed Decrypt response")
            .correlation_value()
            .expect("the initial Decrypt response supplies Correlation Value"),
        _ => unreachable!("only Encrypt and Decrypt use this test helper"),
    };
    // Keep the response-byte copy zeroizing until ownership moves into SecretBytes.
    Zeroizing::new(value.with_bytes(<[u8]>::to_vec))
}

fn payload_fields(state: &SharedTransportState, request_index: usize) -> Vec<CapturedField> {
    let state = state.borrow();
    let request = state
        .requests
        .get(request_index)
        .expect("the explicit request reached the fake transport");
    let message = decode(request.as_slice()).expect("the captured request is valid TTLV");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("the captured KMIP request is a Structure");
        };
        let batch_item = message
            .children()
            .iter()
            .find(|item| item.tag().raw() == BATCH_ITEM)
            .expect("the request contains one Batch Item");
        batch_item.with_value(|value| {
            let ValueView::Structure(batch_item) = value else {
                panic!("the captured Batch Item is a Structure");
            };
            let request_payload = batch_item
                .children()
                .iter()
                .find(|item| item.tag().raw() == REQUEST_PAYLOAD)
                .expect("the Batch Item contains a Request Payload");
            request_payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("the Request Payload is a Structure");
                };
                payload
                    .children()
                    .iter()
                    .map(|field| {
                        let (byte_string, boolean) = field.with_value(|value| match value {
                            ValueView::ByteString(value) => {
                                (Some(Zeroizing::new(value.to_vec())), None)
                            }
                            ValueView::Boolean(value) => (None, Some(*value)),
                            _ => (None, None),
                        });
                        CapturedField {
                            tag: field.tag().raw(),
                            item_type: field.item_type(),
                            byte_string,
                            boolean,
                        }
                    })
                    .collect()
            })
        })
    })
}

fn assert_payload_byte_string(
    state: &SharedTransportState,
    request_index: usize,
    tag: u32,
    expected: &[u8],
) {
    let fields = payload_fields(state, request_index);
    let field = fields.iter().find(|field| field.tag == tag);
    assert!(
        field.is_some_and(|field| {
            field.item_type == ItemType::ByteString
                && field
                    .byte_string
                    .as_ref()
                    .is_some_and(|value| value.as_slice() == expected)
        }),
        "request field {tag:#010x} preserves its exact opaque bytes"
    );
}

fn assert_payload_boolean(
    state: &SharedTransportState,
    request_index: usize,
    tag: u32,
    expected: bool,
) {
    let fields = payload_fields(state, request_index);
    assert!(
        fields.iter().any(|field| {
            field.tag == tag
                && field.item_type == ItemType::Boolean
                && field.boolean == Some(expected)
        }),
        "request field {tag:#010x} has the expected Boolean value"
    );
}

fn assert_payload_field_absent(state: &SharedTransportState, request_index: usize, tag: u32) {
    let fields = payload_fields(state, request_index);
    assert!(
        fields.iter().all(|field| field.tag != tag),
        "request field {tag:#010x} is absent"
    );
}

fn assert_request_count(state: &SharedTransportState, expected: usize) {
    assert_eq!(
        state.borrow().requests.len(),
        expected,
        "each explicit caller invocation performs exactly one exchange"
    );
}

#[test]
fn client_accepts_framed_single_part_with_data_for_encrypt_and_decrypt() {
    let requests = [
        (
            ENCRYPT_OPERATION,
            ClientRequest::Encrypt(
                EncryptRequest::new(
                    Some(UniqueIdentifier::TextString("multipart-object".to_owned())),
                    Some(OperationData::ByteString(SecretBytes::new(
                        FRAMED_SINGLE_DATA.to_vec(),
                    ))),
                )
                .with_init_indicator(true)
                .with_final_indicator(true),
            ),
        ),
        (
            DECRYPT_OPERATION,
            ClientRequest::Decrypt(
                DecryptRequest::new(
                    Some(UniqueIdentifier::TextString("multipart-object".to_owned())),
                    Some(OperationData::ByteString(SecretBytes::new(
                        FRAMED_SINGLE_DATA.to_vec(),
                    ))),
                )
                .with_init_indicator(true)
                .with_final_indicator(true),
            ),
        ),
    ];

    for (operation, request) in requests {
        let (mut client, state) = client_with_responses([
            success_response(operation, None),
            success_response(operation, None),
            success_response(operation, None),
        ]);

        let response = execute_one(&mut client, request);

        assert_eq!(response.len(), 1);
        assert_request_count(&state, 1);
        assert_payload_boolean(&state, 0, INIT_INDICATOR, true);
        assert_payload_boolean(&state, 0, FINAL_INDICATOR, true);
        assert_payload_byte_string(&state, 0, DATA, FRAMED_SINGLE_DATA);
    }
}

#[test]
fn client_rejects_framed_single_part_without_data_before_exchange() {
    let requests = [
        (
            ENCRYPT_OPERATION,
            ClientRequest::Encrypt(
                EncryptRequest::new(
                    Some(UniqueIdentifier::TextString("multipart-object".to_owned())),
                    None,
                )
                .with_init_indicator(true)
                .with_final_indicator(true),
            ),
        ),
        (
            DECRYPT_OPERATION,
            ClientRequest::Decrypt(
                DecryptRequest::new(
                    Some(UniqueIdentifier::TextString("multipart-object".to_owned())),
                    None,
                )
                .with_init_indicator(true)
                .with_final_indicator(true),
            ),
        ),
    ];

    for (operation, request) in requests {
        let (mut client, state) = client_with_responses([
            success_response(operation, None),
            success_response(operation, None),
            success_response(operation, None),
        ]);

        let error = client
            .execute(
                ClientBatch::new(ClientBatchItem::new(request)),
                &CodecLimits::defaults(),
            )
            .expect_err("the ambiguous Data-omission form is rejected locally");

        assert_eq!(error.category(), ClientErrorCategory::Validation);
        assert_eq!(
            error.cause_category(),
            Some(ClientCauseCategory::InvalidInput)
        );
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        assert_eq!(
            error.to_string(),
            "client validation failure (invalid input, NotSent)"
        );
        assert_request_count(&state, 0);
    }
}

#[test]
fn encrypt_caller_reuses_initial_server_correlation_for_middle_and_final_parts() {
    let (mut client, state) = client_with_responses([
        success_response(ENCRYPT_OPERATION, Some(SERVER_CORRELATION)),
        success_response(ENCRYPT_OPERATION, None),
        success_response(ENCRYPT_OPERATION, None),
    ]);

    let initial = EncryptRequest::new(
        Some(UniqueIdentifier::TextString("multipart-object".to_owned())),
        Some(OperationData::ByteString(SecretBytes::new(
            b"encrypt-initial-data".to_vec(),
        ))),
    )
    .with_init_indicator(true)
    .with_authenticated_encryption_additional_data(SecretBytes::new(ENCRYPT_AAD.to_vec()));
    let initial_response = execute_one(&mut client, ClientRequest::Encrypt(initial));
    let initial_correlation: Zeroizing<Vec<u8>> =
        response_correlation(&initial_response, ENCRYPT_OPERATION);
    assert!(
        initial_correlation.as_slice() == SERVER_CORRELATION,
        "the typed first response exposes the exact server Correlation Value"
    );
    assert_request_count(&state, 1);
    assert_payload_boolean(&state, 0, INIT_INDICATOR, true);
    assert_payload_field_absent(&state, 0, CORRELATION_VALUE);
    assert_payload_byte_string(
        &state,
        0,
        AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA,
        ENCRYPT_AAD,
    );
    assert_payload_field_absent(&state, 0, AUTHENTICATED_ENCRYPTION_TAG);

    let middle = EncryptRequest::new(
        Some(UniqueIdentifier::TextString("multipart-object".to_owned())),
        Some(OperationData::ByteString(SecretBytes::new(
            b"encrypt-middle-data".to_vec(),
        ))),
    )
    .with_correlation_value(SecretBytes::new(initial_correlation.to_vec()));
    let _middle_response = execute_one(&mut client, ClientRequest::Encrypt(middle));
    assert_request_count(&state, 2);
    assert_payload_byte_string(&state, 1, CORRELATION_VALUE, SERVER_CORRELATION);
    assert_payload_field_absent(&state, 1, AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA);
    assert_payload_field_absent(&state, 1, AUTHENTICATED_ENCRYPTION_TAG);

    let final_part = EncryptRequest::new(
        Some(UniqueIdentifier::TextString("multipart-object".to_owned())),
        None,
    )
    .with_correlation_value(SecretBytes::new(initial_correlation.to_vec()));
    let final_part = final_part.with_final_indicator(true);
    let _final_response = execute_one(&mut client, ClientRequest::Encrypt(final_part));
    assert_request_count(&state, 3);
    assert_payload_byte_string(&state, 2, CORRELATION_VALUE, SERVER_CORRELATION);
    assert_payload_boolean(&state, 2, FINAL_INDICATOR, true);
    assert_payload_field_absent(&state, 2, AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA);
    assert_payload_field_absent(&state, 2, AUTHENTICATED_ENCRYPTION_TAG);
}

#[test]
fn decrypt_caller_reuses_initial_server_correlation_and_keeps_aad_and_tag_initial() {
    let (mut client, state) = client_with_responses([
        success_response(DECRYPT_OPERATION, Some(SERVER_CORRELATION)),
        success_response(DECRYPT_OPERATION, None),
        success_response(DECRYPT_OPERATION, None),
    ]);

    let initial = DecryptRequest::new(
        Some(UniqueIdentifier::TextString("multipart-object".to_owned())),
        Some(OperationData::ByteString(SecretBytes::new(
            b"decrypt-initial-data".to_vec(),
        ))),
    )
    .with_init_indicator(true)
    .with_authenticated_encryption_additional_data(SecretBytes::new(DECRYPT_AAD.to_vec()))
    .with_authenticated_encryption_tag(SecretBytes::new(DECRYPT_TAG.to_vec()));
    let initial_response = execute_one(&mut client, ClientRequest::Decrypt(initial));
    let initial_correlation: Zeroizing<Vec<u8>> =
        response_correlation(&initial_response, DECRYPT_OPERATION);
    assert!(
        initial_correlation.as_slice() == SERVER_CORRELATION,
        "the typed first response exposes the exact server Correlation Value"
    );
    assert_request_count(&state, 1);
    assert_payload_boolean(&state, 0, INIT_INDICATOR, true);
    assert_payload_field_absent(&state, 0, CORRELATION_VALUE);
    assert_payload_byte_string(
        &state,
        0,
        AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA,
        DECRYPT_AAD,
    );
    assert_payload_byte_string(&state, 0, AUTHENTICATED_ENCRYPTION_TAG, DECRYPT_TAG);

    let middle = DecryptRequest::new(
        Some(UniqueIdentifier::TextString("multipart-object".to_owned())),
        Some(OperationData::ByteString(SecretBytes::new(
            b"decrypt-middle-data".to_vec(),
        ))),
    )
    .with_correlation_value(SecretBytes::new(initial_correlation.to_vec()));
    let _middle_response = execute_one(&mut client, ClientRequest::Decrypt(middle));
    assert_request_count(&state, 2);
    assert_payload_byte_string(&state, 1, CORRELATION_VALUE, SERVER_CORRELATION);
    assert_payload_field_absent(&state, 1, AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA);
    assert_payload_field_absent(&state, 1, AUTHENTICATED_ENCRYPTION_TAG);

    let final_part = DecryptRequest::new(
        Some(UniqueIdentifier::TextString("multipart-object".to_owned())),
        None,
    )
    .with_correlation_value(SecretBytes::new(initial_correlation.to_vec()));
    let final_part = final_part.with_final_indicator(true);
    let _final_response = execute_one(&mut client, ClientRequest::Decrypt(final_part));
    assert_request_count(&state, 3);
    assert_payload_byte_string(&state, 2, CORRELATION_VALUE, SERVER_CORRELATION);
    assert_payload_boolean(&state, 2, FINAL_INDICATOR, true);
    assert_payload_field_absent(&state, 2, AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA);
    assert_payload_field_absent(&state, 2, AUTHENTICATED_ENCRYPTION_TAG);
}
