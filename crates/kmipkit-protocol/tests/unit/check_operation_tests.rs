//! Check contracts derived from OASIS KMIP Specification v2.1 §6.1.7,
//! Tables 183–185, and the value encodings in §§4.17, 4.30, and 4.59.
//! Traceability: `KMIPKIT-REQ-SPEC-6.1.7-001`,
//! `-002-001`, `-002-002`, `-003`, `-004-001`, `-004-002`, and `-005`.
//! These are source-derived tests, not official OASIS Test Case executions.

use crate::async_operation_fixtures::{response_message, structure};
use crate::{
    CheckError, CheckRequest, CheckResponse, ResponseBatchItemView, ResponseMessage, ResultReason,
    UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Value, ValueView};

const CHECK_OPERATION: u32 = 0x0000_0009;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OBJECT_NOT_FOUND: u32 = 0x0000_0037;
const INCOMPATIBLE_CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0000_0029;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const LEASE_TIME: u32 = 0x0042_0049;
const USAGE_LIMITS_COUNT: u32 = 0x0042_0096;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one validated response batch item")
}

fn field_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
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

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw_tag)
            .expect("the source-derived tag fits TTLV width")
            .try_checked()
            .expect("the source-derived tag is allocated"),
        value,
    )
    .expect("the source-derived tag and value form a valid TTLV item")
}

fn success_payload() -> Structure {
    structure([
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string("checked-object".to_owned()),
        ),
        item(USAGE_LIMITS_COUNT, Value::long_integer(3)),
        item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(2)),
        item(LEASE_TIME, Value::interval(60)),
        item(UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ])
}

fn malformed_success(payload: Structure) -> Result<CheckResponse, CheckError> {
    check_response(SUCCESS, None, Some(payload))
}

fn check_response(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<CheckResponse, CheckError> {
    let message = response_message(CHECK_OPERATION, status, reason, None, payload);
    CheckResponse::try_from_response_item(response_item(&message))
}

#[test]
fn request_emits_table_183_fields_once_in_order_and_preserves_each_value() {
    let identifier = UniqueIdentifier::TextString("checked-object".to_owned());
    let request = CheckRequest::new()
        .with_unique_identifier(identifier.clone())
        .with_usage_limits_count(3)
        .with_cryptographic_usage_mask(2)
        .with_lease_time(60);

    assert_eq!(request.unique_identifier(), Some(&identifier));
    let payload = request
        .into_ttlv_payload()
        .expect("the optional Table 183 policy fields form a valid Check request");

    assert_eq!(
        field_tags(&payload),
        [
            UNIQUE_IDENTIFIER,
            USAGE_LIMITS_COUNT,
            CRYPTOGRAPHIC_USAGE_MASK,
            LEASE_TIME,
        ],
        "Table 183 fields are emitted in declared order and only once"
    );
    let fields = payload.view();
    for (tag, expected) in [
        (USAGE_LIMITS_COUNT, 3_i64),
        (CRYPTOGRAPHIC_USAGE_MASK, 2_i64),
        (LEASE_TIME, 60_i64),
    ] {
        let actual = fields
            .children()
            .iter()
            .find(|field| field.tag().raw() == tag)
            .and_then(|field| {
                field.with_value(|value| match value {
                    ValueView::LongInteger(value) => Some(*value),
                    ValueView::Integer(value) => Some(i64::from(*value)),
                    ValueView::Interval(value) => Some(i64::from(*value)),
                    _ => None,
                })
            });
        assert_eq!(actual, Some(expected));
    }
}

#[test]
fn request_can_omit_identifier_and_all_optional_policy_values() {
    let payload = CheckRequest::new()
        .into_ttlv_payload()
        .expect("Table 183 makes all Check request fields optional");

    assert!(payload.view().children().is_empty());
}

#[test]
fn successful_response_returns_identifier_and_present_policy_values_and_keeps_extensions() {
    let identifier = UniqueIdentifier::TextString("checked-object".to_owned());
    let response = check_response(SUCCESS, None, Some(success_payload()))
        .expect("Table 184 success contains its required Unique Identifier");

    assert_eq!(response.result().status().raw(), SUCCESS);
    assert_eq!(response.unique_identifier(), Some(&identifier));
    assert_eq!(response.usage_limits_count(), Some(3));
    assert_eq!(response.cryptographic_usage_mask(), Some(2));
    assert_eq!(response.lease_time(), Some(60));
    assert!(response.with_ttlv(|view| structure_contains_tag(&view, UNKNOWN_VENDOR_TAG)));
}

#[test]
fn successful_response_rejects_missing_or_duplicate_unique_identifier() {
    for payload in [
        Structure::new(),
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            item(UNIQUE_IDENTIFIER, Value::text_string("second".to_owned())),
        ]),
    ] {
        assert!(
            malformed_success(payload).is_err(),
            "Table 184 requires one well-formed Unique Identifier on success"
        );
    }
}

#[test]
fn failure_response_preserves_table_185_reason_without_success_identifier() {
    let response = check_response(OPERATION_FAILED, Some(OBJECT_NOT_FOUND), None)
        .expect("Table 185 Operation Failed remains a common KMIP result");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(OBJECT_NOT_FOUND)
    );
    assert!(response.unique_identifier().is_none());
    assert!(response.usage_limits_count().is_none());
    assert!(response.cryptographic_usage_mask().is_none());
    assert!(response.lease_time().is_none());
}

#[test]
fn policy_denial_response_returns_the_policy_value_without_a_success_identifier() {
    let response = check_response(
        OPERATION_FAILED,
        Some(INCOMPATIBLE_CRYPTOGRAPHIC_USAGE_MASK),
        Some(structure([
            item(USAGE_LIMITS_COUNT, Value::long_integer(3)),
            item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(2)),
            item(LEASE_TIME, Value::interval(60)),
        ])),
    )
    .expect("§6.1.7 returns policy attributes when Check denies use");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(INCOMPATIBLE_CRYPTOGRAPHIC_USAGE_MASK)
    );
    assert!(response.unique_identifier().is_none());
    assert_eq!(response.usage_limits_count(), Some(3));
    assert_eq!(response.cryptographic_usage_mask(), Some(2));
    assert_eq!(response.lease_time(), Some(60));
}

#[test]
fn request_debug_does_not_expose_identifier_value() {
    let sentinel = "check-request-debug-sentinel";
    let request = CheckRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(sentinel.to_owned()));

    assert!(!format!("{request:?}").contains(sentinel));
}
