//! Typed client outcome coverage derived from KMIP v2.1 §6.1.11 Table 197
//! (Decrypt), §6.1.17 Table 215 (Encrypt), §9.17 (Result Message), and §9.18
//! (Result Reason). Traceability: KMIPKIT-0019-FR-005, FR-007, FR-008, and
//! KMIPKIT-REQ-SPEC-6.1-001-002. UID-only success complements the fixture
//! round trips in `encrypt_decrypt_fixture_execution_tests`; Pending routing
//! is covered for both operations in `encrypt_decrypt_multipart_execution_tests`.

use std::cell::Cell;
use std::io;
use std::rc::Rc;

use kmipkit_protocol::{
    DecryptRequest, EncryptRequest, OperationData, SecretBytes, UniqueIdentifier,
};
use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Structure, Value};
use zeroize::Zeroizing;

use crate::execute::Client;
use crate::execute_test_support::{one_item_response_bytes, test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientBatchResponse, ClientError,
    ClientErrorCategory, ClientOperation, ClientRequest,
};

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const OPERATION_FAILED: u32 = 1;
const SUCCESS: u32 = 0;
const UNKNOWN_RESULT_REASON: u32 = 0xDEAD_BEEF;

const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const DATA: u32 = 0x0042_00C2;

const RESULT_MESSAGE_TEXT: &str = "server rejected the cryptographic operation";
const RESPONSE_IDENTIFIER: &str = "crypto-outcome-object";

#[derive(Clone, Copy)]
enum CryptoOperation {
    Encrypt,
    Decrypt,
}

impl CryptoOperation {
    const fn raw(self) -> u32 {
        match self {
            Self::Encrypt => ENCRYPT_OPERATION,
            Self::Decrypt => DECRYPT_OPERATION,
        }
    }

    const fn client_operation(self) -> ClientOperation {
        match self {
            Self::Encrypt => ClientOperation::Encrypt,
            Self::Decrypt => ClientOperation::Decrypt,
        }
    }

    fn request(self) -> ClientRequest {
        let identifier = Some(UniqueIdentifier::TextString(
            "crypto-outcome-request".to_owned(),
        ));
        let data = Some(OperationData::ByteString(SecretBytes::new(
            b"request".to_vec(),
        )));
        match self {
            Self::Encrypt => ClientRequest::Encrypt(EncryptRequest::new(identifier, data)),
            Self::Decrypt => ClientRequest::Decrypt(DecryptRequest::new(identifier, data)),
        }
    }
}

const OPERATIONS: [CryptoOperation; 2] = [CryptoOperation::Encrypt, CryptoOperation::Decrypt];

struct OneResponseTransport {
    response: Option<Zeroizing<Vec<u8>>>,
    exchange_count: Rc<Cell<usize>>,
}

impl Transport for OneResponseTransport {
    fn exchange(
        &mut self,
        _request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.exchange_count
            .set(self.exchange_count.get().saturating_add(1));
        let Some(response) = self.response.take() else {
            return Err(TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("unexpected additional operation exchange"),
            ));
        };
        Ok(TransportResponse::new(response.to_vec()))
    }
}

fn client_for(response: Vec<u8>) -> (Client, Rc<Cell<usize>>) {
    let exchange_count = Rc::new(Cell::new(0));
    let client = Client::for_test(OneResponseTransport {
        response: Some(Zeroizing::new(response)),
        exchange_count: Rc::clone(&exchange_count),
    });
    (client, exchange_count)
}

fn execute(
    operation: CryptoOperation,
    response: Vec<u8>,
    limits: &CodecLimits,
) -> (Result<ClientBatchResponse, ClientError>, Rc<Cell<usize>>) {
    let (mut client, exchange_count) = client_for(response);
    let result = client.execute(
        ClientBatch::new(ClientBatchItem::new(operation.request())),
        limits,
    );
    (result, exchange_count)
}

fn response_bytes(
    operation: CryptoOperation,
    result_status: u32,
    result_reason: Option<u32>,
    result_message: Option<&str>,
    response_payload: Option<Structure>,
) -> Vec<u8> {
    one_item_response_bytes(
        operation.raw(),
        result_status,
        result_reason,
        result_message,
        None,
        None,
        response_payload,
    )
}

fn uid_only_payload() -> Structure {
    test_structure([test_item(
        UNIQUE_IDENTIFIER,
        Value::text_string(RESPONSE_IDENTIFIER.to_owned()),
    )])
}

#[test]
fn client_accepts_uid_only_success_payload_for_encrypt_and_decrypt() {
    for operation in OPERATIONS {
        let response = response_bytes(operation, SUCCESS, None, None, Some(uid_only_payload()));
        let (result, exchange_count) = execute(operation, response, &CodecLimits::defaults());
        let response = result.expect("UID-only success is valid for either operation");
        let item = response.get(0).expect("one response item is associated");
        assert_eq!(item.outcome().operation(), operation.client_operation());

        match (operation, item.outcome()) {
            (CryptoOperation::Encrypt, ClientBatchOutcome::Encrypt(response)) => {
                assert_eq!(response.result().status().raw(), SUCCESS);
                assert!(matches!(
                    response.unique_identifier(),
                    Some(UniqueIdentifier::TextString(value)) if value == RESPONSE_IDENTIFIER
                ));
                assert!(response.data().is_none());
                assert!(response.iv_counter_nonce().is_none());
                assert!(response.correlation_value().is_none());
                assert!(response.authenticated_encryption_tag().is_none());
            }
            (CryptoOperation::Decrypt, ClientBatchOutcome::Decrypt(response)) => {
                assert_eq!(response.result().status().raw(), SUCCESS);
                assert!(matches!(
                    response.unique_identifier(),
                    Some(UniqueIdentifier::TextString(value)) if value == RESPONSE_IDENTIFIER
                ));
                assert!(response.data().is_none());
                assert!(response.correlation_value().is_none());
            }
            _ => panic!("client outcome matches the requested crypto operation"),
        }
        assert_eq!(exchange_count.get(), 1);
    }
}

#[test]
fn client_preserves_unknown_failure_reason_and_message_for_encrypt_and_decrypt() {
    for operation in OPERATIONS {
        let response = response_bytes(
            operation,
            OPERATION_FAILED,
            Some(UNKNOWN_RESULT_REASON),
            Some(RESULT_MESSAGE_TEXT),
            None,
        );
        let (result, exchange_count) = execute(operation, response, &CodecLimits::defaults());
        let response = result.expect("an operation failure remains a typed batch outcome");
        let item = response.get(0).expect("one response item is associated");
        assert_eq!(item.outcome().operation(), operation.client_operation());

        match (operation, item.outcome()) {
            (CryptoOperation::Encrypt, ClientBatchOutcome::Encrypt(response)) => {
                assert_eq!(response.result().status().raw(), OPERATION_FAILED);
                assert_eq!(
                    response
                        .result()
                        .reason()
                        .map(kmipkit_protocol::ResultReason::raw),
                    Some(UNKNOWN_RESULT_REASON)
                );
                assert_eq!(
                    response
                        .result()
                        .message()
                        .map(kmipkit_protocol::ResultMessage::as_str),
                    Some(RESULT_MESSAGE_TEXT)
                );
                assert!(response.unique_identifier().is_none());
                assert!(response.data().is_none());
                assert!(response.iv_counter_nonce().is_none());
                assert!(response.correlation_value().is_none());
                assert!(response.authenticated_encryption_tag().is_none());
            }
            (CryptoOperation::Decrypt, ClientBatchOutcome::Decrypt(response)) => {
                assert_eq!(response.result().status().raw(), OPERATION_FAILED);
                assert_eq!(
                    response
                        .result()
                        .reason()
                        .map(kmipkit_protocol::ResultReason::raw),
                    Some(UNKNOWN_RESULT_REASON)
                );
                assert_eq!(
                    response
                        .result()
                        .message()
                        .map(kmipkit_protocol::ResultMessage::as_str),
                    Some(RESULT_MESSAGE_TEXT)
                );
                assert!(response.unique_identifier().is_none());
                assert!(response.data().is_none());
                assert!(response.correlation_value().is_none());
            }
            _ => panic!("client outcome matches the requested crypto operation"),
        }
        assert_eq!(exchange_count.get(), 1);
    }
}

#[test]
fn client_rejects_malformed_success_payloads_after_one_exchange() {
    for operation in OPERATIONS {
        // A successful Table 197/215 payload requires Unique Identifier.
        let malformed_payload = test_structure([test_item(
            DATA,
            Value::byte_string(b"response-data".to_vec()),
        )]);
        let response = response_bytes(operation, SUCCESS, None, None, Some(malformed_payload));
        let (result, exchange_count) = execute(operation, response, &CodecLimits::defaults());
        let error = result.expect_err("success payloads without Unique Identifier are malformed");
        assert_eq!(error.category(), ClientErrorCategory::Protocol);
        assert_eq!(
            error.delivery_state(),
            Some(RequestDeliveryState::ResponseStarted)
        );
        assert_eq!(
            exchange_count.get(),
            1,
            "malformed responses are not retried"
        );
    }
}

#[test]
fn client_applies_local_decoder_message_limit_to_encrypt_and_decrypt_responses() {
    let limits = CodecLimits::new(256, 64, 100_000)
        .expect("the test's depth and element bounds are within supported limits");
    for operation in OPERATIONS {
        let payload = test_structure([
            test_item(
                UNIQUE_IDENTIFIER,
                Value::text_string(RESPONSE_IDENTIFIER.to_owned()),
            ),
            test_item(DATA, Value::byte_string(vec![0xA5; 512])),
        ]);
        let response = response_bytes(operation, SUCCESS, None, None, Some(payload));
        assert!(response.len() > limits.max_message_bytes());

        let (result, exchange_count) = execute(operation, response, &limits);
        let error = result.expect_err("responses over the caller limit are rejected");
        assert_eq!(error.category(), ClientErrorCategory::Protocol);
        assert_eq!(
            error.delivery_state(),
            Some(RequestDeliveryState::ResponseStarted)
        );
        assert_eq!(
            exchange_count.get(),
            1,
            "oversized responses are not retried"
        );
    }
}
