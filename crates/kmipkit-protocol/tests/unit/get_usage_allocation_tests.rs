//! Project-derived checks for OASIS KMIP Specification v2.1 §6.1.23,
//! Tables 232–234, and §4.59, Table 148. These are not official test cases.
//! Traceability: KMIPKIT-025-T232-REQUEST, T233-RESPONSE, T234-ERROR.

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{
    GetUsageAllocationError, GetUsageAllocationRequest, GetUsageAllocationResponse,
    ResponseMessage, ResultReason, ResultValidationError, UniqueIdentifier,
};
use kmipkit_ttlv::{ItemType, Structure, Value, ValueView};
use std::error::Error as _;

const GET_USAGE_ALLOCATION: u32 = 0x0000_0011;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const USAGE_LIMITS_COUNT: u32 = 0x0042_0096;
const VENDOR_MEMBER: u32 = 0x0054_1234;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const OBJECT_ID: &str = "usage-object-23";

fn response_item(message: &ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("project fixture has one response item")
}

fn successful_payload() -> Structure {
    structure([
        item(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string(OBJECT_ID.to_owned()),
        ),
        item(VENDOR_MEMBER, Value::integer(17)),
    ])
}

#[test]
fn table_232_preserves_optional_identifier_and_required_long_integer_count() {
    let omitted_request = GetUsageAllocationRequest::new(0x0102_0304_0506_0708);
    assert_eq!(omitted_request.unique_identifier(), None);
    assert_eq!(omitted_request.usage_limits_count(), 0x0102_0304_0506_0708);
    assert_eq!(
        format!("{omitted_request:?}"),
        "GetUsageAllocationRequest { unique_identifier: None, usage_limits_count: 72623859790382856 }"
    );
    let omitted = omitted_request
        .to_ttlv_payload()
        .expect("Table 232 request is representable");
    let omitted_view = omitted.view();
    let fields = omitted_view.children();
    assert_eq!(fields.len(), 1, "omitted identifier remains absent");
    assert_eq!(fields[0].tag().raw(), USAGE_LIMITS_COUNT);
    assert_eq!(fields[0].item_type(), ItemType::LongInteger);
    assert!(
        fields[0]
            .with_value(|value| matches!(value, ValueView::LongInteger(0x0102_0304_0506_0708)))
    );

    let explicit = GetUsageAllocationRequest::new(9)
        .with_unique_identifier(UniqueIdentifier::TextString(OBJECT_ID.to_owned()));
    assert_eq!(
        explicit.unique_identifier(),
        Some(&UniqueIdentifier::TextString(OBJECT_ID.to_owned()))
    );
    assert_eq!(explicit.usage_limits_count(), 9);
    let explicit = explicit
        .to_ttlv_payload()
        .expect("optional Unique Identifier is representable");
    assert_eq!(
        explicit
            .view()
            .children()
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [UNIQUE_IDENTIFIER_TAG, USAGE_LIMITS_COUNT]
    );
    assert_eq!(
        explicit.view().children()[1].item_type(),
        ItemType::LongInteger
    );
}

#[test]
fn table_233_requires_resolved_identifier_and_retains_unknown_siblings() {
    let actual = {
        let message = response_message(
            GET_USAGE_ALLOCATION,
            0,
            None,
            None,
            Some(successful_payload()),
        );
        GetUsageAllocationResponse::try_from_response_item(response_item(&message))
            .expect("Table 233 success includes the resolved identifier")
    };
    assert_eq!(
        actual.unique_identifier(),
        Some(&UniqueIdentifier::TextString(OBJECT_ID.to_owned()))
    );
    assert_eq!(actual.result().status().raw(), 0);
    assert_eq!(actual.with_ttlv(|_| true), Some(true));
    assert_eq!(
        format!("{actual:?}"),
        "GetUsageAllocationResponse { result: KmipOperationResult { status: ResultStatus(0), reason: None, message: None }, has_unique_identifier: true, response_ttlv: \"[REDACTED]\" }"
    );
    actual.with_ttlv(|response| {
        let payload = response
            .children()
            .iter()
            .find(|field| field.tag().raw() == RESPONSE_PAYLOAD)
            .expect("complete response item retains its payload");
        payload.with_value(|value| {
            let ValueView::Structure(payload) = value else {
                panic!("Response Payload is a Structure");
            };
            assert_eq!(
                payload
                    .children()
                    .iter()
                    .map(|field| field.tag().raw())
                    .collect::<Vec<_>>(),
                [UNIQUE_IDENTIFIER_TAG, VENDOR_MEMBER]
            );
            assert!(
                payload.children()[1].with_value(|value| matches!(value, ValueView::Integer(17)))
            );
        });
    });
}

#[test]
fn get_usage_allocation_errors_format_messages_and_expose_only_wrapped_sources() {
    let result_error = ResultValidationError::FailureRequiresReason;
    let cases = [
        (
            GetUsageAllocationError::UnexpectedOperation,
            "response item is not Get Usage Allocation",
        ),
        (
            GetUsageAllocationError::MissingResultStatus,
            "Get Usage Allocation result status is missing",
        ),
        (
            GetUsageAllocationError::InvalidOperationResult(result_error),
            "invalid Get Usage Allocation result: Failure requires a Result Reason",
        ),
        (
            GetUsageAllocationError::MissingSuccessPayload,
            "successful Get Usage Allocation payload is missing",
        ),
        (
            GetUsageAllocationError::MalformedResponseItem,
            "Get Usage Allocation response item is malformed",
        ),
        (
            GetUsageAllocationError::MalformedSuccessPayload,
            "successful Get Usage Allocation payload is malformed",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
    }

    let invalid_result = GetUsageAllocationError::InvalidOperationResult(result_error);
    assert_eq!(
        invalid_result
            .source()
            .and_then(|source| source.downcast_ref::<ResultValidationError>()),
        Some(&result_error)
    );
    for error in [
        GetUsageAllocationError::UnexpectedOperation,
        GetUsageAllocationError::MissingResultStatus,
        GetUsageAllocationError::MissingSuccessPayload,
        GetUsageAllocationError::MalformedResponseItem,
        GetUsageAllocationError::MalformedSuccessPayload,
    ] {
        assert!(error.source().is_none());
    }
}

#[test]
fn table_233_rejects_missing_duplicate_and_wrong_typed_identifier() {
    for payload in [
        Some(Structure::new()),
        Some(structure([
            item(
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string(OBJECT_ID.to_owned()),
            ),
            item(
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("duplicate".to_owned()),
            ),
        ])),
        Some(structure([item(
            UNIQUE_IDENTIFIER_TAG,
            Value::boolean(true),
        )])),
    ] {
        let message = response_message(GET_USAGE_ALLOCATION, 0, None, None, payload);
        assert!(
            GetUsageAllocationResponse::try_from_response_item(response_item(&message)).is_err()
        );
    }
}

#[test]
fn table_233_preserves_integer_and_open_enumeration_identifiers() {
    for (wire_value, expected) in [
        (Value::integer(23), UniqueIdentifier::Integer(23)),
        (
            Value::enumeration(0x54FF),
            UniqueIdentifier::Enumeration(0x54FF),
        ),
    ] {
        let message = response_message(
            GET_USAGE_ALLOCATION,
            0,
            None,
            None,
            Some(structure([item(UNIQUE_IDENTIFIER_TAG, wire_value)])),
        );
        let actual = GetUsageAllocationResponse::try_from_response_item(response_item(&message))
            .expect("Table 145 Unique Identifier representations remain open");
        assert_eq!(actual.unique_identifier(), Some(&expected));
    }
}

#[test]
fn table_234_failures_and_unknown_reasons_remain_common_results() {
    // Table 234 repeats Invalid Message. Exercise every distinct listed reason.
    for reason in [
        0x21, 0x04, 0x30, 0x37, 0x1A, 0x15, 0x14, 0x08, 0x07, 0x05, 0x0C, 0x02, 0x54FF,
    ] {
        let message = response_message(GET_USAGE_ALLOCATION, 1, Some(reason), None, None);
        let actual = GetUsageAllocationResponse::try_from_response_item(response_item(&message))
            .expect("Failure remains a typed common result without a success payload");
        assert_eq!(actual.result().status().raw(), 1);
        assert_eq!(
            actual.result().reason(),
            Some(ResultReason::from_raw(reason))
        );
        assert!(actual.unique_identifier().is_none());
        assert!(actual.with_ttlv(|_| true).is_none());
    }
}

#[test]
fn get_usage_allocation_rejects_another_operation() {
    let message = response_message(0x0000_0001, 1, Some(0x07), None, None);
    let error = GetUsageAllocationResponse::try_from_response_item(response_item(&message))
        .expect_err("a different operation cannot be converted as Get Usage Allocation");
    assert_eq!(error, GetUsageAllocationError::UnexpectedOperation);
}

#[test]
fn pending_response_retains_exact_correlation_without_fabricating_identifier() {
    let correlation = [0xA5, 0x00, 0x5A, 0xFF];
    let message = response_message(
        GET_USAGE_ALLOCATION,
        2,
        None,
        Some(&correlation),
        Some(Structure::new()),
    );
    let actual = GetUsageAllocationResponse::try_from_response_item(response_item(&message))
        .expect("Pending uses the common operation result");
    assert_eq!(actual.result().status().raw(), 2);
    assert!(actual.unique_identifier().is_none());
    assert_eq!(
        response_item(&message).with_asynchronous_correlation_value(<[u8]>::to_vec),
        Some(correlation.to_vec()),
        "Pending correlation remains available through its dedicated field"
    );
    assert_eq!(actual.with_ttlv(|_| true), None);
}
