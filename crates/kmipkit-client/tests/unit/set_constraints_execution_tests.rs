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
const CONSTRAINT: u32 = 0x0042_0169;
const ATTRIBUTES: u32 = 0x0042_0025;
const ATTRIBUTE: u32 = 0x0042_0008;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const ATTRIBUTE_NAME: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE: u32 = 0x0042_000B;
const VENDOR_MEMBER: u32 = 0x0054_1234;
const DEBUG_SENTINEL: &str = "P025-DEBUG-SECRET-SENTINEL";

fn request(constraints: Structure) -> ClientBatch {
    ClientBatch::new(ClientBatchItem::new(ClientRequest::set_constraints(
        SetConstraintsRequest::new(constraints),
    )))
}

fn constraints() -> Structure {
    let vendor_attribute = test_structure([
        test_item(
            VENDOR_IDENTIFICATION,
            Value::text_string("KMIPKit test vendor".to_owned()),
        ),
        test_item(
            ATTRIBUTE_NAME,
            Value::text_string("Debug Redaction Sentinel".to_owned()),
        ),
        test_item(
            ATTRIBUTE_VALUE,
            Value::byte_string(b"preserve this nested member".to_vec()),
        ),
    ]);
    let attributes = test_structure([test_item(ATTRIBUTE, Value::structure(vendor_attribute))]);
    let constraint = test_structure([test_item(ATTRIBUTES, Value::structure(attributes))]);
    test_structure([test_item(CONSTRAINT, Value::structure(constraint))])
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
                    assert_eq!(constraints.children().len(), 1);
                    assert_eq!(constraints.children()[0].tag().raw(), CONSTRAINT);
                    constraints.children()[0].with_value(|value| {
                        let ValueView::Structure(constraint) = value else {
                            panic!("Constraint is a Structure");
                        };
                        assert_eq!(constraint.children().len(), 1);
                        assert_eq!(constraint.children()[0].tag().raw(), ATTRIBUTES);
                        constraint.children()[0].with_value(|value| {
                            let ValueView::Structure(attributes) = value else {
                                panic!("Attributes is a Structure");
                            };
                            assert_eq!(attributes.children().len(), 1);
                            assert_eq!(attributes.children()[0].tag().raw(), ATTRIBUTE);
                            attributes.children()[0].with_value(|value| {
                                let ValueView::Structure(attribute) = value else {
                                    panic!("Vendor Attribute is a Structure");
                                };
                                assert_eq!(attribute.children().len(), 3);
                                assert_eq!(
                                    attribute
                                        .children()
                                        .iter()
                                        .map(|field| field.tag().raw())
                                        .collect::<Vec<_>>(),
                                    [VENDOR_IDENTIFICATION, ATTRIBUTE_NAME, ATTRIBUTE_VALUE]
                                );
                                assert!(attribute.children()[2].with_value(|value| matches!(
                                    value,
                                    ValueView::ByteString(bytes)
                                        if bytes == b"preserve this nested member"
                                )));
                            });
                        });
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
fn set_constraints_outcome_exposes_typed_views_and_safe_diagnostics() {
    // Table 326 defines an empty Success payload. This extra unknown child is
    // intentionally a robustness-only, non-conformant fixture that checks the
    // typed Debug wrappers redact retained raw response TTLV.
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            SET_CONSTRAINTS,
            0,
            None,
            None,
            Some(test_structure([test_item(
                VENDOR_MEMBER,
                Value::text_string(DEBUG_SENTINEL.to_owned()),
            )])),
        ),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .execute(request(constraints()), &CodecLimits::defaults())
        .expect("Set Constraints succeeds");
    let outcome = response.get(0).expect("one response").outcome();
    let view = outcome.response();

    assert_eq!(outcome.operation(), crate::ClientOperation::SetConstraints);
    assert_eq!(outcome.result().status().raw(), 0);
    assert_eq!(view.result().status().raw(), 0);
    assert!(view.get_constraints().is_none());
    assert!(view.set_constraints().is_some());
    assert!(view.set_defaults().is_none());
    assert!(view.get_usage_allocation().is_none());
    let outcome_debug = format!("{outcome:?}");
    let view_debug = format!("{view:?}");
    assert!(outcome_debug.starts_with("SetConstraints("));
    assert!(view_debug.starts_with("SetConstraints("));
    for formatted in [&outcome_debug, &view_debug] {
        assert!(formatted.contains("[REDACTED]"), "{formatted}");
        assert!(!formatted.contains(DEBUG_SENTINEL), "{formatted}");
    }
    assert_eq!(
        outcome.to_string(),
        format!("SetConstraints({})", outcome.result())
    );
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
    let view = outcome.response();
    assert!(view.set_constraints().is_some());
    assert_eq!(view.result().status().raw(), 2);
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
