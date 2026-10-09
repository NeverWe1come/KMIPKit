#![cfg(test)]

//! Fake-transport Destroy execution checks derived from OASIS KMIP v2.1
//! §6.1.15, Tables 208–210; Unique Identifier encodings follow §4.58 Tables
//! 145–146, and §11.56 Table 487 assigns the tag; shared response and
//! asynchronous rules follow §§8.6, 9.1, and 9.2, Tables
//! 399–401; §11.3/Table 431; and the KMIPKIT-0007/0009 client contracts.
//! Traceability:
//! `KMIPKIT-ELEM-OP-C2S-DESTROY`, KMIPKIT-0018 FR-001, FR-002, FR-003,
//! FR-006, FR-007, FR-008, SC-001, and SC-003.
//! `KMIPKIT-CLAUSE-SPEC-6.1.15-001` is server-only. These are derived
//! structural/execution tests, not official OASIS Test Cases.

use kmipkit_protocol::{DestroyRequest, UniqueIdentifier};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Item, ItemType, Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::{client_for, request_contains};
use crate::execute::encode_message_for_test;
use crate::execute_test_support::{test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const DESTROY_OPERATION: u32 = 0x0000_0014;
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
const OBJECT_NOT_FOUND: u32 = 0x37; // Destroy's Table 210 error reason.
const RESPONSE_MESSAGE_SENTINEL: &str = "KMIP_DESTROY_FAILURE_SENTINEL_3187";
const MALFORMED_RESPONSE_SENTINEL: &[u8] = b"KMIP_DESTROY_RAW_SENTINEL_4291";
const REQUEST_BATCH_ID: &[u8] = b"destroy-batch-id";
const PENDING_CORRELATION: &[u8] = b"DESTROY_PENDING\x00CORRELATION\xff";

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

    let mut batch_fields = vec![test_item(OPERATION, Value::enumeration(DESTROY_OPERATION))];
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
    .expect("the Destroy response fixture has valid KMIP 2.1 TTLV framing")
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

fn assert_destroy_request(bytes: &[u8], expected_identifier: Option<&str>) {
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

    assert_eq!(operation, Some(DESTROY_OPERATION));
    match expected_identifier {
        Some(expected) => {
            assert_eq!(payload_fields.len(), 1);
            assert_eq!(payload_fields[0].0, UNIQUE_IDENTIFIER);
            assert_eq!(payload_fields[0].1, ItemType::TextString);
            assert_eq!(payload_fields[0].2.as_deref(), Some(expected));
        }
        None => assert!(payload_fields.is_empty()),
    }
}

fn enumeration_value(item: &Item) -> Option<u32> {
    item.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    })
}

#[test]
fn destroy_success_returns_server_result_without_simulating_object_state() {
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
        let typed_request = DestroyRequest::new(
            request_identifier.map(|value| UniqueIdentifier::TextString(value.to_owned())),
        );

        let response = client
            .destroy(typed_request, &CodecLimits::defaults())
            .expect("Client::destroy returns the server's typed Destroy result");

        let outcome = response.outcome();
        let typed = outcome
            .response()
            .destroy()
            .expect("the response view retains the Destroy response");
        assert_eq!(outcome.operation(), ClientOperation::Destroy);
        assert_eq!(outcome.result().status().raw(), SUCCESS);
        assert_eq!(
            typed.unique_identifier(),
            Some(&UniqueIdentifier::TextString(
                response_identifier.to_owned()
            ))
        );
        // The server-only state effect from KMIPKIT-CLAUSE-SPEC-6.1.15-001 is
        // not represented as local state; the client returns only the KMIP
        // result and identifier required by the successful response.
        assert_destroy_request(
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
fn destroy_failure_preserves_common_result_and_does_not_retry() {
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
        .destroy(DestroyRequest::new(None), &CodecLimits::defaults())
        .expect("a valid server Failure remains an Destroy result");
    let outcome = response.outcome();
    let typed = outcome
        .response()
        .destroy()
        .expect("server Failure retains the operation-specific response");

    assert_eq!(outcome.operation(), ClientOperation::Destroy);
    assert_eq!(typed.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        typed.result().reason().map(|reason| reason.raw()),
        Some(OBJECT_NOT_FOUND)
    );
    assert_eq!(
        typed.result().message().map(|message| message.as_str()),
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
fn malformed_destroy_success_is_a_redacted_protocol_error_after_one_exchange() {
    for payload in malformed_payloads() {
        let response = response_bytes(SUCCESS, None, None, None, None, Some(payload));
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        });

        let error = client
            .destroy(DestroyRequest::new(None), &CodecLimits::defaults())
            .expect_err("a successful Destroy response needs one Unique Identifier");

        assert_eq!(error.category(), ClientErrorCategory::Protocol);
        assert_eq!(
            error.delivery_state(),
            Some(RequestDeliveryState::ResponseStarted)
        );
        assert!(!error.to_string().contains("KMIP_DESTROY_RAW_SENTINEL_4291"));
        assert!(!format!("{error:?}").contains("KMIP_DESTROY_RAW_SENTINEL_4291"));
        assert!(
            fake.borrow()
                .captured_logs()
                .iter()
                .all(|line| !line.contains("KMIP_DESTROY_RAW_SENTINEL_4291"))
        );
        assert_eq!(fake.borrow().exchange_count(), 1);
    }
}

#[test]
fn destroy_pending_preserves_exact_correlation_without_poll_or_follow_up() {
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
    let request = ClientRequest::Destroy(DestroyRequest::new(None));
    let batch = ClientBatch::new(
        ClientBatchItem::new(request).with_unique_batch_item_id(REQUEST_BATCH_ID.to_vec()),
    )
    .with_asynchronous_indicator(1);

    let result = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the caller permits the common KMIP Pending response");
    let item = result
        .get(0)
        .expect("the response is associated with Destroy");
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("the permitted server Pending result remains Pending");
    };

    assert_eq!(item.unique_batch_item_id(), Some(REQUEST_BATCH_ID));
    assert_eq!(pending.operation(), ClientOperation::Destroy);
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
    assert!(!format!("{pending:?}").contains("DESTROY_PENDING"));
    assert_eq!(fake.borrow().exchange_count(), 1);
}
