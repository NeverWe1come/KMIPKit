//! Client-boundary redaction characterization for KMIP v2.1 §6.1.11
//! Tables 196–197 (Decrypt), §6.1.17 Tables 214–215 (Encrypt), and §§7.3,
//! 7.4, 7.8, and 7.9. Traceability: KMIPKIT-0019-FR-009 and SC-004.

use std::cell::Cell;
use std::error::Error;
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

use crate::execute::{Client, encode_message_for_test};
use crate::execute_test_support::{test_item, test_structure};
use crate::{ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientError, ClientRequest};

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const SUCCESS: u32 = 0;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const DATA: u32 = 0x0042_00C2;
const IV_COUNTER_NONCE: u32 = 0x0042_003D;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const AUTHENTICATED_ENCRYPTION_TAG: u32 = 0x0042_00FF;

const ENCRYPT_REQUEST_DATA: &str = "KMIP_T036_CLIENT_ENCRYPT_REQUEST_DATA_47A2";
const DECRYPT_REQUEST_DATA: &str = "KMIP_T036_CLIENT_DECRYPT_REQUEST_DATA_B182";
const ENCRYPT_REQUEST_IV: &str = "KMIP_T036_CLIENT_ENCRYPT_REQUEST_IV_22FC";
const DECRYPT_REQUEST_IV: &str = "KMIP_T036_CLIENT_DECRYPT_REQUEST_IV_193D";
const ENCRYPT_REQUEST_CORRELATION: &str = "KMIP_T036_CLIENT_ENCRYPT_REQUEST_CORRELATION_5E79";
const DECRYPT_REQUEST_CORRELATION: &str = "KMIP_T036_CLIENT_DECRYPT_REQUEST_CORRELATION_64A0";
const ENCRYPT_REQUEST_AAD: &str = "KMIP_T036_CLIENT_ENCRYPT_REQUEST_AAD_17EB";
const DECRYPT_REQUEST_AAD: &str = "KMIP_T036_CLIENT_DECRYPT_REQUEST_AAD_0C35";
const DECRYPT_REQUEST_TAG: &str = "KMIP_T036_CLIENT_DECRYPT_REQUEST_TAG_9D5A";
const ENCRYPT_RESPONSE_DATA: &str = "KMIP_T036_CLIENT_ENCRYPT_RESPONSE_DATA_35D8";
const DECRYPT_RESPONSE_DATA: &str = "KMIP_T036_CLIENT_DECRYPT_RESPONSE_DATA_8B61";
const ENCRYPT_RESPONSE_IV: &str = "KMIP_T036_CLIENT_ENCRYPT_RESPONSE_IV_5FA3";
const ENCRYPT_RESPONSE_CORRELATION: &str = "KMIP_T036_CLIENT_ENCRYPT_RESPONSE_CORRELATION_2091";
const DECRYPT_RESPONSE_CORRELATION: &str = "KMIP_T036_CLIENT_DECRYPT_RESPONSE_CORRELATION_716C";
const ENCRYPT_RESPONSE_TAG: &str = "KMIP_T036_CLIENT_ENCRYPT_RESPONSE_TAG_AE04";
const RAW_RESPONSE_SENTINEL: &str = "KMIP_T036_CLIENT_RAW_RESPONSE_BODY_4AA8";
const MALFORMED_RESPONSE_SENTINEL: &str = "KMIP_T036_CLIENT_MALFORMED_DATA_108E";

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

    fn request(self) -> (ClientRequest, String) {
        let identifier = Some(UniqueIdentifier::TextString("t036-client-key".to_owned()));
        match self {
            Self::Encrypt => {
                let request = EncryptRequest::new(
                    identifier,
                    Some(OperationData::ByteString(SecretBytes::new(
                        ENCRYPT_REQUEST_DATA.as_bytes().to_vec(),
                    ))),
                )
                .with_iv_counter_nonce(SecretBytes::new(ENCRYPT_REQUEST_IV.as_bytes().to_vec()))
                .with_correlation_value(SecretBytes::new(
                    ENCRYPT_REQUEST_CORRELATION.as_bytes().to_vec(),
                ))
                .with_final_indicator(true)
                .with_authenticated_encryption_additional_data(SecretBytes::new(
                    ENCRYPT_REQUEST_AAD.as_bytes().to_vec(),
                ));
                let protocol_debug = format!("{request:?}");
                (ClientRequest::Encrypt(request), protocol_debug)
            }
            Self::Decrypt => {
                let request = DecryptRequest::new(
                    identifier,
                    Some(OperationData::ByteString(SecretBytes::new(
                        DECRYPT_REQUEST_DATA.as_bytes().to_vec(),
                    ))),
                )
                .with_iv_counter_nonce(SecretBytes::new(DECRYPT_REQUEST_IV.as_bytes().to_vec()))
                .with_correlation_value(SecretBytes::new(
                    DECRYPT_REQUEST_CORRELATION.as_bytes().to_vec(),
                ))
                .with_final_indicator(true)
                .with_authenticated_encryption_additional_data(SecretBytes::new(
                    DECRYPT_REQUEST_AAD.as_bytes().to_vec(),
                ))
                .with_authenticated_encryption_tag(SecretBytes::new(
                    DECRYPT_REQUEST_TAG.as_bytes().to_vec(),
                ));
                let protocol_debug = format!("{request:?}");
                (ClientRequest::Decrypt(request), protocol_debug)
            }
        }
    }

    const fn request_sentinels(self) -> &'static [&'static str] {
        match self {
            Self::Encrypt => &[
                ENCRYPT_REQUEST_DATA,
                ENCRYPT_REQUEST_IV,
                ENCRYPT_REQUEST_CORRELATION,
                ENCRYPT_REQUEST_AAD,
            ],
            Self::Decrypt => &[
                DECRYPT_REQUEST_DATA,
                DECRYPT_REQUEST_IV,
                DECRYPT_REQUEST_CORRELATION,
                DECRYPT_REQUEST_AAD,
                DECRYPT_REQUEST_TAG,
            ],
        }
    }
}

const OPERATIONS: [CryptoOperation; 2] = [CryptoOperation::Encrypt, CryptoOperation::Decrypt];

struct ResponseTransport {
    response: Zeroizing<Vec<u8>>,
    exchanges: Rc<Cell<usize>>,
}

impl Transport for ResponseTransport {
    fn exchange(
        &mut self,
        _request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.exchanges.set(self.exchanges.get().saturating_add(1));
        Ok(TransportResponse::new(self.response.to_vec()))
    }
}

struct EchoRequestFailureTransport {
    sentinels_seen: Rc<Cell<bool>>,
    expected_sentinels: &'static [&'static str],
}

impl Transport for EchoRequestFailureTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.sentinels_seen
            .set(self.expected_sentinels.iter().all(|sentinel| {
                request
                    .windows(sentinel.len())
                    .any(|window| window == sentinel.as_bytes())
            }));
        let source = io::Error::other(String::from_utf8_lossy(request).into_owned());
        Err(TransportError::new(
            RequestDeliveryState::PossiblySent,
            TransportCauseCategory::Io,
            source,
        ))
    }
}

fn payload(items: impl IntoIterator<Item = kmipkit_ttlv::Item>) -> Structure {
    let mut payload = Structure::new();
    for item in items {
        payload
            .try_push(item)
            .expect("the redaction test payload fits TTLV structural limits");
    }
    payload
}

fn response_bytes(operation: CryptoOperation, response_payload: Option<Structure>) -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut batch = vec![
        test_item(OPERATION, Value::enumeration(operation.raw())),
        test_item(RESULT_STATUS, Value::enumeration(SUCCESS)),
    ];
    if let Some(response_payload) = response_payload {
        batch.push(test_item(
            RESPONSE_PAYLOAD,
            Value::structure(response_payload),
        ));
    }
    let message = test_structure([
        test_item(RESPONSE_HEADER, Value::structure(header)),
        test_item(BATCH_ITEM, Value::structure(test_structure(batch))),
    ]);
    encode_message_for_test(message, &CodecLimits::defaults())
        .expect("the test response fits default TTLV limits")
}

fn successful_payload(operation: CryptoOperation) -> Structure {
    let unique_identifier = test_item(
        UNIQUE_IDENTIFIER,
        Value::text_string("t036-client-output".to_owned()),
    );
    match operation {
        CryptoOperation::Encrypt => payload([
            unique_identifier,
            test_item(
                DATA,
                Value::byte_string(ENCRYPT_RESPONSE_DATA.as_bytes().to_vec()),
            ),
            test_item(
                IV_COUNTER_NONCE,
                Value::byte_string(ENCRYPT_RESPONSE_IV.as_bytes().to_vec()),
            ),
            test_item(
                CORRELATION_VALUE,
                Value::byte_string(ENCRYPT_RESPONSE_CORRELATION.as_bytes().to_vec()),
            ),
            test_item(
                AUTHENTICATED_ENCRYPTION_TAG,
                Value::byte_string(ENCRYPT_RESPONSE_TAG.as_bytes().to_vec()),
            ),
        ]),
        CryptoOperation::Decrypt => payload([
            unique_identifier,
            test_item(
                DATA,
                Value::byte_string(DECRYPT_RESPONSE_DATA.as_bytes().to_vec()),
            ),
            test_item(
                CORRELATION_VALUE,
                Value::byte_string(DECRYPT_RESPONSE_CORRELATION.as_bytes().to_vec()),
            ),
        ]),
    }
}

fn execute_with_response(
    operation: CryptoOperation,
    response: Vec<u8>,
) -> (
    Result<crate::ClientBatchResponse, ClientError>,
    Rc<Cell<usize>>,
) {
    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(ResponseTransport {
        response: Zeroizing::new(response),
        exchanges: Rc::clone(&exchanges),
    });
    let (request, _) = operation.request();
    let result = client.execute(
        ClientBatch::new(ClientBatchItem::new(request)),
        &CodecLimits::defaults(),
    );
    (result, exchanges)
}

fn assert_absent(formatted: &str, sentinels: &[&str], context: &str) {
    for sentinel in sentinels {
        assert!(
            !formatted.contains(sentinel),
            "{context} must redact values"
        );
    }
}

fn format_error_chain(error: &(dyn Error + 'static)) -> String {
    let mut formatted = String::new();
    let mut source = Some(error);
    while let Some(cause) = source {
        formatted.push_str(&cause.to_string());
        source = cause.source();
    }
    formatted
}

fn assert_client_secret_redacted(secret: &SecretBytes, sentinel: &str) {
    assert!(
        secret.with_bytes(|bytes| bytes == sentinel.as_bytes()),
        "the typed client response retains the sentinel bytes"
    );
    let debug = format!("{secret:?}");
    let display = secret.to_string();
    assert_absent(&debug, &[sentinel], "client SecretBytes Debug");
    assert_absent(&display, &[sentinel], "client SecretBytes Display");
}

#[test]
fn client_and_protocol_request_debug_redact_all_encrypt_and_decrypt_inputs() {
    for operation in OPERATIONS {
        let (request, protocol_debug) = operation.request();
        assert_absent(
            &protocol_debug,
            operation.request_sentinels(),
            "typed protocol request Debug",
        );
        let client_debug = format!("{request:?}");
        assert_absent(
            &client_debug,
            operation.request_sentinels(),
            "typed client request Debug",
        );
    }
}

#[test]
fn client_response_debug_and_secret_display_redact_encrypt_and_decrypt_outputs() {
    for operation in OPERATIONS {
        let response = response_bytes(operation, Some(successful_payload(operation)));
        let (result, exchanges) = execute_with_response(operation, response);
        let response = result.expect("the successful cryptographic response is well formed");
        let item = response
            .get(0)
            .expect("one cryptographic response item is associated");
        let outcome = item.outcome();
        let formatted = format!("{outcome:?}{:?}", outcome.response());
        let sentinels: &[&str] = match operation {
            CryptoOperation::Encrypt => &[
                ENCRYPT_RESPONSE_DATA,
                ENCRYPT_RESPONSE_IV,
                ENCRYPT_RESPONSE_CORRELATION,
                ENCRYPT_RESPONSE_TAG,
            ],
            CryptoOperation::Decrypt => &[DECRYPT_RESPONSE_DATA, DECRYPT_RESPONSE_CORRELATION],
        };
        assert_absent(&formatted, sentinels, "typed client response Debug");

        match (operation, outcome) {
            (CryptoOperation::Encrypt, ClientBatchOutcome::Encrypt(response)) => {
                assert_client_secret_redacted(
                    response.data().expect("Encrypt Data is present"),
                    ENCRYPT_RESPONSE_DATA,
                );
                assert_client_secret_redacted(
                    response.iv_counter_nonce().expect("Encrypt IV is present"),
                    ENCRYPT_RESPONSE_IV,
                );
                assert_client_secret_redacted(
                    response
                        .correlation_value()
                        .expect("Encrypt Correlation Value is present"),
                    ENCRYPT_RESPONSE_CORRELATION,
                );
                assert_client_secret_redacted(
                    response
                        .authenticated_encryption_tag()
                        .expect("Encrypt AEAD Tag is present"),
                    ENCRYPT_RESPONSE_TAG,
                );
            }
            (CryptoOperation::Decrypt, ClientBatchOutcome::Decrypt(response)) => {
                assert_client_secret_redacted(
                    response.data().expect("Decrypt Data is present"),
                    DECRYPT_RESPONSE_DATA,
                );
                assert_client_secret_redacted(
                    response
                        .correlation_value()
                        .expect("Decrypt Correlation Value is present"),
                    DECRYPT_RESPONSE_CORRELATION,
                );
            }
            _ => panic!("the typed client response matches the operation"),
        }
        assert_eq!(exchanges.get(), 1, "one request uses one exchange");
    }
}

#[test]
fn client_errors_and_source_chains_redact_malformed_payload_values() {
    for operation in OPERATIONS {
        let malformed = payload([test_item(
            DATA,
            Value::text_string(MALFORMED_RESPONSE_SENTINEL.to_owned()),
        )]);
        let response = response_bytes(operation, Some(malformed));
        let (result, exchanges) = execute_with_response(operation, response);
        let Err(error) = result else {
            panic!("wrong-type cryptographic success Data is rejected");
        };
        let debug = format!("{error:?}");
        let display = error.to_string();
        let chain = format_error_chain(&error);
        assert_absent(&debug, &[MALFORMED_RESPONSE_SENTINEL], "client error Debug");
        assert_absent(
            &display,
            &[MALFORMED_RESPONSE_SENTINEL],
            "client error Display",
        );
        assert_absent(&chain, &[MALFORMED_RESPONSE_SENTINEL], "client error chain");
        assert_eq!(exchanges.get(), 1, "malformed response is not retried");
    }
}

#[test]
fn client_errors_and_source_chains_redact_raw_request_and_response_bodies() {
    for operation in OPERATIONS {
        let (request, _) = operation.request();
        let sentinels_seen = Rc::new(Cell::new(false));
        let mut client = Client::for_test(EchoRequestFailureTransport {
            sentinels_seen: Rc::clone(&sentinels_seen),
            expected_sentinels: operation.request_sentinels(),
        });
        let Err(error) = client.execute(
            ClientBatch::new(ClientBatchItem::new(request)),
            &CodecLimits::defaults(),
        ) else {
            panic!("the fake transport reports the request I/O failure");
        };
        assert!(
            sentinels_seen.get(),
            "the fake request body contains every operation sentinel"
        );
        let debug = format!("{error:?}");
        let display = error.to_string();
        let chain = format_error_chain(&error);
        assert_absent(&debug, operation.request_sentinels(), "request error Debug");
        assert_absent(
            &display,
            operation.request_sentinels(),
            "request error Display",
        );
        assert_absent(&chain, operation.request_sentinels(), "request error chain");

        let (result, exchanges) =
            execute_with_response(operation, RAW_RESPONSE_SENTINEL.as_bytes().to_vec());
        let Err(error) = result else {
            panic!("raw response bytes are rejected as malformed TTLV");
        };
        let debug = format!("{error:?}");
        let display = error.to_string();
        let chain = format_error_chain(&error);
        assert_absent(&debug, &[RAW_RESPONSE_SENTINEL], "response error Debug");
        assert_absent(&display, &[RAW_RESPONSE_SENTINEL], "response error Display");
        assert_absent(&chain, &[RAW_RESPONSE_SENTINEL], "response error chain");
        assert_eq!(exchanges.get(), 1, "invalid raw response is not retried");
    }
}
