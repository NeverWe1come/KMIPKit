//! Re-key contracts from OASIS KMIP Specification v2.1 §6.1.46, Tables
//! 303–307. Traceability: `KMIPKIT-REQ-SPEC-6.1.46-001`, `-004-001`, and
//! `-004-002`. Catalog cases CN01-2-76 through -87 have no pinned fixtures;
//! these are source-derived tests, not official OASIS Test Case executions.

use crate::async_operation_fixtures::{response_message, structure};
use crate::{
    RekeyError, RekeyRequest, RekeyResponse, ResponseBatchItemView, ResponseMessage, ResultReason,
    UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Value, ValueView};

const REKEY_OPERATION: u32 = 0x0000_0004;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const KEY_VALUE_NOT_PRESENT: u32 = 0x0000_0013;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const OFFSET: u32 = 0x0042_0058;
const ATTRIBUTES: u32 = 0x0042_0125;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
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

fn identifier_item(value: &str) -> Item {
    item(UNIQUE_IDENTIFIER, Value::text_string(value.to_owned()))
}

fn typed_response(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<RekeyResponse, RekeyError> {
    let message = response_message(REKEY_OPERATION, status, reason, None, payload);
    RekeyResponse::try_from_response_item(response_item(&message))
}

#[test]
fn request_emits_table_305_fields_once_in_order_and_preserves_values() {
    let identifier = UniqueIdentifier::TextString("existing-key".to_owned());
    let attributes = structure([item(0x0042_001D, Value::enumeration(1))]);
    let masks = structure([item(0x0042_0160, Value::integer(3))]);
    let request = RekeyRequest::new()
        .with_unique_identifier(identifier.clone())
        .with_offset(86_400)
        .with_attributes(attributes)
        .with_protection_storage_masks(masks);

    assert_eq!(request.unique_identifier(), Some(&identifier));
    let payload = request
        .into_ttlv_payload()
        .expect("the complete Table 305 request form is valid");
    assert_eq!(
        field_tags(&payload),
        [
            UNIQUE_IDENTIFIER,
            OFFSET,
            ATTRIBUTES,
            PROTECTION_STORAGE_MASKS
        ]
    );
    assert_eq!(
        payload.view().children()[1].with_value(|value| match value {
            ValueView::Interval(value) => Some(*value),
            _ => None,
        }),
        Some(86_400)
    );
}

#[test]
fn request_can_omit_identifier_to_use_the_batch_id_placeholder() {
    let payload = RekeyRequest::new()
        .into_ttlv_payload()
        .expect("Table 305 permits omitted UID for the server ID Placeholder");

    assert!(payload.view().children().is_empty());
}

#[test]
fn request_rejects_repeated_singleton_and_dsa_when_object_context_is_not_pgp() {
    // KMIP Specification v2.1 §§4, 4.22 Table 72, and §6.1.46 Table 305.
    let repeated_comment = structure([
        item(0x0042_00FD, Value::text_string("first".to_owned())),
        item(0x0042_00FD, Value::text_string("second".to_owned())),
    ]);
    assert!(
        RekeyRequest::new()
            .with_attributes(repeated_comment)
            .into_ttlv_payload()
            .is_err()
    );

    let repeated_dsa = structure([
        item(0x0042_00AE, Value::enumeration(1)),
        item(0x0042_00AE, Value::enumeration(2)),
    ]);
    assert!(
        RekeyRequest::new()
            .with_attributes(repeated_dsa)
            .into_ttlv_payload()
            .is_err()
    );
}

#[test]
fn request_rejects_repeated_dsa_for_symmetric_key_context() {
    // KMIP Specification v2.1 §§4, 4.22 Table 72, and §6.1.46 Table 305.
    let attributes = structure([
        item(0x0042_00AE, Value::enumeration(1)),
        item(0x0042_00AE, Value::enumeration(2)),
    ]);

    assert!(
        RekeyRequest::new()
            .with_attributes(attributes)
            .into_ttlv_payload()
            .is_err()
    );
}

#[test]
fn successful_response_returns_replacement_identifier_and_preserves_unknown_fields() {
    let payload = structure([
        identifier_item("replacement-key"),
        item(UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ]);
    let response = typed_response(SUCCESS, None, Some(payload))
        .expect("Table 306 success includes the replacement key identifier");

    assert_eq!(response.result().status().raw(), SUCCESS);
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString("replacement-key".to_owned()))
    );
    assert!(response.with_ttlv(|view| structure_contains_tag(&view, UNKNOWN_VENDOR_TAG)));
}

#[test]
fn successful_response_rejects_missing_or_duplicate_unique_identifier() {
    for payload in [
        Structure::new(),
        structure([identifier_item("first"), identifier_item("second")]),
    ] {
        assert!(
            typed_response(SUCCESS, None, Some(payload)).is_err(),
            "Table 306 success requires exactly one Unique Identifier"
        );
    }
}

#[test]
fn failure_preserves_table_307_reason_without_success_identifier() {
    let response = typed_response(OPERATION_FAILED, Some(KEY_VALUE_NOT_PRESENT), None)
        .expect("Table 307 Operation Failed remains a common KMIP result");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(KEY_VALUE_NOT_PRESENT)
    );
    assert!(response.unique_identifier().is_none());
}

#[test]
fn request_debug_does_not_expose_mask_structure_values() {
    let sentinel = b"REKEY_MASK_SECRET_SENTINEL";
    let masks = structure([item(
        0x0042_0160,
        Value::text_string(String::from_utf8_lossy(sentinel).into_owned()),
    )]);
    let request = RekeyRequest::new().with_protection_storage_masks(masks);

    assert!(!format!("{request:?}").contains("REKEY_MASK_SECRET_SENTINEL"));
}
