//! Project-derived fake-transport checks for OASIS KMIP Specification v2.1
//! §6.1.23, Tables 232–234, and §4.59, Table 148. These are not official cases.
//! Traceability: KMIPKIT-025-T232-REQUEST, T233-RESPONSE, T234-ERROR,
//! KMIPKIT-025-COMMON-RESULT, and KMIPKIT-025-COMMON-DELIVERY.

use kmipkit_protocol::{
    GetUsageAllocationRequest, GetUsageAllocationResponse, ResultReason, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{ItemType, Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::encode_message_for_test;
use crate::execute_test_support::{test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const GET_USAGE_ALLOCATION: u32 = 0x0000_0011;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const USAGE_LIMITS_COUNT: u32 = 0x0042_0096;
const VENDOR_MEMBER: u32 = 0x0054_1234;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const BATCH_ID: &[u8] = b"get-usage-allocation-batch";
const CORRELATION: &[u8] = &[0xA5, 0x00, 0x5A, 0xFF];
const OBJECT_ID: &str = "usage-object-23";
const DEBUG_SENTINEL: &str = "P025-DEBUG-SECRET-SENTINEL";

fn response_bytes(
    status: u32,
    reason: Option<u32>,
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
    let mut fields = vec![test_item(
        OPERATION,
        Value::enumeration(GET_USAGE_ALLOCATION),
    )];
    if let Some(id) = batch_id {
        fields.push(test_item(
            UNIQUE_BATCH_ITEM_ID,
            Value::byte_string(id.to_vec()),
        ));
    }
    fields.push(test_item(RESULT_STATUS, Value::enumeration(status)));
    if let Some(reason) = reason {
        fields.push(test_item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(correlation) = correlation {
        fields.push(test_item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(correlation.to_vec()),
        ));
    }
    if let Some(payload) = payload {
        fields.push(test_item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(test_structure(fields))),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("project fixture is valid KMIP TTLV")
}

fn success_payload() -> Structure {
    test_structure([
        test_item(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string(OBJECT_ID.to_owned()),
        ),
        test_item(VENDOR_MEMBER, Value::integer(17)),
    ])
}

fn request_batch(request: GetUsageAllocationRequest) -> ClientBatch {
    ClientBatch::new(ClientBatchItem::new(ClientRequest::get_usage_allocation(
        request,
    )))
}

fn request_with_identifier(count: i64) -> GetUsageAllocationRequest {
    GetUsageAllocationRequest::new(count)
        .with_unique_identifier(UniqueIdentifier::TextString(OBJECT_ID.to_owned()))
}

fn completed(outcome: &ClientBatchOutcome) -> &GetUsageAllocationResponse {
    assert_eq!(outcome.operation(), ClientOperation::GetUsageAllocation);
    let ClientBatchOutcome::GetUsageAllocation(response) = outcome else {
        panic!("Get Usage Allocation returns its typed result");
    };
    response
}

fn assert_request(wire: &[u8], expected_identifier: Option<&str>, expected_count: i64) {
    let decoded = decode(wire).expect("outbound request is valid TTLV");
    decoded.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("request message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .find(|field| field.tag().raw() == BATCH_ITEM)
            .expect("one Batch Item is present");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("Batch Item is a Structure");
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)
                .expect("operation is present");
            assert!(operation.with_value(|value| matches!(
                value,
                ValueView::Enumeration(actual) if *actual == GET_USAGE_ALLOCATION
            )));
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("Table 232 has a Request Payload");
            assert_eq!(payload.item_type(), ItemType::Structure);
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                let fields = payload.children();
                let expected_count_position = usize::from(expected_identifier.is_some());
                assert_eq!(fields.len(), expected_count_position + 1);
                if let Some(identifier) = expected_identifier {
                    assert_eq!(fields[0].tag().raw(), UNIQUE_IDENTIFIER_TAG);
                    assert!(fields[0].with_value(|value| matches!(
                        value,
                        ValueView::TextString(actual) if actual == identifier
                    )));
                }
                let count = &fields[expected_count_position];
                assert_eq!(count.tag().raw(), USAGE_LIMITS_COUNT);
                assert_eq!(count.item_type(), ItemType::LongInteger);
                assert!(count.with_value(|value| matches!(
                    value,
                    ValueView::LongInteger(actual) if *actual == expected_count
                )));
            });
        });
    });
}

#[test]
fn table_232_one_exchange_encodes_identifier_and_long_integer_count() {
    for (identifier, request, expected_count) in [
        (
            Some(OBJECT_ID),
            request_with_identifier(0x0102_0304_0506_0708),
            0x0102_0304_0506_0708,
        ),
        (
            Some(OBJECT_ID),
            GetUsageAllocationRequest::new(29)
                .with_unique_identifier(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
            29,
        ),
    ] {
        let (mut client, fake, captured) = client_for(ExchangeScript::Success {
            response: response_bytes(0, None, None, None, Some(success_payload())),
            request_write_chunks: Vec::new(),
        });
        let result = client
            .execute(request_batch(request), &CodecLimits::defaults())
            .expect("Table 233 success is returned as a typed response");
        assert_eq!(fake.borrow().exchange_count(), 1);
        assert_request(
            captured
                .borrow()
                .as_ref()
                .expect("request was captured")
                .as_slice(),
            identifier,
            expected_count,
        );
        assert_eq!(
            completed(result.get(0).expect("one result").outcome()).unique_identifier(),
            Some(&UniqueIdentifier::TextString(OBJECT_ID.to_owned()))
        );
    }
}

#[test]
fn table_234_failures_and_unknown_reasons_remain_typed_results_without_retry() {
    for reason in [
        0x21, 0x04, 0x30, 0x37, 0x1A, 0x15, 0x14, 0x08, 0x07, 0x05, 0x0C, 0x02, 0x54FF,
    ] {
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response: response_bytes(1, Some(reason), None, None, None),
            request_write_chunks: Vec::new(),
        });
        let result = client
            .execute(
                request_batch(request_with_identifier(1)),
                &CodecLimits::defaults(),
            )
            .expect("Table 234 Failure remains a typed result");
        let outcome = result.get(0).expect("one response").outcome();
        let response = completed(outcome);
        assert_eq!(fake.borrow().exchange_count(), 1, "reason {reason:#x}");
        assert_eq!(response.result().status().raw(), 1);
        assert_eq!(
            response.result().reason(),
            Some(ResultReason::from_raw(reason))
        );
        assert!(response.unique_identifier().is_none());
    }
}

#[test]
fn malformed_success_missing_identifier_is_response_started_without_retry() {
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: response_bytes(0, None, None, None, Some(Structure::new())),
        request_write_chunks: Vec::new(),
    });
    let error = client
        .execute(
            request_batch(request_with_identifier(1)),
            &CodecLimits::defaults(),
        )
        .expect_err("Table 233 success requires Unique Identifier");
    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn pending_keeps_exact_correlation_and_does_not_poll() {
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: response_bytes(
            2,
            None,
            Some(CORRELATION),
            Some(BATCH_ID),
            Some(Structure::new()),
        ),
        request_write_chunks: Vec::new(),
    });
    let result = client
        .execute(
            ClientBatch::new(
                ClientBatchItem::new(ClientRequest::get_usage_allocation(
                    request_with_identifier(1),
                ))
                .with_unique_batch_item_id(BATCH_ID.to_vec()),
            )
            .with_asynchronous_indicator(1),
            &CodecLimits::defaults(),
        )
        .expect("Pending remains explicit");
    assert_eq!(fake.borrow().exchange_count(), 1);
    let outcome = result.get(0).expect("one response").outcome();
    assert_eq!(outcome.operation(), ClientOperation::GetUsageAllocation);
    assert_eq!(outcome.asynchronous_correlation_value(), Some(CORRELATION));
    assert!(matches!(outcome, ClientBatchOutcome::Pending(_)));
    assert!(!format!("{outcome:?}").contains("A5005AFF"));
    let view = outcome.response();
    assert!(view.get_usage_allocation().is_some());
    assert_eq!(view.result().status().raw(), 2);
    assert!(format!("{view:?}").starts_with("GetUsageAllocation("));
}

#[test]
fn get_usage_allocation_outcome_exposes_typed_views_and_safe_diagnostics() {
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: response_bytes(
            0,
            None,
            None,
            None,
            Some(test_structure([test_item(
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string(DEBUG_SENTINEL.to_owned()),
            )])),
        ),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .execute(
            request_batch(request_with_identifier(1)),
            &CodecLimits::defaults(),
        )
        .expect("Get Usage Allocation succeeds");
    let outcome = response.get(0).expect("one response").outcome();
    let view = outcome.response();

    assert_eq!(outcome.operation(), ClientOperation::GetUsageAllocation);
    assert_eq!(outcome.result().status().raw(), 0);
    assert_eq!(view.result().status().raw(), 0);
    assert!(view.get_constraints().is_none());
    assert!(view.set_constraints().is_none());
    assert!(view.set_defaults().is_none());
    assert!(view.get_usage_allocation().is_some());
    let outcome_debug = format!("{outcome:?}");
    let view_debug = format!("{view:?}");
    assert!(outcome_debug.starts_with("GetUsageAllocation("));
    assert!(view_debug.starts_with("GetUsageAllocation("));
    for formatted in [&outcome_debug, &view_debug] {
        assert!(formatted.contains("[REDACTED]"), "{formatted}");
        assert!(!formatted.contains(DEBUG_SENTINEL), "{formatted}");
    }
    assert_eq!(
        outcome.to_string(),
        format!("GetUsageAllocation({})", outcome.result())
    );
}

#[test]
fn local_limit_is_not_sent_and_partial_write_is_possibly_sent_without_retry() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 1 });
    let impossible = CodecLimits::new(0, 64, 100_000).expect("zero byte limit is supported");
    let error = client
        .execute(request_batch(request_with_identifier(1)), &impossible)
        .expect_err("request cannot fit in a zero byte limit");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);

    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = client
        .execute(
            request_batch(request_with_identifier(1)),
            &CodecLimits::defaults(),
        )
        .expect_err("partial write keeps ambiguous delivery");
    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
