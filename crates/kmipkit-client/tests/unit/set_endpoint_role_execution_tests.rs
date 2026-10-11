//! Project-derived fake-client checks for OASIS KMIP Specification v2.1
//! §6.1.54, Tables 331–333, and Endpoint Role §11.19, Tables 451–452.
//! Traceability: KMIPKIT-025-T331-REQUEST, T332-RESPONSE, T333-ERROR,
//! KMIPKIT-025-COMMON-RESULT, KMIPKIT-025-COMMON-DELIVERY.
//! These are not official OASIS Test Cases. Only the client-to-server §6.1.54
//! operation is exposed; server-initiated §6.2.5 is outside this API.

use kmipkit_protocol::{EndpointRole, ResultReason, SetEndpointRoleRequest};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{ItemType, Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{
    asynchronous_response_bytes, one_item_response_bytes, test_item, test_structure,
};
use crate::{ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientRequest};

const SET_ENDPOINT_ROLE: u32 = 0x0000_0032;
const ENDPOINT_ROLE: u32 = 0x0042_0151;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const SERVER_MESSAGE: &str = "opaque-set-endpoint-role-error";

fn request(raw_role: u32) -> ClientBatch {
    ClientBatch::new(ClientBatchItem::new(ClientRequest::set_endpoint_role(
        SetEndpointRoleRequest::new(EndpointRole::from_raw(raw_role)),
    )))
}

fn response_payload(raw_role: u32) -> Structure {
    test_structure([test_item(ENDPOINT_ROLE, Value::enumeration(raw_role))])
}

fn assert_captured_request_role(request_bytes: &[u8], raw_role: u32) {
    let decoded = decode(request_bytes).expect("outbound request is valid TTLV");
    decoded.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("request Message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .find(|field| field.tag().raw() == BATCH_ITEM)
            .expect("one Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("Batch Item is a Structure");
            };
            assert!(batch.children().iter().any(|field| {
                field.tag().raw() == OPERATION
                    && field.with_value(|value| {
                        matches!(value, ValueView::Enumeration(raw) if *raw == SET_ENDPOINT_ROLE)
                    })
            }));
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("Table 331 Request Payload is present");
            assert_eq!(payload.item_type(), ItemType::Structure);
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                assert_eq!(payload.children().len(), 1);
                let role = &payload.children()[0];
                assert_eq!(role.tag().raw(), ENDPOINT_ROLE);
                assert_eq!(role.item_type(), ItemType::Enumeration);
                assert!(role.with_value(|value| matches!(
                    value,
                    ValueView::Enumeration(actual) if *actual == raw_role
                )));
            });
        });
    });
}

#[test]
fn table_331_sends_client_operation_and_known_or_extension_role_once() {
    // Table 452 values 1 and 2 plus a vendor extension in its 8xxxxxxx range.
    for raw_role in [1, 2, 3, 0x8000_0054] {
        let (mut client, fake, captured) = client_for(ExchangeScript::Success {
            response: asynchronous_response_bytes(
                SET_ENDPOINT_ROLE,
                0,
                None,
                None,
                Some(response_payload(raw_role)),
            ),
            request_write_chunks: Vec::new(),
        });
        client
            .execute(request(raw_role), &CodecLimits::defaults())
            .expect("valid required Endpoint Role is sent");
        assert_eq!(fake.borrow().exchange_count(), 1);
        let captured = captured.borrow();
        let request_bytes = captured.as_ref().expect("request was captured").as_slice();
        assert_captured_request_role(request_bytes, raw_role);
    }
}

#[test]
fn table_332_success_preserves_the_server_accepted_role() {
    for raw_role in [1, 2, 3, 0x8000_0054] {
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response: asynchronous_response_bytes(
                SET_ENDPOINT_ROLE,
                0,
                None,
                None,
                Some(response_payload(raw_role)),
            ),
            request_write_chunks: Vec::new(),
        });
        let result = client
            .execute(request(raw_role), &CodecLimits::defaults())
            .expect("Table 332 response returns the accepted role");
        assert_eq!(fake.borrow().exchange_count(), 1);
        let outcome = result.get(0).expect("one response").outcome();
        let ClientBatchOutcome::SetEndpointRole(response) = outcome else {
            panic!("Set Endpoint Role returns a typed outcome");
        };
        assert_eq!(
            response.endpoint_role(),
            Some(EndpointRole::from_raw(raw_role))
        );
        response.with_ttlv(|item| {
            assert!(item.children().iter().any(|field| {
                field.tag().raw() == RESPONSE_PAYLOAD
                    && field.with_value(|value| matches!(value, ValueView::Structure(payload) if payload.children().len() == 1))
            }));
        });
    }
}

#[test]
fn client_convenience_method_executes_set_endpoint_role_once() {
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            SET_ENDPOINT_ROLE,
            0,
            None,
            None,
            Some(response_payload(1)),
        ),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .set_endpoint_role(
            SetEndpointRoleRequest::new(EndpointRole::from_raw(1)),
            &CodecLimits::defaults(),
        )
        .expect("typed convenience method executes one request");
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(
        response
            .outcome()
            .response()
            .set_endpoint_role()
            .expect("typed response")
            .endpoint_role(),
        Some(EndpointRole::from_raw(1))
    );
}

#[test]
fn table_333_failure_reasons_and_message_are_preserved_without_retry() {
    for reason in [0x0C, 0x15, 0x14, 0x08, 0x07, 0x04, 0x05, 0x02, 0x8000_0054] {
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response: one_item_response_bytes(
                SET_ENDPOINT_ROLE,
                1,
                Some(reason),
                Some(SERVER_MESSAGE),
                None,
                None,
                None,
            ),
            request_write_chunks: Vec::new(),
        });
        let result = client
            .execute(request(1), &CodecLimits::defaults())
            .expect("Operation Failed remains a typed result");
        assert_eq!(fake.borrow().exchange_count(), 1);
        let outcome = result.get(0).expect("one response").outcome();
        let ClientBatchOutcome::SetEndpointRole(response) = outcome else {
            panic!("Set Endpoint Role returns a typed outcome");
        };
        assert_eq!(response.result().status().raw(), 1);
        assert_eq!(
            response.result().reason(),
            Some(ResultReason::from_raw(reason))
        );
        assert_eq!(response.endpoint_role(), None);
        assert_eq!(
            response
                .result()
                .message()
                .map(kmipkit_protocol::ResultMessage::as_str),
            Some(SERVER_MESSAGE)
        );
        assert!(!format!("{outcome:?}").contains(SERVER_MESSAGE));
        assert!(!outcome.to_string().contains(SERVER_MESSAGE));
    }
}

#[test]
fn caller_enabled_pending_preserves_correlation_without_polling() {
    let correlation = b"set-endpoint-role-correlation";
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            SET_ENDPOINT_ROLE,
            2,
            None,
            Some(correlation),
            Some(test_structure([])),
        ),
        request_write_chunks: Vec::new(),
    });
    let result = client
        .execute(
            request(1).with_asynchronous_indicator(1),
            &CodecLimits::defaults(),
        )
        .expect("caller-enabled Pending returns after one exchange");
    assert_eq!(fake.borrow().exchange_count(), 1);
    let outcome = result.get(0).expect("one response").outcome();
    assert!(matches!(outcome, ClientBatchOutcome::Pending(_)));
    assert_eq!(
        outcome.asynchronous_correlation_value(),
        Some(&correlation[..])
    );
}

#[test]
fn local_encoding_failure_is_not_sent_and_never_retried() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let limits = CodecLimits::new(0, 64, 100_000).expect("zero byte message limit is valid");
    let error = client
        .execute(request(1), &limits)
        .expect_err("unencodable local request never reaches the transport");
    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);
}
