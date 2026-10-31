//! Project-derived fake-client checks for OASIS KMIP Specification v2.1
//! §6.1.53, Tables 328–330 and Defaults Information §7.11.
//! Traceability: KMIPKIT-025-T328-REQUEST, T329-RESPONSE, T330-ERROR,
//! KMIPKIT-025-COMMON-RESULT, KMIPKIT-025-COMMON-DELIVERY.
//! These are not official OASIS Test Cases. Table 328 explicitly states the
//! absent-field behavior; the approved KMIPKIT-DISC-020 disposition applies
//! to Table 330's truncated error introduction.

use kmipkit_protocol::{ResultReason, SetDefaultsRequest};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{ItemType, Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{
    asynchronous_response_bytes, one_item_response_bytes, test_item, test_structure,
};
use crate::{ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientRequest};

const SET_DEFAULTS: u32 = 0x0000_0036;
const DEFAULTS_INFORMATION: u32 = 0x0042_0152;
const VENDOR_MEMBER: u32 = 0x0054_1234;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const BATCH_ITEM: u32 = 0x0042_000F;
const SERVER_MESSAGE: &str = "opaque-set-defaults-error";
const DEBUG_SENTINEL: &str = "P025-DEBUG-SECRET-SENTINEL";

fn request(defaults_information: Option<Structure>) -> ClientBatch {
    let mut request = SetDefaultsRequest::new();
    if let Some(information) = defaults_information {
        request = request.with_defaults_information(information);
    }
    ClientBatch::new(ClientBatchItem::new(ClientRequest::set_defaults(request)))
}

#[test]
fn table_328_omission_and_explicit_empty_structure_are_distinct_on_the_wire() {
    for (defaults_information, expected_children) in
        [(None, None), (Some(Structure::new()), Some(0))]
    {
        let (mut client, fake, captured) = client_for(ExchangeScript::Success {
            response: asynchronous_response_bytes(
                SET_DEFAULTS,
                0,
                None,
                None,
                Some(test_structure([])),
            ),
            request_write_chunks: Vec::new(),
        });
        let result = client
            .execute(request(defaults_information), &CodecLimits::defaults())
            .expect("Table 328 optional field is sent through one typed exchange");
        assert_eq!(fake.borrow().exchange_count(), 1);
        assert_eq!(result.len(), 1);

        let wire = captured.borrow();
        let decoded = decode(wire.as_ref().expect("request was captured"))
            .expect("outbound request is valid TTLV");
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
                let payload = batch
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                    .expect("Table 328 has a Request Payload");
                payload.with_value(|value| {
                    let ValueView::Structure(payload) = value else {
                        panic!("Request Payload is a Structure");
                    };
                    match expected_children {
                        None => assert!(payload.children().is_empty()),
                        Some(children) => {
                            assert_eq!(payload.children().len(), 1);
                            let defaults = &payload.children()[0];
                            assert_eq!(defaults.tag().raw(), DEFAULTS_INFORMATION);
                            assert_eq!(defaults.item_type(), ItemType::Structure);
                            defaults.with_value(|value| {
                                let ValueView::Structure(defaults) = value else {
                                    panic!("Defaults Information is a Structure");
                                };
                                assert_eq!(defaults.children().len(), children);
                            });
                        }
                    }
                });
            });
        });
    }
}

#[test]
fn table_328_populated_defaults_information_keeps_order_and_nested_values() {
    let information = test_structure([
        test_item(VENDOR_MEMBER, Value::integer(3)),
        test_item(
            VENDOR_MEMBER,
            Value::structure(test_structure([test_item(
                VENDOR_MEMBER,
                Value::text_string("nested-default".to_owned()),
            )])),
        ),
    ]);
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            SET_DEFAULTS,
            0,
            None,
            None,
            Some(test_structure([])),
        ),
        request_write_chunks: Vec::new(),
    });
    client
        .execute(request(Some(information)), &CodecLimits::defaults())
        .expect("generic Defaults Information is sent");
    assert_eq!(fake.borrow().exchange_count(), 1);

    let wire = captured.borrow();
    let decoded = decode(wire.as_ref().expect("request was captured"))
        .expect("outbound request is valid TTLV");
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
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("Request Payload is present");
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                let defaults = payload
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == DEFAULTS_INFORMATION)
                    .expect("supplied Defaults Information remains present");
                defaults.with_value(|value| {
                    let ValueView::Structure(defaults) = value else {
                        panic!("Defaults Information is a Structure");
                    };
                    assert_eq!(defaults.children().len(), 2);
                    assert!(
                        defaults.children()[0]
                            .with_value(|value| matches!(value, ValueView::Integer(3)))
                    );
                    defaults.children()[1].with_value(|value| {
                        let ValueView::Structure(nested) = value else {
                            panic!("second Defaults Information child is a Structure");
                        };
                        assert!(nested.children()[0].with_value(|value| matches!(
                            value,
                            ValueView::TextString("nested-default")
                        )));
                    });
                });
            });
        });
    });
}

#[test]
fn set_defaults_outcome_exposes_typed_views_and_safe_diagnostics() {
    // Table 329 defines an empty Success payload. This extra unknown child is
    // intentionally a robustness-only, non-conformant fixture that checks the
    // typed Debug wrappers redact retained raw response TTLV.
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            SET_DEFAULTS,
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
        .execute(request(None), &CodecLimits::defaults())
        .expect("Set Defaults succeeds");
    let outcome = response.get(0).expect("one response").outcome();
    let view = outcome.response();

    assert_eq!(outcome.operation(), crate::ClientOperation::SetDefaults);
    assert_eq!(outcome.result().status().raw(), 0);
    assert_eq!(view.result().status().raw(), 0);
    assert!(view.get_constraints().is_none());
    assert!(view.set_constraints().is_none());
    assert!(view.set_defaults().is_some());
    assert!(view.get_usage_allocation().is_none());
    let outcome_debug = format!("{outcome:?}");
    let view_debug = format!("{view:?}");
    assert!(outcome_debug.starts_with("SetDefaults("));
    assert!(view_debug.starts_with("SetDefaults("));
    for formatted in [&outcome_debug, &view_debug] {
        assert!(formatted.contains("[REDACTED]"), "{formatted}");
        assert!(!formatted.contains(DEBUG_SENTINEL), "{formatted}");
    }
    assert_eq!(
        outcome.to_string(),
        format!("SetDefaults({})", outcome.result())
    );
}

#[test]
fn table_329_success_returns_typed_result_with_empty_payload() {
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            SET_DEFAULTS,
            0,
            None,
            None,
            Some(test_structure([])),
        ),
        request_write_chunks: Vec::new(),
    });
    let result = client
        .execute(request(None), &CodecLimits::defaults())
        .expect("Table 329 success returns a typed result");
    assert_eq!(fake.borrow().exchange_count(), 1);
    let outcome = result.get(0).expect("one response").outcome();
    let ClientBatchOutcome::SetDefaults(response) = outcome else {
        panic!("Set Defaults returns a typed outcome");
    };
    assert_eq!(response.result().status().raw(), 0);
    assert!(response.with_ttlv(|item| {
        assert!(item.children().iter().any(|field| {
            field.tag().raw() == RESPONSE_PAYLOAD
                && field.with_value(|value| matches!(value, ValueView::Structure(payload) if payload.children().is_empty()))
        }));
    })
    .is_some());
}

#[test]
fn typed_set_defaults_convenience_call_returns_operation_result() {
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            SET_DEFAULTS,
            0,
            None,
            None,
            Some(test_structure([])),
        ),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .set_defaults(SetDefaultsRequest::new(), &CodecLimits::defaults())
        .expect("typed convenience method executes one request");
    assert_eq!(fake.borrow().exchange_count(), 1);
    let outcome = response.outcome();
    let response_view = outcome.response();
    assert_eq!(
        response_view
            .set_defaults()
            .expect("typed Set Defaults response")
            .result()
            .status()
            .raw(),
        0
    );
}

#[test]
fn table_330_failure_reason_message_and_unknown_reason_are_preserved_without_retry() {
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
            response: one_item_response_bytes(
                SET_DEFAULTS,
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
            .execute(request(None), &CodecLimits::defaults())
            .expect("Operation Failed remains a typed KMIP result");
        assert_eq!(fake.borrow().exchange_count(), 1);
        let outcome = result.get(0).expect("one response").outcome();
        let ClientBatchOutcome::SetDefaults(response) = outcome else {
            panic!("Set Defaults returns a typed outcome");
        };
        assert_eq!(response.result().status().raw(), 1);
        assert_eq!(
            response.result().reason(),
            Some(ResultReason::from_raw(reason))
        );
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
    let correlation = b"set-defaults-correlation";
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            SET_DEFAULTS,
            2,
            None,
            Some(correlation),
            Some(test_structure([])),
        ),
        request_write_chunks: Vec::new(),
    });
    let result = client
        .execute(
            request(None).with_asynchronous_indicator(1),
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
    assert!(view.set_defaults().is_some());
    assert_eq!(view.result().status().raw(), 2);
}

#[test]
fn local_encoding_failure_is_not_sent_and_never_retried() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let limits = CodecLimits::new(0, 64, 100_000).expect("zero byte message limit is valid");
    let error = client
        .execute(request(None), &limits)
        .expect_err("unencodable local request never reaches the transport");
    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);
}
