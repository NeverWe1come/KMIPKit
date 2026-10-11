//! Import payload and result contracts from OASIS KMIP v2.1 §6.1.25, Tables 238–240.
//! Local vectors derive from pinned source tables and are not official Test Case executions.

use crate::{
    AttributeSet, ImportError, ImportRequest, ImportResponse, KeyWrapType, ObjectType,
    ResponseBatchItemView, ResponseMessage, ResultReason, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

const IMPORT_OPERATION: u32 = 0x0000_002A;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const OBJECT_TYPE: u32 = 0x0042_0057;
const REPLACE_EXISTING: u32 = 0x0042_0072;
const KEY_WRAP_TYPE: u32 = 0x0042_00F8;
const ATTRIBUTES: u32 = 0x0042_0125;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_WRAPPING_DATA: u32 = 0x0042_0046;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0043;
const WRAPPING_METHOD: u32 = 0x0042_009E;
const ENCRYPTION_KEY_INFORMATION: u32 = 0x0042_003D;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
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

fn attributes() -> AttributeSet {
    AttributeSet::try_new([
        item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
    ])
    .expect("caller supplied object attributes are valid")
}

fn key_object() -> Item {
    let value = structure([item(
        KEY_MATERIAL,
        Value::byte_string(b"synthetic-import-material".to_vec()),
    )]);
    let key_block = structure([
        item(KEY_FORMAT_TYPE, Value::enumeration(1)),
        item(KEY_VALUE, Value::structure(value)),
    ]);
    item(
        SYMMETRIC_KEY,
        Value::structure(structure([item(KEY_BLOCK, Value::structure(key_block))])),
    )
}

fn wrapped_key_object() -> Item {
    let key_wrapping_data = structure([
        item(WRAPPING_METHOD, Value::enumeration(1)),
        item(
            ENCRYPTION_KEY_INFORMATION,
            Value::structure(structure([item(
                UNIQUE_IDENTIFIER,
                Value::text_string("wrapping-key".to_owned()),
            )])),
        ),
    ]);
    let value = structure([item(
        KEY_MATERIAL,
        Value::byte_string(b"synthetic-import-material".to_vec()),
    )]);
    let key_block = structure([
        item(KEY_FORMAT_TYPE, Value::enumeration(1)),
        item(KEY_VALUE, Value::structure(value)),
        item(KEY_WRAPPING_DATA, Value::structure(key_wrapping_data)),
    ]);
    item(
        SYMMETRIC_KEY,
        Value::structure(structure([item(KEY_BLOCK, Value::structure(key_block))])),
    )
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
        item(0x0042_005C, Value::enumeration(IMPORT_OPERATION)),
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

#[test]
fn request_preserves_required_fields_and_only_explicit_optional_choices() {
    let request = ImportRequest::new(
        UniqueIdentifier::TextString("import-key".to_owned()),
        ObjectType::from_raw(2),
        attributes(),
        key_object(),
    );
    let debug = format!("{request:?}");
    assert!(!debug.contains("import-key"));
    assert!(!debug.contains("synthetic-import-material"));
    let payload = request
        .into_ttlv_payload()
        .expect("Table 238 required values are caller supplied");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        vec![UNIQUE_IDENTIFIER, OBJECT_TYPE, ATTRIBUTES, SYMMETRIC_KEY]
    );
    assert!(fields[2].with_value(|value| matches!(value, ValueView::Structure(_))));
    assert!(fields[3].with_value(|value| matches!(value, ValueView::Structure(_))));
}

#[test]
fn replacement_and_conditional_wrap_type_are_preserved_without_unwrapping() {
    let request = ImportRequest::new(
        UniqueIdentifier::TextString("replace-key".to_owned()),
        ObjectType::from_raw(2),
        attributes(),
        wrapped_key_object(),
    )
    .with_replace_existing(true)
    .with_key_wrap_type(KeyWrapType::from_raw(0xF123_4568));
    let payload = request
        .into_ttlv_payload()
        .expect("caller-selected wrap type and replacement intent are preserved");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        vec![
            UNIQUE_IDENTIFIER,
            OBJECT_TYPE,
            REPLACE_EXISTING,
            KEY_WRAP_TYPE,
            ATTRIBUTES,
            SYMMETRIC_KEY,
        ]
    );
    assert!(fields[2].with_value(|value| matches!(value, ValueView::Boolean(true))));
    assert!(
        fields[3].with_value(
            |value| matches!(value, ValueView::Enumeration(raw) if *raw == 0xF123_4568)
        )
    );
}

#[test]
fn key_wrap_type_is_present_if_and_only_if_key_block_has_wrapping_data() {
    let wrapped_without_type = ImportRequest::new(
        UniqueIdentifier::TextString("wrapped-key".to_owned()),
        ObjectType::from_raw(2),
        attributes(),
        wrapped_key_object(),
    );
    assert!(wrapped_without_type.into_ttlv_payload().is_err());

    let unwrapped_with_type = ImportRequest::new(
        UniqueIdentifier::TextString("unwrapped-key".to_owned()),
        ObjectType::from_raw(2),
        attributes(),
        key_object(),
    )
    .with_key_wrap_type(KeyWrapType::from_raw(1));
    assert!(unwrapped_with_type.into_ttlv_payload().is_err());
}

#[test]
fn success_returns_identifier_and_failure_preserves_unknown_reason() {
    let success = response_message(
        SUCCESS,
        None,
        Some(structure([item(
            UNIQUE_IDENTIFIER,
            Value::text_string("imported-key".to_owned()),
        )])),
    );
    let response = ImportResponse::try_from_response_item(response_item(&success))
        .expect("Table 239 success contains the resulting Unique Identifier");
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString("imported-key".to_owned()))
    );

    let failure = response_message(OPERATION_FAILED, Some(FUTURE_RESULT_REASON), None);
    let response = ImportResponse::try_from_response_item(response_item(&failure))
        .expect("Table 240 server errors remain typed results");
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(FUTURE_RESULT_REASON)
    );
}

#[test]
fn malformed_success_payload_is_rejected() {
    let malformed = response_message(SUCCESS, None, Some(structure([])));
    assert!(matches!(
        ImportResponse::try_from_response_item(response_item(&malformed)),
        Err(ImportError::MalformedSuccessPayload)
    ));
}
