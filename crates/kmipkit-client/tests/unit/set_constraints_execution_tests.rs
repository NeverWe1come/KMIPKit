//! Project-derived fake-client checks for OASIS KMIP Specification v2.1
//! §6.1.52, Tables 325–327; Constraints structure §7.7.
//! Traceability: KMIPKIT-025-T325-REQUEST, T326-RESPONSE, T327-ERROR,
//! KMIPKIT-025-COMMON-RESULT, KMIPKIT-025-COMMON-DELIVERY.
//! These are not official OASIS Test Cases. The error mapping follows the
//! maintainer-approved KMIPKIT-DISC-020 interpretation in `spec.md`.

use kmipkit_protocol::{ResultReason, SetConstraintsRequest};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{ItemType, Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientRequest};

const SET_CONSTRAINTS: u32 = 0x0000_0037;
const CONSTRAINTS: u32 = 0x0042_0168;
const VENDOR_MEMBER: u32 = 0x0054_1234;

fn request(constraints: Structure) -> ClientBatch {
    ClientBatch::new(ClientBatchItem::new(ClientRequest::set_constraints(
        SetConstraintsRequest::new(constraints),
    )))
}

fn constraints() -> Structure {
    test_structure([
        test_item(VENDOR_MEMBER, Value::integer(7)),
        test_item(
            VENDOR_MEMBER,
            Value::structure(test_structure([test_item(
                VENDOR_MEMBER,
                Value::text_string("preserve this nested member".to_owned()),
            )])),
        ),
    ])
}

#[test]
fn table_325_request_is_required_ordered_and_sent_once() {
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            SET_CONSTRAINTS,
            0,
            None,
            None,
            Some(test_structure([])),
        ),
        request_write_chunks: Vec::new(),
    });
    let result = client
        .execute(request(constraints()), &CodecLimits::defaults())
        .expect("valid required Constraints is sent through one typed exchange");
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(result.len(), 1);

    let wire = captured.borrow();
    let decoded = decode(wire.as_ref().expect("request was captured"))
        .expect("request is bounded valid TTLV");
    decoded.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("request Message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .find(|field| field.tag().raw() == 0x0042_000F)
            .expect("one Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("Batch Item is a Structure");
            };
            assert!(batch.children().iter().any(|field| {
                field.tag().raw() == 0x0042_005C
                    && field.with_value(|value| {
                        matches!(value, ValueView::Enumeration(raw) if *raw == SET_CONSTRAINTS)
                    })
            }));
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == 0x0042_0079)
                .expect("Table 325 requires Request Payload");
            assert_eq!(payload.item_type(), ItemType::Structure);
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                assert_eq!(payload.children().len(), 1);
                assert_eq!(payload.children()[0].tag().raw(), CONSTRAINTS);
                payload.children()[0].with_value(|value| {
                    let ValueView::Structure(constraints) = value else {
                        panic!("Constraints is a Structure");
                    };
                    assert_eq!(constraints.children().len(), 2);
                    assert_eq!(constraints.children()[0].tag().raw(), VENDOR_MEMBER);
                    assert_eq!(constraints.children()[1].tag().raw(), VENDOR_MEMBER);
                    assert!(
                        constraints.children()[0]
                            .with_value(|value| matches!(value, ValueView::Integer(7)))
                    );
                    constraints.children()[1].with_value(|value| {
                        let ValueView::Structure(nested) = value else {
                            panic!("second Constraints child is a Structure");
                        };
                        assert_eq!(nested.children().len(), 1);
                        assert!(nested.children()[0].with_value(|value| matches!(
                            value,
                            ValueView::TextString("preserve this nested member")
                        )));
                    });
                });
            });
        });
    });
}

#[test]
fn typed_set_constraints_convenience_call_returns_the_operation_result() {
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            SET_CONSTRAINTS,
            0,
            None,
            None,
            Some(test_structure([])),
        ),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .set_constraints(
            SetConstraintsRequest::new(constraints()),
            &CodecLimits::defaults(),
        )
        .expect("the typed convenience method executes one request");
    assert_eq!(fake.borrow().exchange_count(), 1);
    let outcome = response.outcome();
    let response_view = outcome.response();
    let typed = response_view.set_constraints().expect("typed result");
    assert_eq!(typed.result().status().raw(), 0);
}

#[test]
fn table_327_failure_alternatives_are_preserved_without_retry() {
    // Table 327 repeats Invalid Field. KMIPKIT-DISC-020 associates this error
    // introduction with Set Constraints; the table's reasons remain binding.
    for reason in [
        0x07,
        0x30,
        0x15,
        0x14,
        0x08,
        0x04,
        0x05,
        0x0C,
        0x02,
        0x8000_0054,
    ] {
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response: asynchronous_response_bytes(SET_CONSTRAINTS, 1, Some(reason), None, None),
            request_write_chunks: Vec::new(),
        });
        let result = client
            .execute(request(constraints()), &CodecLimits::defaults())
            .expect("Operation Failed remains a typed KMIP result");
        let outcome = result.get(0).expect("one result").outcome();
        assert_eq!(fake.borrow().exchange_count(), 1);
        let response_view = outcome.response();
        let response = response_view.set_constraints().expect("typed result");
        assert_eq!(response.result().status().raw(), 1);
        assert_eq!(
            response.result().reason(),
            Some(ResultReason::from_raw(reason))
        );
    }
}

#[test]
fn caller_enabled_pending_preserves_correlation_and_does_not_poll() {
    let correlation = b"set-constraints-correlation";
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            SET_CONSTRAINTS,
            2,
            None,
            Some(correlation),
            Some(test_structure([])),
        ),
        request_write_chunks: Vec::new(),
    });
    let result = client
        .execute(
            request(constraints()).with_asynchronous_indicator(1),
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
        .execute(request(constraints()), &limits)
        .expect_err("unencodable local request never reaches the transport");
    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);
}
