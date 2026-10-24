//! Import payload and result contracts from OASIS KMIP v2.1 §6.1.25, Tables 238–240.
//! `import_preserves_wrapped_secret_data_without_key_wrap_type` also traces §2.7/Table 8.
//! Local vectors derive from pinned source tables and are not official Test Case executions.

use crate::{
    AttributeSet, ImportError, ImportRequest, ImportResponse, KeyWrapType, ObjectType,
    ResponseBatchItemView, ResponseMessage, ResultReason, ResultValidationError, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};
use std::error::Error as _;

const IMPORT_OPERATION: u32 = 0x0000_002A;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const OBJECT_TYPE: u32 = 0x0042_0057;
const KEY_WRAP_TYPE: u32 = 0x0042_00F8;
const ATTRIBUTES: u32 = 0x0042_0125;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const PGP_KEY: u32 = 0x0042_00BD;
const PGP_KEY_VERSION: u32 = 0x0042_00BE;
const SECRET_DATA: u32 = 0x0042_0085;
const SECRET_DATA_TYPE: u32 = 0x0042_0086;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_WRAPPING_DATA: u32 = 0x0042_0046;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0043;
const WRAPPING_METHOD: u32 = 0x0042_009E;
const ENCRYPTION_KEY_INFORMATION: u32 = 0x0042_0036;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const FUTURE_RESULT_REASON: u32 = 0xF123_4567;
const WRAPPED_SECRET_DATA_SENTINEL: &[u8] = b"KMIPKIT_WRAPPED_SECRET_DATA_FIXTURE";

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
    let key_block = structure([
        item(KEY_FORMAT_TYPE, Value::enumeration(1)),
        synthetic_wrapped_key_value(),
        encryption_key_wrapping_data(),
    ]);
    item(
        SYMMETRIC_KEY,
        Value::structure(structure([item(KEY_BLOCK, Value::structure(key_block))])),
    )
}

fn wrapped_pgp_key_object() -> Item {
    let key_block = structure([
        item(KEY_FORMAT_TYPE, Value::enumeration(1)),
        synthetic_wrapped_key_value(),
        encryption_key_wrapping_data(),
    ]);
    item(
        PGP_KEY,
        Value::structure(structure([
            item(PGP_KEY_VERSION, Value::integer(4)),
            item(KEY_BLOCK, Value::structure(key_block)),
        ])),
    )
}

fn pgp_key_object_with_version(version: i32, key_bytes: &[u8]) -> Item {
    item(
        PGP_KEY,
        Value::structure(structure([
            item(PGP_KEY_VERSION, Value::integer(version)),
            item(
                KEY_BLOCK,
                Value::structure(structure([
                    item(KEY_FORMAT_TYPE, Value::enumeration(1)),
                    item(KEY_VALUE, Value::byte_string(key_bytes.to_vec())),
                ])),
            ),
        ])),
    )
}

fn wrapped_secret_data_object() -> Item {
    let key_block = structure([
        item(KEY_FORMAT_TYPE, Value::enumeration(1)),
        item(
            KEY_VALUE,
            Value::byte_string(WRAPPED_SECRET_DATA_SENTINEL.to_vec()),
        ),
        encryption_key_wrapping_data(),
    ]);
    item(
        SECRET_DATA,
        Value::structure(structure([
            item(SECRET_DATA_TYPE, Value::enumeration(1)),
            item(KEY_BLOCK, Value::structure(key_block)),
        ])),
    )
}

fn synthetic_wrapped_key_value() -> Item {
    // Synthetic ciphertext; KMIPKit transports these bytes and does not unwrap them.
    item(
        KEY_VALUE,
        Value::byte_string(b"synthetic-wrapped-key-value".to_vec()),
    )
}

fn encryption_key_wrapping_data() -> Item {
    item(
        KEY_WRAPPING_DATA,
        Value::structure(structure([
            item(WRAPPING_METHOD, Value::enumeration(1)),
            item(
                ENCRYPTION_KEY_INFORMATION,
                Value::structure(structure([item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string("wrapping-key".to_owned()),
                )])),
            ),
        ])),
    )
}

fn key_block_field_matches(
    object: &Item,
    field_tag: u32,
    matches_field: impl Fn(&Item) -> bool,
) -> bool {
    object.with_value(|object_value| match object_value {
        ValueView::Structure(object_fields) => object_fields.children().iter().any(|key_block| {
            key_block.tag().raw() == KEY_BLOCK
                && key_block.with_value(|key_block_value| match key_block_value {
                    ValueView::Structure(key_block_fields) => key_block_fields
                        .children()
                        .iter()
                        .any(|field| field.tag().raw() == field_tag && matches_field(field)),
                    _ => false,
                })
        }),
        _ => false,
    })
}

fn wrapped_object_has_encryption_key_information(object: &Item) -> bool {
    key_block_field_matches(object, KEY_WRAPPING_DATA, |field| {
        field.with_value(|wrapping_value| match wrapping_value {
            ValueView::Structure(wrapping_fields) => {
                wrapping_fields.children().iter().any(|information| {
                    // OASIS KMIP 2.1 §3.3/Table 15 and §11.56/Table 487.
                    information.tag().raw() == 0x0042_0036
                        && information.with_value(|value| matches!(value, ValueView::Structure(_)))
                })
            }
            _ => false,
        })
    })
}

fn wrapped_object_has_byte_string_key_value(object: &Item) -> bool {
    key_block_field_matches(object, KEY_VALUE, |field| {
        field.with_value(|value| matches!(value, ValueView::ByteString(_)))
    })
}

fn wrapped_object_has_key_value_bytes(object: &Item, expected: &[u8]) -> bool {
    key_block_field_matches(object, KEY_VALUE, |field| {
        field.with_value(|value| matches!(value, ValueView::ByteString(bytes) if bytes == expected))
    })
}

fn response_message(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> ResponseMessage {
    response_message_for_operation(IMPORT_OPERATION, status, reason, payload)
}

fn response_message_for_operation(
    operation: u32,
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
        item(0x0042_005C, Value::enumeration(operation)),
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
    assert_eq!(
        request.unique_identifier(),
        &UniqueIdentifier::TextString("import-key".to_owned())
    );
    assert_eq!(request.object_type(), ObjectType::from_raw(2));
    assert_eq!(request.replace_existing(), None);
    assert_eq!(request.key_wrap_type(), None);
    assert_eq!(request.attributes().len(), 2);
    assert_eq!(request.object().tag().raw(), SYMMETRIC_KEY);
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
            // OASIS KMIP 2.1 §11.56 assigns Replace Existing tag 0x420124.
            0x0042_0124,
            KEY_WRAP_TYPE,
            ATTRIBUTES,
            SYMMETRIC_KEY,
        ]
    );
    assert!(fields[2].with_value(|value| matches!(value, ValueView::Boolean(true))));
    assert!(wrapped_object_has_encryption_key_information(&fields[5]));
    assert!(wrapped_object_has_byte_string_key_value(&fields[5]));
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
fn wrapped_pgp_key_accepts_key_wrap_type() {
    let pgp_key = wrapped_pgp_key_object();
    // OASIS KMIP 2.1 §3.1/Table 12 requires Key Format Type in every Key Block.
    assert!(
        key_block_field_matches(&pgp_key, KEY_FORMAT_TYPE, |field| {
            field.with_value(|value| matches!(value, ValueView::Enumeration(_)))
        }),
        "Key Block includes the required Key Format Type Enumeration"
    );

    let request = ImportRequest::new(
        UniqueIdentifier::TextString("wrapped-pgp-key".to_owned()),
        ObjectType::from_raw(9),
        attributes(),
        pgp_key,
    )
    .with_key_wrap_type(KeyWrapType::from_raw(1));

    // OASIS KMIP 2.1 §2.4/Table 5 defines PGP Key as a key object with Key Block;
    // §6.1.25/Table 238 requires Key Wrap Type iff the key object is wrapped.
    let payload = request
        .into_ttlv_payload()
        .expect("wrapped PGP Key can carry its explicit Key Wrap Type");
    assert!(payload.view().children().iter().any(|field| {
        field.tag().raw() == KEY_WRAP_TYPE
            && field.with_value(|value| matches!(value, ValueView::Enumeration(raw) if *raw == 1))
    }));
}

#[test]
fn import_preserves_pgp_key_versions_and_opaque_key_bytes() {
    // KMIP Specification v2.1 §2.4/Table 5 requires an Integer version and a Key Block.
    // These synthetic values exercise byte preservation without parsing PGP material.
    for (version, key_bytes) in [
        (3, b"synthetic-pgp-public-key-only".as_slice()),
        (4, b"synthetic-pgp-public-and-private-key".as_slice()),
    ] {
        let request = ImportRequest::new(
            UniqueIdentifier::TextString(format!("pgp-version-{version}")),
            ObjectType::from_raw(9),
            AttributeSet::try_new([]).expect("empty PGP attributes are valid"),
            pgp_key_object_with_version(version, key_bytes),
        );
        let payload = request
            .into_ttlv_payload()
            .expect("Import preserves a caller-supplied PGP Key Object");
        let payload_view = payload.view();
        let object = payload_view
            .children()
            .iter()
            .find(|field| field.tag().raw() == PGP_KEY)
            .expect("Import payload retains its PGP Key object");
        assert!(object.with_value(|value| match value {
            ValueView::Structure(fields) => fields.children().iter().any(|field| {
                field.tag().raw() == PGP_KEY_VERSION
                    && field.with_value(
                        |value| matches!(value, ValueView::Integer(actual) if *actual == version),
                    )
            }),
            _ => false,
        }));
        assert!(wrapped_object_has_key_value_bytes(object, key_bytes));
    }
}

#[test]
fn import_preserves_wrapped_secret_data_without_key_wrap_type() {
    // OASIS KMIP v2.1 §2.7 says Secret Data Key Value MAY be wrapped; Table 8
    // requires Secret Data Type and Key Block. §3.1/Table 12 encodes wrapped
    // Key Value as a byte string, and the preceding §3.1 prose requires Key
    // Wrapping Data when it is wrapped. §6.1.25/Table 238 limits Key Wrap Type
    // to a wrapped key object, so this Secret Data request omits that field.
    let request = ImportRequest::new(
        UniqueIdentifier::TextString("wrapped-secret-data".to_owned()),
        ObjectType::from_raw(7),
        AttributeSet::new(),
        wrapped_secret_data_object(),
    );

    let payload = request
        .into_ttlv_payload()
        .expect("wrapped Secret Data object is preserved by Import");
    let payload_view = payload.view();
    let payload_fields = payload_view.children();
    assert_eq!(
        payload_fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        vec![UNIQUE_IDENTIFIER, OBJECT_TYPE, ATTRIBUTES, SECRET_DATA]
    );
    assert!(
        !payload_fields
            .iter()
            .any(|field| field.tag().raw() == KEY_WRAP_TYPE)
    );
    assert!(key_block_field_matches(
        &payload_fields[3],
        KEY_FORMAT_TYPE,
        |field| field.with_value(|value| matches!(value, ValueView::Enumeration(raw) if *raw == 1))
    ));
    assert!(wrapped_object_has_byte_string_key_value(&payload_fields[3]));
    assert!(wrapped_object_has_key_value_bytes(
        &payload_fields[3],
        WRAPPED_SECRET_DATA_SENTINEL
    ));
    assert!(wrapped_object_has_encryption_key_information(
        &payload_fields[3]
    ));
    let secret_data_type_is_preserved = payload_fields[3].with_value(|value| match value {
        ValueView::Structure(fields) => fields.children().iter().any(|field| {
            field.tag().raw() == SECRET_DATA_TYPE
                && field
                    .with_value(|value| matches!(value, ValueView::Enumeration(raw) if *raw == 1))
        }),
        _ => false,
    });
    assert!(secret_data_type_is_preserved);
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

#[test]
fn import_accepts_non_key_objects_and_rejects_non_structure_and_invalid_wrap_state() {
    let secret_data_tag = 0x0042_0085;
    let secret_data = item(secret_data_tag, Value::structure(structure([])));
    let request = ImportRequest::new(
        UniqueIdentifier::Integer(-5),
        ObjectType::from_raw(8),
        attributes(),
        secret_data,
    )
    .with_replace_existing(false);
    assert_eq!(request.unique_identifier(), &UniqueIdentifier::Integer(-5));
    assert_eq!(request.replace_existing(), Some(false));
    assert!(request.into_ttlv_payload().is_ok());

    let non_key_with_wrap_type = ImportRequest::new(
        UniqueIdentifier::TextString("secret".to_owned()),
        ObjectType::from_raw(8),
        attributes(),
        item(secret_data_tag, Value::structure(structure([]))),
    )
    .with_key_wrap_type(KeyWrapType::from_raw(1));
    assert!(non_key_with_wrap_type.into_ttlv_payload().is_err());

    let primitive_object = ImportRequest::new(
        UniqueIdentifier::TextString("bad-object".to_owned()),
        ObjectType::from_raw(8),
        attributes(),
        item(secret_data_tag, Value::integer(9)),
    );
    assert!(primitive_object.into_ttlv_payload().is_err());

    let missing_key_block = ImportRequest::new(
        UniqueIdentifier::TextString("missing-block".to_owned()),
        ObjectType::from_raw(2),
        attributes(),
        item(SYMMETRIC_KEY, Value::structure(structure([]))),
    )
    .with_key_wrap_type(KeyWrapType::from_raw(1));
    assert!(missing_key_block.into_ttlv_payload().is_err());
}

#[test]
fn import_response_accepts_all_wire_identifier_forms_and_rejects_malformed_ones() {
    for (identifier, encoded) in [
        (
            UniqueIdentifier::Enumeration(0xF123_4567),
            Value::enumeration(0xF123_4567),
        ),
        (UniqueIdentifier::Integer(-9), Value::integer(-9)),
    ] {
        let message = response_message(
            SUCCESS,
            None,
            Some(structure([item(UNIQUE_IDENTIFIER, encoded)])),
        );
        let response = ImportResponse::try_from_response_item(response_item(&message))
            .expect("unknown and integer identifier values are preserved");
        assert_eq!(response.unique_identifier(), Some(&identifier));
        assert!(response.with_ttlv(|ttlv| !ttlv.children().is_empty()));
        assert!(format!("{response:?}").contains("ImportResponse"));
    }

    let duplicate = response_message(
        SUCCESS,
        None,
        Some(structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            item(UNIQUE_IDENTIFIER, Value::integer(2)),
        ])),
    );
    assert_eq!(
        ImportResponse::try_from_response_item(response_item(&duplicate)).unwrap_err(),
        ImportError::MalformedSuccessPayload
    );
    let wrong_type = response_message(
        SUCCESS,
        None,
        Some(structure([item(UNIQUE_IDENTIFIER, Value::boolean(true))])),
    );
    assert_eq!(
        ImportResponse::try_from_response_item(response_item(&wrong_type)).unwrap_err(),
        ImportError::MalformedSuccessPayload
    );
}

#[test]
fn import_error_formatting_exposes_only_safe_categories() {
    let invalid_result =
        ImportError::InvalidOperationResult(ResultValidationError::SuccessForbidsReason);
    assert!(invalid_result.source().is_some());
    assert!(ImportError::UnexpectedOperation.source().is_none());
    for error in [
        ImportError::UnexpectedOperation,
        ImportError::MissingResultStatus,
        invalid_result,
        ImportError::MissingSuccessPayload,
        ImportError::MalformedSuccessPayload,
        ImportError::MalformedResponseItem,
    ] {
        assert_ne!(error.to_string(), "");
    }
}

#[test]
fn import_response_ignores_unknown_payload_fields_and_rejects_other_operation() {
    let success = response_message(
        SUCCESS,
        None,
        Some(structure([
            item(0x0054_1234, Value::integer(5)),
            item(UNIQUE_IDENTIFIER, Value::text_string("imported".to_owned())),
        ])),
    );
    assert_eq!(
        ImportResponse::try_from_response_item(response_item(&success))
            .expect("unknown extension fields remain in the generic response")
            .unique_identifier(),
        Some(&UniqueIdentifier::TextString("imported".to_owned()))
    );

    let other_operation = response_message_for_operation(
        IMPORT_OPERATION + 1,
        OPERATION_FAILED,
        Some(FUTURE_RESULT_REASON),
        None,
    );
    assert_eq!(
        ImportResponse::try_from_response_item(response_item(&other_operation)).unwrap_err(),
        ImportError::UnexpectedOperation
    );
}
