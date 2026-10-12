//! Obtain Lease contracts derived from OASIS KMIP Specification v2.1 §6.1.35,
//! Tables 268–270, and §4.29/§4.30 for Last Change Date and Lease Time.
//! Traceability: `KMIPKIT-REQ-SPEC-6.1.35-001-001`, `-001-002`, `-002`, and
//! `-005`. These are derived tests, not official OASIS Test Case executions.

use crate::async_operation_fixtures::{response_message, structure};
use crate::{
    ObtainLeaseError, ObtainLeaseRequest, ObtainLeaseResponse, ResponseBatchItemView,
    ResponseMessage, ResultReason, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Value, ValueView};

const OBTAIN_LEASE_OPERATION: u32 = 0x0000_0010;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OBJECT_NOT_FOUND: u32 = 0x0000_0037;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const LEASE_TIME: u32 = 0x0042_0049;
const LAST_CHANGE_DATE: u32 = 0x0042_0048;
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
            Value::text_string("leased-object".to_owned()),
        ),
        item(LEASE_TIME, Value::interval(0)),
        item(LAST_CHANGE_DATE, Value::date_time(1_700_000_000)),
        item(UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ])
}

fn typed_response(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<ObtainLeaseResponse, ObtainLeaseError> {
    let message = response_message(OBTAIN_LEASE_OPERATION, status, reason, None, payload);
    ObtainLeaseResponse::try_from_response_item(response_item(&message))
}

#[test]
fn request_emits_optional_unique_identifier_once_in_table_268_order() {
    let identifier = UniqueIdentifier::TextString("leased-object".to_owned());
    let payload = ObtainLeaseRequest::new()
        .with_unique_identifier(identifier)
        .into_ttlv_payload()
        .expect("the optional Table 268 identifier forms a valid payload");

    assert_eq!(field_tags(&payload), [UNIQUE_IDENTIFIER]);
}

#[test]
fn request_can_omit_unique_identifier_for_an_id_placeholder_batch() {
    let payload = ObtainLeaseRequest::new()
        .into_ttlv_payload()
        .expect("Table 268 makes Unique Identifier optional");

    assert!(payload.view().children().is_empty());
}

#[test]
fn successful_response_preserves_identifier_zero_lease_time_date_and_extensions() {
    let response = typed_response(SUCCESS, None, Some(success_payload()))
        .expect("Table 269 success includes its three required fields");

    assert_eq!(response.result().status().raw(), SUCCESS);
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString("leased-object".to_owned()))
    );
    assert_eq!(response.lease_time(), Some(0));
    assert_eq!(response.last_change_date(), Some(1_700_000_000));
    assert!(response.with_ttlv(|view| structure_contains_tag(&view, UNKNOWN_VENDOR_TAG)));
}

#[test]
fn successful_response_exposes_positive_lease_interval() {
    let payload = structure([
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string("leased-object".to_owned()),
        ),
        item(LEASE_TIME, Value::interval(30)),
        item(LAST_CHANGE_DATE, Value::date_time(1_700_000_000)),
    ]);
    let response = typed_response(SUCCESS, None, Some(payload))
        .expect("Table 269 success exposes the required positive Lease Time");

    assert_eq!(response.lease_time(), Some(30));
}

#[test]
fn successful_response_rejects_missing_duplicate_or_mistyped_required_fields() {
    let payloads = [
        structure([
            item(LEASE_TIME, Value::interval(1)),
            item(LAST_CHANGE_DATE, Value::date_time(2)),
        ]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("id".to_owned())),
            item(LAST_CHANGE_DATE, Value::date_time(2)),
        ]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("id".to_owned())),
            item(LEASE_TIME, Value::interval(1)),
            item(LAST_CHANGE_DATE, Value::integer(2)),
        ]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            item(UNIQUE_IDENTIFIER, Value::text_string("second".to_owned())),
            item(LEASE_TIME, Value::interval(1)),
            item(LAST_CHANGE_DATE, Value::date_time(2)),
        ]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("id".to_owned())),
            item(LEASE_TIME, Value::interval(1)),
            item(LAST_CHANGE_DATE, Value::date_time(2)),
            item(LAST_CHANGE_DATE, Value::date_time(3)),
        ]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("id".to_owned())),
            item(LEASE_TIME, Value::interval(1)),
            item(LEASE_TIME, Value::interval(2)),
            item(LAST_CHANGE_DATE, Value::date_time(3)),
        ]),
        structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::byte_string(b"not-an-identifier".to_vec()),
            ),
            item(LEASE_TIME, Value::interval(1)),
            item(LAST_CHANGE_DATE, Value::date_time(2)),
        ]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("id".to_owned())),
            item(LEASE_TIME, Value::integer(1)),
            item(LAST_CHANGE_DATE, Value::date_time(2)),
        ]),
    ];
    for payload in payloads {
        assert!(
            typed_response(SUCCESS, None, Some(payload)).is_err(),
            "Table 269 success requires one correctly typed instance of each field"
        );
    }
}

#[test]
fn failure_preserves_table_270_reason_without_typed_success_values() {
    let response = typed_response(OPERATION_FAILED, Some(OBJECT_NOT_FOUND), None)
        .expect("Table 270 Operation Failed remains a common KMIP result");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(OBJECT_NOT_FOUND)
    );
    assert!(response.unique_identifier().is_none());
    assert!(response.lease_time().is_none());
    assert!(response.last_change_date().is_none());
}

#[test]
fn request_debug_does_not_expose_identifier_value() {
    let sentinel = "obtain-lease-request-debug-sentinel";
    let request = ObtainLeaseRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(sentinel.to_owned()));

    assert!(!format!("{request:?}").contains(sentinel));
}
