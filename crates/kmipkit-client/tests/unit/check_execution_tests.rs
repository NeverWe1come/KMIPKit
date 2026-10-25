#![cfg(test)]

//! Fake-transport Check contracts from OASIS KMIP v2.1 §6.1.7,
//! Tables 183–185, and the value encodings in §§4.17, 4.30, and 4.59.
//! Traceability: `KMIPKIT-REQ-SPEC-6.1.7-001`,
//! `-002-001`, `-002-002`, `-003`, `-004-001`, `-004-002`, and `-005`.
//! Shared Pending and delivery behavior follows KMIP v2.1 §§8.6, 9.1, and
//! 9.2. These are derived tests, not official OASIS Test Case executions.

use kmipkit_protocol::{
    AttributeSet, CheckRequest, CreateRequest, LocateRequest, ObjectType, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Structure, StructureView, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{
    asynchronous_response_bytes, operation_batch_response_bytes, test_item, test_structure,
};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const CHECK_OPERATION: u32 = 0x0000_0009;
const LOCATE_OPERATION: u32 = 0x0000_0008;
const CREATE_OPERATION: u32 = 0x0000_0001;
const BATCH_ITEM: u32 = 0x0042_000F;
const REQUEST_HEADER: u32 = 0x0042_0077;
const BATCH_ORDER_OPTION: u32 = 0x0042_0010;
const BATCH_ERROR_CONTINUATION_OPTION: u32 = 0x0042_000E;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const OBJECT_TYPE: u32 = 0x0042_0057;
const USAGE_LIMITS_COUNT: u32 = 0x0042_0096;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const LEASE_TIME: u32 = 0x0042_0049;
const ATTRIBUTES: u32 = 0x0042_0125;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const OBJECT_NOT_FOUND: u32 = 0x0000_0037;
const INCOMPATIBLE_CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0000_0029;
const PENDING_CORRELATION: &[u8] = b"CHECK_PENDING\x00CORRELATION\xFF";

fn check_request(identifier: &str) -> CheckRequest {
    CheckRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned()))
        .with_usage_limits_count(3)
        .with_cryptographic_usage_mask(2)
        .with_lease_time(60)
}

fn response_payload(identifier: &str) -> Structure {
    test_structure([
        test_item(UNIQUE_IDENTIFIER, Value::text_string(identifier.to_owned())),
        test_item(USAGE_LIMITS_COUNT, Value::long_integer(3)),
        test_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(2)),
        test_item(LEASE_TIME, Value::interval(60)),
        test_item(UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ])
}

fn check_response_bytes(status: u32, reason: Option<u32>, payload: Option<Structure>) -> Vec<u8> {
    asynchronous_response_bytes(CHECK_OPERATION, status, reason, None, payload)
}

fn structure_contains_tag(view: &StructureView<'_>, raw_tag: u32) -> bool {
    view.children().iter().any(|field| {
        field.tag().raw() == raw_tag
            || field.with_value(|value| match value {
                ValueView::Structure(nested) => structure_contains_tag(&nested, raw_tag),
                _ => false,
            })
    })
}

fn integer_field(view: &StructureView<'_>, raw_tag: u32) -> Option<i32> {
    view.children().iter().find_map(|field| {
        if field.tag().raw() == raw_tag {
            field.with_value(|value| match value {
                ValueView::Integer(value) => Some(*value),
                _ => None,
            })
        } else {
            field.with_value(|value| match value {
                ValueView::Structure(nested) => integer_field(&nested, raw_tag),
                _ => None,
            })
        }
    })
}

fn request_fields(bytes: &[u8]) -> (Option<u32>, Vec<(u32, Option<i64>)>) {
    let message = decode(bytes).expect("the captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("the KMIP request message is a Structure");
        };
        let batch_item = message
            .children()
            .iter()
            .find(|field| field.tag().raw() == BATCH_ITEM)
            .expect("the Check request contains a Batch Item");
        batch_item.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("the Batch Item is a Structure");
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)
                .and_then(|field| {
                    field.with_value(|value| match value {
                        ValueView::Enumeration(value) => Some(*value),
                        _ => None,
                    })
                });
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("the Check Batch Item contains a Request Payload");
            let fields = payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("the Request Payload is a Structure");
                };
                payload
                    .children()
                    .iter()
                    .map(|field| {
                        let integer = field.with_value(|value| match value {
                            ValueView::LongInteger(value) => Some(*value),
                            ValueView::Integer(value) => Some(i64::from(*value)),
                            ValueView::Interval(value) => Some(i64::from(*value)),
                            _ => None,
                        });
                        (field.tag().raw(), integer)
                    })
                    .collect()
            });
            (operation, fields)
        })
    })
}

fn request_payload_tags(bytes: &[u8], batch_index: usize) -> Vec<u32> {
    let message = decode(bytes).expect("the captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("the KMIP request message is a Structure");
        };
        let batch_items = message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM)
            .collect::<Vec<_>>();
        batch_items[batch_index].with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("the Batch Item is a Structure");
            };
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("the Batch Item contains a Request Payload");
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("the Request Payload is a Structure");
                };
                payload
                    .children()
                    .iter()
                    .map(|field| field.tag().raw())
                    .collect()
            })
        })
    })
}

fn request_usage_mask(bytes: &[u8], batch_index: usize) -> (Option<u32>, Option<i32>) {
    let message = decode(bytes).expect("the captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("the KMIP request message is a Structure");
        };
        let batch_item = message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM)
            .nth(batch_index)
            .expect("the captured request contains the indexed Batch Item");
        batch_item.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("the Batch Item is a Structure");
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)
                .and_then(|field| {
                    field.with_value(|value| match value {
                        ValueView::Enumeration(value) => Some(*value),
                        _ => None,
                    })
                });
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("the Batch Item contains a Request Payload");
            let mask = payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("the Request Payload is a Structure");
                };
                if operation == Some(LOCATE_OPERATION) {
                    let attributes = payload
                        .children()
                        .iter()
                        .find(|field| field.tag().raw() == ATTRIBUTES)
                        .expect("the Locate payload contains Attributes");
                    attributes.with_value(|value| {
                        let ValueView::Structure(attributes) = value else {
                            panic!("Locate Attributes is a Structure");
                        };
                        integer_field(&attributes, CRYPTOGRAPHIC_USAGE_MASK)
                    })
                } else {
                    integer_field(&payload, CRYPTOGRAPHIC_USAGE_MASK)
                }
            });
            (operation, mask)
        })
    })
}

fn request_header_values(bytes: &[u8]) -> (Option<bool>, Option<u32>) {
    let message = decode(bytes).expect("the captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("the KMIP request message is a Structure");
        };
        let header = message
            .children()
            .iter()
            .find(|field| field.tag().raw() == REQUEST_HEADER)
            .expect("the request contains a Request Header");
        header.with_value(|value| {
            let ValueView::Structure(header) = value else {
                panic!("the Request Header is a Structure");
            };
            let batch_order = header
                .children()
                .iter()
                .find(|field| field.tag().raw() == BATCH_ORDER_OPTION)
                .and_then(|field| {
                    field.with_value(|value| match value {
                        ValueView::Boolean(value) => Some(*value),
                        _ => None,
                    })
                });
            let continuation = header
                .children()
                .iter()
                .find(|field| field.tag().raw() == BATCH_ERROR_CONTINUATION_OPTION)
                .and_then(|field| {
                    field.with_value(|value| match value {
                        ValueView::Enumeration(value) => Some(*value),
                        _ => None,
                    })
                });
            (batch_order, continuation)
        })
    })
}

fn two_check_response_bytes() -> Vec<u8> {
    operation_batch_response_bytes([
        (
            CHECK_OPERATION,
            b"check-first".to_vec(),
            response_payload("checked-first"),
        ),
        (
            CHECK_OPERATION,
            b"check-second".to_vec(),
            response_payload("checked-second"),
        ),
    ])
}

#[test]
fn check_sends_table_183_fields_and_exposes_table_184_response_once() {
    crate::execute_test_support::assert_lifecycle_request_debug(
        &crate::ClientRequest::check(check_request("debug-check")),
        "Check",
    );
    let response = check_response_bytes(SUCCESS, None, Some(response_payload("checked-object")));
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![4, 7],
    });

    let item = client
        .check(check_request("checked-object"), &CodecLimits::defaults())
        .expect("the Table 184 Check success response is valid");
    let outcome = item.outcome();
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        outcome,
        ClientOperation::Check,
        "Check(",
        "Check(",
    );
    let typed = outcome
        .response()
        .check()
        .expect("the result retains its typed Check response");

    assert_eq!(outcome.operation(), ClientOperation::Check);
    assert_eq!(outcome.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.unique_identifier(),
        Some(&UniqueIdentifier::TextString("checked-object".to_owned()))
    );
    assert_eq!(typed.usage_limits_count(), Some(3));
    assert_eq!(typed.cryptographic_usage_mask(), Some(2));
    assert_eq!(typed.lease_time(), Some(60));
    assert!(typed.with_ttlv(|view| structure_contains_tag(&view, UNKNOWN_VENDOR_TAG)));
    let (operation, fields) = request_fields(
        captured
            .borrow()
            .as_deref()
            .expect("the fake transport captured the Check request"),
    );
    assert_eq!(operation, Some(CHECK_OPERATION));
    assert_eq!(
        fields,
        [
            (UNIQUE_IDENTIFIER, None),
            (USAGE_LIMITS_COUNT, Some(3)),
            (CRYPTOGRAPHIC_USAGE_MASK, Some(2)),
            (LEASE_TIME, Some(60)),
        ]
    );
    assert_eq!(fake.borrow().exchange_count(), 1, "Check is never retried");
}

#[test]
fn locate_and_check_requests_preserve_distinct_cryptographic_usage_masks() {
    // OASIS KMIP v2.1 §6.1.7/Table 182 permits Check's mask to narrow a preceding
    // Locate mask. §12.1/Table 495 defines Encrypt as 0x04 and Decrypt as 0x08.
    // It asserts only the two client request payloads; the fake response makes
    // no server claim.
    let locate_attributes =
        AttributeSet::try_new([test_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(0x0C))])
            .expect("the Locate mask is a valid direct Integer criterion");
    let response = operation_batch_response_bytes([
        (
            LOCATE_OPERATION,
            b"locate-usage-mask".to_vec(),
            test_structure([test_item(
                UNIQUE_IDENTIFIER,
                Value::text_string("located-object".to_owned()),
            )]),
        ),
        (
            CHECK_OPERATION,
            b"check-usage-mask".to_vec(),
            response_payload("checked-object"),
        ),
    ]);
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Locate(LocateRequest::new(locate_attributes)))
            .with_unique_batch_item_id(b"locate-usage-mask".to_vec()),
        ClientBatchItem::new(ClientRequest::Check(
            check_request("checked-object").with_cryptographic_usage_mask(0x04),
        ))
        .with_unique_batch_item_id(b"check-usage-mask".to_vec()),
    ]);

    client
        .execute(batch, &CodecLimits::defaults())
        .expect("the typed Locate/Check request batch is serializable");

    let captured = captured.borrow();
    let request = captured
        .as_deref()
        .expect("the fake transport captured the submitted batch");
    assert_eq!(
        request_usage_mask(request, 0),
        (Some(LOCATE_OPERATION), Some(0x0C))
    );
    assert_eq!(
        request_usage_mask(request, 1),
        (Some(CHECK_OPERATION), Some(0x04))
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn check_failure_preserves_table_185_result_without_retry() {
    let response = check_response_bytes(OPERATION_FAILED, Some(OBJECT_NOT_FOUND), None);
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let item = client
        .check(check_request("missing-object"), &CodecLimits::defaults())
        .expect("a valid Table 185 Operation Failed response remains a KMIP result");
    assert_eq!(item.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        item.outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(OBJECT_NOT_FOUND)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn check_policy_denial_exposes_the_returned_mask_without_an_identifier() {
    let response = check_response_bytes(
        OPERATION_FAILED,
        Some(INCOMPATIBLE_CRYPTOGRAPHIC_USAGE_MASK),
        Some(test_structure([
            test_item(USAGE_LIMITS_COUNT, Value::long_integer(3)),
            test_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(2)),
            test_item(LEASE_TIME, Value::interval(60)),
        ])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let item = client
        .check(check_request("checked-object"), &CodecLimits::defaults())
        .expect("Check preserves the §6.1.7 policy values in a denial response");
    let check = item
        .outcome()
        .response()
        .check()
        .expect("the failed result retains its typed Check response");

    assert!(check.unique_identifier().is_none());
    assert_eq!(check.usage_limits_count(), Some(3));
    assert_eq!(check.cryptographic_usage_mask(), Some(2));
    assert_eq!(check.lease_time(), Some(60));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn check_pending_preserves_correlation_without_poll_or_retry() {
    let response = asynchronous_response_bytes(
        CHECK_OPERATION,
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Check(check_request(
        "checked-object",
    ))))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("caller-enabled typed batch preserves Check Pending");
    let item = response.get(0).expect("one Check result is associated");
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        item.outcome(),
        ClientOperation::Check,
        "Check(",
        "Pending(",
    );
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("the server's Operation Pending result remains Pending");
    };

    assert_eq!(pending.operation(), ClientOperation::Check);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1, "no implicit Poll occurs");
}

#[test]
fn check_leaves_recommended_batch_options_explicitly_selected_by_the_caller() {
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response: two_check_response_bytes(),
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Check(check_request("first")))
            .with_unique_batch_item_id(b"check-first".to_vec()),
        ClientBatchItem::new(ClientRequest::Check(check_request("second")))
            .with_unique_batch_item_id(b"check-second".to_vec()),
    ])
    .with_batch_order_option(true)
    .with_batch_error_continuation_option(2);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the caller's ordered Stop batch of Checks is valid");
    assert!(response.get(0).is_some());
    assert!(response.get(1).is_some());
    let captured_request = captured.borrow();
    let request = captured_request
        .as_deref()
        .expect("the caller's Check batch reaches the fake transport");
    assert_eq!(request_header_values(request), (Some(true), Some(2)));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn check_can_consume_a_preceding_successful_create_placeholder() {
    let response = operation_batch_response_bytes([
        (
            CREATE_OPERATION,
            b"create-item".to_vec(),
            test_structure([
                test_item(OBJECT_TYPE, Value::enumeration(7)),
                test_item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string("created-object".to_owned()),
                ),
            ]),
        ),
        (
            CHECK_OPERATION,
            b"check-item".to_vec(),
            response_payload("created-object"),
        ),
    ]);
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Create(CreateRequest::new(
            ObjectType::from_raw(7),
            AttributeSet::new(),
        )))
        .with_unique_batch_item_id(b"create-item".to_vec()),
        ClientBatchItem::new(ClientRequest::Check(CheckRequest::new()))
            .with_unique_batch_item_id(b"check-item".to_vec()),
    ])
    .with_batch_order_option(true)
    .with_batch_error_continuation_option(1);

    client
        .execute(batch, &CodecLimits::defaults())
        .expect("Check can use the preceding Create response ID Placeholder");

    let captured_request = captured.borrow();
    let request = captured_request
        .as_deref()
        .expect("the ordered Create and Check batch reaches the fake transport");
    assert_eq!(request_header_values(request), (Some(true), Some(2)));
    assert!(!request_payload_tags(request, 1).contains(&UNIQUE_IDENTIFIER));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn check_transport_error_preserves_delivery_state_after_one_exchange() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });

    let error = client
        .check(check_request("checked-object"), &CodecLimits::defaults())
        .expect_err("the fake transport deliberately fails after part of the request is written");

    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1, "Check is never retried");
}
