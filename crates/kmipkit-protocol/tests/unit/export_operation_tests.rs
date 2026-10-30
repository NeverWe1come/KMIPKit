//! Export payload and result contracts from OASIS KMIP v2.1 §6.1.18, Tables 217–219.
//! The local vectors derive from pinned source tables and are not official Test Case executions.

use crate::{
    ExportError, ExportRequest, ExportResponse, KeyCompressionType, KeyFormatType, KeyWrapType,
    ObjectType, ResponseBatchItemView, ResponseMessage, ResultReason, ResultValidationError,
    UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};
use std::error::Error as _;

const EXPORT_OPERATION: u32 = 0x0000_002B;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_WRAP_TYPE: u32 = 0x0042_00F8;
const KEY_COMPRESSION_TYPE: u32 = 0x0042_0041;
const KEY_WRAPPING_SPECIFICATION: u32 = 0x0042_0047;
const ATTRIBUTES: u32 = 0x0042_0125;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_FORMAT_TYPE_IN_OBJECT: u32 = 0x0042_0042;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0043;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;
const FUTURE_RESULT_REASON: u32 = 0xF123_4567;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits TTLV")
        .try_checked()
        .expect("fixture tag is allocated")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("fixture item is valid")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut value = Structure::new();
    for child in items {
        value.try_push(child).expect("fixture structure is valid");
    }
    value
}

fn response_message(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> ResponseMessage {
    let version = structure([
        item(0x0042_006A, Value::integer(2)),
        item(0x0042_006B, Value::integer(1)),
    ]);
    let header = structure([
        item(0x0042_0069, Value::structure(version)),
        item(0x0042_0092, Value::date_time(1)),
        item(0x0042_000D, Value::integer(1)),
    ]);
    let mut fields = vec![
        item(0x0042_005C, Value::enumeration(EXPORT_OPERATION)),
        item(0x0042_007F, Value::enumeration(status)),
    ];
    if let Some(reason) = reason {
        fields.push(item(0x0042_007E, Value::enumeration(reason)));
    }
    if let Some(payload) = payload {
        fields.push(item(0x0042_007C, Value::structure(payload)));
    }
    ResponseMessage::try_from_ttlv(structure([
        item(0x0042_007A, Value::structure(header)),
        item(0x0042_000F, Value::structure(structure(fields))),
    ]))
    .expect("fixture is a valid KMIP 2.1 response")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture has one response batch item")
}

fn exported_key() -> Item {
    let value = structure([item(
        KEY_MATERIAL,
        Value::byte_string(b"synthetic-export-material".to_vec()),
    )]);
    let key_block = structure([
        item(KEY_FORMAT_TYPE_IN_OBJECT, Value::enumeration(1)),
        item(KEY_VALUE, Value::structure(value)),
    ]);
    item(
        SYMMETRIC_KEY,
        Value::structure(structure([item(KEY_BLOCK, Value::structure(key_block))])),
    )
}

#[test]
fn request_preserves_all_explicit_optional_choices_and_omits_absent_fields() {
    let empty_request = ExportRequest::new();
    assert_eq!(empty_request.unique_identifier(), None);
    let empty = empty_request
        .into_ttlv_payload()
        .expect("Table 217 permits omitted selectors");
    assert!(empty.view().children().is_empty());

    let specification = structure([item(UNKNOWN_VENDOR_TAG, Value::enumeration(0xF123_4568))]);
    let identifier = UniqueIdentifier::TextString("export-key".to_owned());
    let request = ExportRequest::new()
        .with_unique_identifier(identifier.clone())
        .with_key_format_type(KeyFormatType::from_raw(0xF123_4569))
        .with_key_wrap_type(KeyWrapType::from_raw(0xF123_456A))
        .with_key_compression_type(KeyCompressionType::from_raw(0xF123_456B))
        .with_key_wrapping_specification(specification);
    assert_eq!(request.unique_identifier(), Some(&identifier));
    assert_eq!(format!("{request:?}"), "ExportRequest([REDACTED])");
    let payload = request
        .into_ttlv_payload()
        .expect("all caller-selected options are structurally valid");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        vec![
            UNIQUE_IDENTIFIER,
            KEY_FORMAT_TYPE,
            KEY_WRAP_TYPE,
            KEY_COMPRESSION_TYPE,
            KEY_WRAPPING_SPECIFICATION,
        ]
    );
    assert!(
        fields[1]
            .with_value(|value| matches!(value, ValueView::Enumeration(v) if *v == 0xF123_4569))
    );
}

#[test]
fn success_preserves_all_response_fields_unknown_children_and_redacts_object_material() {
    let attributes = structure([item(0x0042_001D, Value::enumeration(1))]);
    let payload = structure([
        item(OBJECT_TYPE, Value::enumeration(2)),
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string("exported-key".to_owned()),
        ),
        item(ATTRIBUTES, Value::structure(attributes)),
        exported_key(),
        item(UNKNOWN_VENDOR_TAG, Value::integer(8)),
    ]);
    let message = response_message(SUCCESS, None, Some(payload));
    let response = ExportResponse::try_from_response_item(response_item(&message))
        .expect("Table 218 success contains all required fields");
    assert_eq!(response.object_type(), Some(ObjectType::from_raw(2)));
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString("exported-key".to_owned()))
    );
    assert!(response.attributes().is_some());
    assert!(response.object().is_some());
    let debug = format!("{response:?}");
    assert!(!debug.contains("synthetic-export-material"));
    drop(message);
    assert!(
        response.with_ttlv(|ttlv| ttlv.children().iter().any(|field| {
            field.tag().raw() == 0x0042_007C
                && field.with_value(|value| matches!(value, ValueView::Structure(_)))
        }))
    );
}

#[test]
fn failure_keeps_unknown_result_reason_and_malformed_success_is_rejected() {
    let failed = response_message(OPERATION_FAILED, Some(FUTURE_RESULT_REASON), None);
    let response = ExportResponse::try_from_response_item(response_item(&failed))
        .expect("Table 219 server errors remain typed results");
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(FUTURE_RESULT_REASON)
    );

    let malformed = response_message(SUCCESS, None, Some(structure([])));
    assert!(matches!(
        ExportResponse::try_from_response_item(response_item(&malformed)),
        Err(ExportError::MalformedSuccessPayload)
    ));
}

#[test]
fn export_response_accepts_generic_object_structure_and_all_identifier_forms() {
    let generic_object_tag = 0x0054_1235;
    for (identifier, identifier_value) in [
        (
            UniqueIdentifier::Enumeration(0xF123_4567),
            Value::enumeration(0xF123_4567),
        ),
        (UniqueIdentifier::Integer(-8), Value::integer(-8)),
    ] {
        let message = response_message(
            SUCCESS,
            None,
            Some(structure([
                item(OBJECT_TYPE, Value::enumeration(0xF123_4568)),
                item(UNIQUE_IDENTIFIER, identifier_value),
                item(
                    ATTRIBUTES,
                    Value::structure(structure([item(UNKNOWN_VENDOR_TAG, Value::integer(4))])),
                ),
                item(generic_object_tag, Value::structure(structure([]))),
                item(UNKNOWN_VENDOR_TAG, Value::text_string("ignored".to_owned())),
            ])),
        );
        let response = ExportResponse::try_from_response_item(response_item(&message))
            .expect("one otherwise-unknown structure can carry the object payload");
        assert_eq!(
            response.object_type(),
            Some(ObjectType::from_raw(0xF123_4568))
        );
        assert_eq!(response.unique_identifier(), Some(&identifier));
        assert_eq!(
            response.attributes().map(Item::item_type),
            Some(kmipkit_ttlv::ItemType::Structure)
        );
        assert_eq!(
            response.object().map(Item::tag).map(|tag| tag.raw()),
            Some(generic_object_tag)
        );
        assert!(!format!("{response:?}").contains("ignored"));
    }
}

#[test]
fn export_rejects_duplicate_wrongly_typed_and_ambiguous_success_fields() {
    let base = || {
        vec![
            item(OBJECT_TYPE, Value::enumeration(2)),
            item(UNIQUE_IDENTIFIER, Value::text_string("exported".to_owned())),
            item(ATTRIBUTES, Value::structure(structure([]))),
            exported_key(),
        ]
    };
    let mut malformed = Vec::new();
    let mut duplicate_object = base();
    duplicate_object.push(exported_key());
    malformed.push(structure(duplicate_object));
    let mut duplicate_identifier = base();
    duplicate_identifier.push(item(
        UNIQUE_IDENTIFIER,
        Value::text_string("second".to_owned()),
    ));
    malformed.push(structure(duplicate_identifier));
    let mut duplicate_attributes = base();
    duplicate_attributes.push(item(ATTRIBUTES, Value::structure(structure([]))));
    malformed.push(structure(duplicate_attributes));
    let mut wrong_object_type = base();
    wrong_object_type[0] = item(OBJECT_TYPE, Value::integer(2));
    malformed.push(structure(wrong_object_type));
    let mut wrong_identifier = base();
    wrong_identifier[1] = item(UNIQUE_IDENTIFIER, Value::boolean(false));
    malformed.push(structure(wrong_identifier));
    let mut wrong_attributes = base();
    wrong_attributes[2] = item(ATTRIBUTES, Value::integer(1));
    malformed.push(structure(wrong_attributes));
    let mut wrong_object = base();
    wrong_object[3] = item(SYMMETRIC_KEY, Value::integer(1));
    malformed.push(structure(wrong_object));
    malformed.push(structure([
        item(OBJECT_TYPE, Value::enumeration(2)),
        item(UNIQUE_IDENTIFIER, Value::text_string("exported".to_owned())),
        item(ATTRIBUTES, Value::structure(structure([]))),
        item(UNKNOWN_VENDOR_TAG, Value::structure(structure([]))),
        item(UNKNOWN_VENDOR_TAG + 1, Value::structure(structure([]))),
    ]));

    for payload in malformed {
        let message = response_message(SUCCESS, None, Some(payload));
        assert_eq!(
            ExportResponse::try_from_response_item(response_item(&message)).unwrap_err(),
            ExportError::MalformedSuccessPayload
        );
    }
}

#[test]
fn export_error_formatting_and_failure_views_are_payload_free() {
    let failed = response_message(OPERATION_FAILED, Some(FUTURE_RESULT_REASON), None);
    let response = ExportResponse::try_from_response_item(response_item(&failed))
        .expect("server failures preserve the shared result");
    assert_eq!(response.object_type(), None);
    assert_eq!(response.unique_identifier(), None);
    assert!(response.attributes().is_none());
    assert!(response.object().is_none());
    assert!(response.with_ttlv(|value| !value.children().is_empty()));
    assert!(format!("{response:?}").contains("ExportResponse"));

    let invalid_result =
        ExportError::InvalidOperationResult(ResultValidationError::SuccessForbidsReason);
    assert!(invalid_result.source().is_some());
    for error in [
        ExportError::UnexpectedOperation,
        ExportError::MissingResultStatus,
        invalid_result,
        ExportError::MissingSuccessPayload,
        ExportError::MalformedSuccessPayload,
        ExportError::MalformedResponseItem,
    ] {
        assert_ne!(error.to_string(), "");
    }
}
