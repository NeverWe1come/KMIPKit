#![cfg(test)]

//! Fake-transport Recover execution checks derived from OASIS KMIP v2.1
//! §6.1.42, Tables 288–290; §4.58 Tables 145–146 define permitted Unique
//! Identifier encodings; §11.56 Table 487 assigns tag 0x420094 to Unique Identifier;
//! shared response and
//! asynchronous rules follow §§8.6, 9.1, and 9.2, Tables 399–401,
//! and the KMIPKIT-0007/0009 client contracts. Traceability:
//! `KMIPKIT-ELEM-OP-C2S-RECOVER`, KMIPKIT-0018 FR-001, FR-002, FR-005,
//! FR-006, FR-007, FR-008, FR-009, FR-010, SC-001, and SC-003.
//! Caller-selected Recover-before-Encrypt ordering is required by the archived
//! Managed Object paragraph in KMIP v2.1 §6.1 (`KMIPKIT-REQ-SPEC-6.1-005`;
//! KMIPKIT-0019 FR-010). These derived tests do not observe remote archive state.
//! These are derived structural/execution tests, not official OASIS Test Cases.

use kmipkit_protocol::{
    EncryptRequest, OperationData, RecoverRequest, SecretBytes, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Item, ItemType, Structure, Value, ValueView};
use zeroize::Zeroize;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;

use crate::asynchronous_execution_test_support::{client_for, request_contains};
use crate::execute::{Client, encode_message_for_test};
use crate::execute_test_support::{one_item_response_bytes, test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const RECOVER_OPERATION: u32 = 0x0000_002A;
const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;

const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const OBJECT_NOT_FOUND: u32 = 0x37; // Recover's Table 290 error reason.
const RESPONSE_MESSAGE_SENTINEL: &str = "KMIP_RECOVER_FAILURE_SENTINEL_3187";
const MALFORMED_RESPONSE_SENTINEL: &[u8] = b"KMIP_RECOVER_RAW_SENTINEL_4291";
const REQUEST_BATCH_ID: &[u8] = b"recover-batch-id";
const PENDING_CORRELATION: &[u8] = b"RECOVER_PENDING\x00CORRELATION\xff";
const RECOVER_ENCRYPT_IDENTIFIER: &str = "recover-encrypt-object";
const ENCRYPT_DATA: &[u8] = b"sequence-test-data";

fn response_bytes(
    status: u32,
    reason: Option<u32>,
    result_message: Option<&str>,
    correlation: Option<&[u8]>,
    batch_id: Option<&[u8]>,
    payload: Option<Structure>,
) -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);

    let mut batch_fields = vec![test_item(OPERATION, Value::enumeration(RECOVER_OPERATION))];
    if let Some(batch_id) = batch_id {
        batch_fields.push(test_item(
            UNIQUE_BATCH_ITEM_ID,
            Value::byte_string(batch_id.to_vec()),
        ));
    }
    batch_fields.push(test_item(RESULT_STATUS, Value::enumeration(status)));
    if let Some(reason) = reason {
        batch_fields.push(test_item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(message) = result_message {
        batch_fields.push(test_item(
            RESULT_MESSAGE,
            Value::text_string(message.to_owned()),
        ));
    }
    if let Some(correlation) = correlation {
        batch_fields.push(test_item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(correlation.to_vec()),
        ));
    }
    if let Some(payload) = payload {
        batch_fields.push(test_item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }

    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(test_structure(batch_fields))),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("the Recover response fixture has valid KMIP 2.1 TTLV framing")
}

fn success_payload(identifier: &str) -> Structure {
    test_structure([test_item(
        UNIQUE_IDENTIFIER,
        Value::text_string(identifier.to_owned()),
    )])
}

fn malformed_payloads() -> [Structure; 3] {
    [
        Structure::new(),
        test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::byte_string(MALFORMED_RESPONSE_SENTINEL.to_vec()),
        )]),
        test_structure([
            test_item(
                UNIQUE_IDENTIFIER,
                Value::text_string("duplicate-first".to_owned()),
            ),
            test_item(
                UNIQUE_IDENTIFIER,
                Value::text_string("duplicate-second".to_owned()),
            ),
        ]),
    ]
}

fn assert_recover_request(bytes: &[u8], expected_identifier: Option<&str>) {
    let message = decode(bytes).expect("the captured request is valid TTLV");
    let (operation, payload_fields) = message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("the request message is a Structure");
        };
        let batch_item = message
            .children()
            .iter()
            .find(|child| child.tag().raw() == BATCH_ITEM)
            .expect("the request has one Batch Item");
        batch_item.with_value(|value| {
            let ValueView::Structure(batch_item) = value else {
                panic!("the request Batch Item is a Structure");
            };
            let operation = batch_item
                .children()
                .iter()
                .find(|child| child.tag().raw() == OPERATION)
                .and_then(enumeration_value);
            let request_payload = batch_item
                .children()
                .iter()
                .find(|child| child.tag().raw() == REQUEST_PAYLOAD)
                .expect("the request has a Request Payload");
            let payload_fields = request_payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("the Request Payload is a Structure");
                };
                payload
                    .children()
                    .iter()
                    .map(|field| {
                        let text = field.with_value(|value| match value {
                            ValueView::TextString(text) => Some(text.to_owned()),
                            _ => None,
                        });
                        (field.tag().raw(), field.item_type(), text)
                    })
                    .collect::<Vec<_>>()
            });
            (operation, payload_fields)
        })
    });

    assert_eq!(operation, Some(RECOVER_OPERATION));
    match expected_identifier {
        Some(expected) => {
            assert_eq!(payload_fields.len(), 1);
            assert_eq!(payload_fields[0].0, UNIQUE_IDENTIFIER);
            assert_eq!(payload_fields[0].1, ItemType::TextString);
            assert_eq!(payload_fields[0].2.as_deref(), Some(expected));
        }
        None => assert_eq!(payload_fields.as_slice(), &[]),
    }
}

fn enumeration_value(item: &Item) -> Option<u32> {
    item.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    })
}

#[test]
fn recover_success_returns_completed_result_and_leaves_get_explicit() {
    for (request_identifier, response_identifier) in [
        (Some("requested-object"), "server-returned-object"),
        (None, "server-returned-placeholder-object"),
    ] {
        let response = response_bytes(
            SUCCESS,
            None,
            None,
            None,
            None,
            Some(success_payload(response_identifier)),
        );
        let (mut client, fake, request) = client_for(ExchangeScript::Success {
            response,
            request_write_chunks: vec![3, 5],
        });
        let typed_request = RecoverRequest::new(
            request_identifier.map(|value| UniqueIdentifier::TextString(value.to_owned())),
        );

        let response = client
            .recover(typed_request, &CodecLimits::defaults())
            .expect("Client::recover returns the server's typed Recover result");

        let outcome = response.outcome();
        let typed = outcome
            .response()
            .recover()
            .expect("the response view retains the Recover response");
        let response_view = outcome.response();
        assert_eq!(response_view.result(), typed.result());
        assert!(response_view.activate().is_none());
        assert!(response_view.archive().is_none());
        assert!(response_view.recover().is_some());
        assert!(response_view.destroy().is_none());
        assert!(outcome.activate_response().is_none());
        assert!(outcome.archive_response().is_none());
        assert!(outcome.recover_response().is_some());
        assert!(outcome.destroy_response().is_none());
        assert!(format!("{outcome}").starts_with("Recover("));
        assert_eq!(outcome.operation(), ClientOperation::Recover);
        assert_eq!(outcome.result().status().raw(), SUCCESS);
        assert!(!format!("{outcome:?}").contains(response_identifier));
        assert!(!format!("{:?}", outcome.response()).contains(response_identifier));
        assert_eq!(
            typed.unique_identifier(),
            Some(&UniqueIdentifier::TextString(
                response_identifier.to_owned()
            ))
        );
        // Recover completion is reported by the server; a later Get remains a
        // separate caller action.
        assert_recover_request(
            request
                .borrow()
                .as_ref()
                .expect("the fake transport captured the outbound request"),
            request_identifier,
        );
        assert_eq!(fake.borrow().exchange_count(), 1);
    }
}

#[test]
fn recover_failure_preserves_common_result_and_does_not_retry() {
    let response = response_bytes(
        OPERATION_FAILED,
        Some(OBJECT_NOT_FOUND),
        Some(RESPONSE_MESSAGE_SENTINEL),
        None,
        None,
        None,
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let response = client
        .recover(RecoverRequest::new(None), &CodecLimits::defaults())
        .expect("a valid server Failure remains a Recover result");
    let outcome = response.outcome();
    let typed = outcome
        .response()
        .recover()
        .expect("server Failure retains the operation-specific response");

    assert_eq!(outcome.operation(), ClientOperation::Recover);
    assert_eq!(typed.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        typed
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(OBJECT_NOT_FOUND)
    );
    assert_eq!(
        typed
            .result()
            .message()
            .map(kmipkit_protocol::ResultMessage::as_str),
        Some(RESPONSE_MESSAGE_SENTINEL)
    );
    assert!(typed.unique_identifier().is_none());
    assert!(!format!("{typed:?}").contains(RESPONSE_MESSAGE_SENTINEL));
    assert!(!format!("{outcome:?}").contains(RESPONSE_MESSAGE_SENTINEL));
    assert!(
        fake.borrow()
            .captured_logs()
            .iter()
            .all(|line| !line.contains(RESPONSE_MESSAGE_SENTINEL))
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn malformed_recover_success_is_a_redacted_protocol_error_after_one_exchange() {
    for payload in malformed_payloads() {
        let response = response_bytes(SUCCESS, None, None, None, None, Some(payload));
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        });

        let error = client
            .recover(RecoverRequest::new(None), &CodecLimits::defaults())
            .expect_err("a successful Recover response needs one Unique Identifier");

        assert_eq!(error.category(), ClientErrorCategory::Protocol);
        assert_eq!(
            error.delivery_state(),
            Some(RequestDeliveryState::ResponseStarted)
        );
        assert!(!error.to_string().contains("KMIP_RECOVER_RAW_SENTINEL_4291"));
        assert!(!format!("{error:?}").contains("KMIP_RECOVER_RAW_SENTINEL_4291"));
        assert!(
            fake.borrow()
                .captured_logs()
                .iter()
                .all(|line| !line.contains("KMIP_RECOVER_RAW_SENTINEL_4291"))
        );
        assert_eq!(fake.borrow().exchange_count(), 1);
    }
}

#[test]
fn recover_pending_preserves_exact_correlation_without_poll_or_follow_up() {
    let response = response_bytes(
        OPERATION_PENDING,
        None,
        None,
        Some(PENDING_CORRELATION),
        Some(REQUEST_BATCH_ID),
        Some(test_structure([])),
    );
    let (mut client, fake, request) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let typed_request = ClientRequest::Recover(RecoverRequest::new(None));
    let batch = ClientBatch::new(
        ClientBatchItem::new(typed_request).with_unique_batch_item_id(REQUEST_BATCH_ID.to_vec()),
    )
    .with_asynchronous_indicator(1);

    let result = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the caller permits the common KMIP Pending response");
    let item = result
        .get(0)
        .expect("the response is associated with Recover");
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("the permitted server Pending result remains Pending");
    };
    let response_view = item.outcome().response();
    assert_eq!(response_view.result(), pending.result());
    assert!(response_view.activate().is_none());
    assert!(response_view.archive().is_none());
    assert!(response_view.recover().is_some());
    assert!(response_view.destroy().is_none());

    assert_eq!(item.unique_batch_item_id(), Some(REQUEST_BATCH_ID));
    assert_eq!(pending.operation(), ClientOperation::Recover);
    assert_eq!(pending.result().status().raw(), OPERATION_PENDING);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(
        item.outcome().asynchronous_correlation_value(),
        Some(PENDING_CORRELATION)
    );
    assert!(request_contains(&request, REQUEST_BATCH_ID));
    assert!(!format!("{pending:?}").contains("RECOVER_PENDING"));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn recover_partial_write_failure_preserves_possibly_sent_delivery_without_retry() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 2 });

    let error = client
        .recover(RecoverRequest::new(None), &CodecLimits::defaults())
        .expect_err("a partial Recover request write returns a transport error");

    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SequenceRequestObservation {
    operation: Option<u32>,
    identifier_matches_expected: bool,
}

struct RecoverEncryptTransport {
    responses: VecDeque<Vec<u8>>,
    observations: Rc<RefCell<Vec<SequenceRequestObservation>>>,
}

impl Transport for RecoverEncryptTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.observations
            .borrow_mut()
            .push(observe_recover_encrypt_request(request));
        self.responses
            .pop_front()
            .map(TransportResponse::new)
            .ok_or_else(|| {
                TransportError::new(
                    RequestDeliveryState::NotSent,
                    TransportCauseCategory::Other,
                    io::Error::other("the Recover/Encrypt sequence has no scripted response"),
                )
            })
    }
}

impl Drop for RecoverEncryptTransport {
    fn drop(&mut self) {
        // Responses not consumed by the caller-gated sequence still contain raw TTLV.
        for response in &mut self.responses {
            response.zeroize();
        }
    }
}

fn observe_recover_encrypt_request(bytes: &[u8]) -> SequenceRequestObservation {
    let message = decode(bytes).unwrap_or_else(|_| panic!("the captured request is valid TTLV"));
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("the request message is a Structure");
        };
        let batch_item = message
            .children()
            .iter()
            .find(|child| child.tag().raw() == BATCH_ITEM)
            .unwrap_or_else(|| panic!("the request has one Batch Item"));
        batch_item.with_value(|value| {
            let ValueView::Structure(batch_item) = value else {
                panic!("the request Batch Item is a Structure");
            };
            let operation = batch_item
                .children()
                .iter()
                .find(|child| child.tag().raw() == OPERATION)
                .and_then(enumeration_value);
            let identifier_matches_expected = batch_item
                .children()
                .iter()
                .find(|child| child.tag().raw() == REQUEST_PAYLOAD)
                .is_some_and(|payload| {
                    payload.with_value(|value| {
                        let ValueView::Structure(payload) = value else {
                            return false;
                        };
                        payload
                            .children()
                            .iter()
                            .find(|field| field.tag().raw() == UNIQUE_IDENTIFIER)
                            .is_some_and(|field| {
                                field.with_value(|value| match value {
                                    ValueView::TextString(identifier) => {
                                        identifier == RECOVER_ENCRYPT_IDENTIFIER
                                    }
                                    _ => false,
                                })
                            })
                    })
                });
            SequenceRequestObservation {
                operation,
                identifier_matches_expected,
            }
        })
    })
}

fn recover_encrypt_client(
    responses: impl IntoIterator<Item = Vec<u8>>,
) -> (Client, Rc<RefCell<Vec<SequenceRequestObservation>>>) {
    let observations = Rc::new(RefCell::new(Vec::new()));
    let client = Client::for_test(RecoverEncryptTransport {
        responses: responses.into_iter().collect(),
        observations: Rc::clone(&observations),
    });
    (client, observations)
}

fn encrypt_success_response() -> Vec<u8> {
    one_item_response_bytes(
        ENCRYPT_OPERATION,
        SUCCESS,
        None,
        None,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string(RECOVER_ENCRYPT_IDENTIFIER.to_owned()),
        )])),
    )
}

#[test]
fn caller_executes_encrypt_only_after_completed_recover_using_the_returned_identifier() {
    let recover_response = response_bytes(
        SUCCESS,
        None,
        None,
        None,
        None,
        Some(success_payload(RECOVER_ENCRYPT_IDENTIFIER)),
    );
    let (mut client, observations) =
        recover_encrypt_client([recover_response, encrypt_success_response()]);

    let Ok(recover_result) = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Recover(
            RecoverRequest::new(Some(UniqueIdentifier::TextString(
                RECOVER_ENCRYPT_IDENTIFIER.to_owned(),
            ))),
        ))),
        &CodecLimits::defaults(),
    ) else {
        panic!("the first caller invocation completes Recover");
    };
    let recover_item = recover_result
        .get(0)
        .expect("the Recover response is associated with its request item");
    assert_eq!(recover_item.outcome().operation(), ClientOperation::Recover);
    assert_eq!(recover_item.outcome().result().status().raw(), SUCCESS);
    let ClientBatchOutcome::Recover(recovered) = recover_item.outcome() else {
        panic!("a completed Recover response uses its typed operation view");
    };
    let recovered_identifier = recovered
        .unique_identifier()
        .cloned()
        .expect("a successful Recover supplies the identifier for the caller's next request");
    assert!(matches!(
        &recovered_identifier,
        UniqueIdentifier::TextString(identifier) if identifier == RECOVER_ENCRYPT_IDENTIFIER
    ));

    let Ok(encrypt_result) = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Encrypt(
            EncryptRequest::new(
                Some(recovered_identifier),
                Some(OperationData::ByteString(SecretBytes::new(
                    ENCRYPT_DATA.to_vec(),
                ))),
            ),
        ))),
        &CodecLimits::defaults(),
    ) else {
        panic!("the second caller invocation executes Encrypt after Recover Success");
    };
    let encrypt_item = encrypt_result
        .get(0)
        .expect("the Encrypt response is associated with its request item");
    assert_eq!(encrypt_item.outcome().operation(), ClientOperation::Encrypt);
    assert_eq!(encrypt_item.outcome().result().status().raw(), SUCCESS);
    assert!(matches!(
        encrypt_item.outcome(),
        ClientBatchOutcome::Encrypt(_)
    ));
    assert_eq!(
        observations.borrow().as_slice(),
        &[
            SequenceRequestObservation {
                operation: Some(RECOVER_OPERATION),
                identifier_matches_expected: true,
            },
            SequenceRequestObservation {
                operation: Some(ENCRYPT_OPERATION),
                identifier_matches_expected: true,
            },
        ]
    );
    assert_eq!(observations.borrow().len(), 2);
}

#[test]
fn caller_does_not_execute_encrypt_after_recover_failure() {
    let recover_response = response_bytes(
        OPERATION_FAILED,
        Some(OBJECT_NOT_FOUND),
        Some(RESPONSE_MESSAGE_SENTINEL),
        None,
        None,
        None,
    );
    let (mut client, observations) =
        recover_encrypt_client([recover_response, encrypt_success_response()]);

    let Ok(recover_result) = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Recover(
            RecoverRequest::new(Some(UniqueIdentifier::TextString(
                RECOVER_ENCRYPT_IDENTIFIER.to_owned(),
            ))),
        ))),
        &CodecLimits::defaults(),
    ) else {
        panic!("a valid Recover Failure remains an operation result");
    };
    let recover_item = recover_result
        .get(0)
        .expect("the Recover response is associated with its request item");
    assert_eq!(recover_item.outcome().operation(), ClientOperation::Recover);
    assert_eq!(
        recover_item.outcome().result().status().raw(),
        OPERATION_FAILED
    );
    assert!(matches!(
        recover_item.outcome(),
        ClientBatchOutcome::Recover(_)
    ));

    if recover_item.outcome().result().status().raw() == SUCCESS {
        let recovered_identifier = match recover_item.outcome() {
            ClientBatchOutcome::Recover(recovered) => recovered
                .unique_identifier()
                .cloned()
                .unwrap_or_else(|| panic!("a successful Recover supplies its identifier")),
            _ => panic!("only a Recover outcome can pass the caller's gate"),
        };
        let _ = client.execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::Encrypt(
                EncryptRequest::new(
                    Some(recovered_identifier),
                    Some(OperationData::ByteString(SecretBytes::new(
                        ENCRYPT_DATA.to_vec(),
                    ))),
                ),
            ))),
            &CodecLimits::defaults(),
        );
    }

    assert_eq!(observations.borrow().len(), 1);
    assert_eq!(
        observations.borrow().as_slice(),
        &[SequenceRequestObservation {
            operation: Some(RECOVER_OPERATION),
            identifier_matches_expected: true,
        }]
    );
}

#[test]
fn caller_does_not_execute_encrypt_after_recover_pending() {
    let recover_response = response_bytes(
        OPERATION_PENDING,
        None,
        None,
        Some(PENDING_CORRELATION),
        Some(REQUEST_BATCH_ID),
        Some(test_structure([])),
    );
    let (mut client, observations) =
        recover_encrypt_client([recover_response, encrypt_success_response()]);
    let recover_batch = ClientBatch::new(
        ClientBatchItem::new(ClientRequest::Recover(RecoverRequest::new(Some(
            UniqueIdentifier::TextString(RECOVER_ENCRYPT_IDENTIFIER.to_owned()),
        ))))
        .with_unique_batch_item_id(REQUEST_BATCH_ID.to_vec()),
    )
    .with_asynchronous_indicator(1);

    let Ok(recover_result) = client.execute(recover_batch, &CodecLimits::defaults()) else {
        panic!("a valid asynchronous Recover Pending response remains explicit");
    };
    let recover_item = recover_result
        .get(0)
        .expect("the Recover response is associated with its request item");
    assert_eq!(recover_item.outcome().operation(), ClientOperation::Recover);
    assert_eq!(
        recover_item.outcome().result().status().raw(),
        OPERATION_PENDING
    );
    assert!(matches!(
        recover_item.outcome(),
        ClientBatchOutcome::Pending(_)
    ));
    assert_eq!(
        recover_item.outcome().asynchronous_correlation_value(),
        Some(PENDING_CORRELATION)
    );
    let pending_response = recover_item.outcome().response();
    let typed_recover = pending_response
        .recover()
        .expect("Pending retains a payload-free Recover response view");
    assert!(typed_recover.unique_identifier().is_none());

    if recover_item.outcome().result().status().raw() == SUCCESS {
        let recovered_identifier = match recover_item.outcome() {
            ClientBatchOutcome::Recover(recovered) => recovered
                .unique_identifier()
                .cloned()
                .unwrap_or_else(|| panic!("a successful Recover supplies its identifier")),
            _ => panic!("only a Recover outcome can pass the caller's gate"),
        };
        let _ = client.execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::Encrypt(
                EncryptRequest::new(
                    Some(recovered_identifier),
                    Some(OperationData::ByteString(SecretBytes::new(
                        ENCRYPT_DATA.to_vec(),
                    ))),
                ),
            ))),
            &CodecLimits::defaults(),
        );
    }

    assert_eq!(observations.borrow().len(), 1);
    assert_eq!(
        observations.borrow().as_slice(),
        &[SequenceRequestObservation {
            operation: Some(RECOVER_OPERATION),
            identifier_matches_expected: true,
        }]
    );
}
