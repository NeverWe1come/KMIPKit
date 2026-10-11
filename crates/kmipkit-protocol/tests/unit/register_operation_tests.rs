//! Register tests from OASIS KMIP v2.1 §6.1.43, Tables 291–294.
//! Traceability covers `KMIPKIT-REQ-SPEC-6.1.43-001`, `-005`, `-007`,
//! `-009-001`, `-009-002`, `-010-001`, `-010-002`, and `-011`.
//! These vectors are derived from the pinned tables, not official fixture runs.

use crate::async_operation_fixtures::{response_message, structure};
use crate::{
    AttributeSet, ObjectType, RegisterError, RegisterRequest, RegisterResponse,
    ResponseBatchItemView, ResponseMessage, ResultReason, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

const REGISTER_OPERATION: u32 = 0x0000_0003;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const INVALID_OBJECT_TYPE: u32 = 0x0000_0038;
const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const CERTIFICATE: u32 = 0x0042_0013;
const SECRET_DATA: u32 = 0x0042_0085;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const CERTIFICATE_LENGTH: u32 = 0x0042_00AD;
const DIGITAL_SIGNATURE_ALGORITHM: u32 = 0x0042_00AE;
const KEY_BLOCK: u32 = 0x0042_0040;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;
const SECRET_SENTINEL: &[u8] = b"REGISTER_SECRET_OBJECT_SENTINEL";

fn tag(raw_tag: u32) -> Tag {
    RawTag::new(raw_tag)
        .expect("source-derived tag fits TTLV width")
        .try_checked()
        .expect("source-derived tag is allocated")
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("source-derived tag and value form a valid item")
}

fn attrs(items: impl IntoIterator<Item = Item>) -> AttributeSet {
    AttributeSet::try_new(items).expect("fixture uses well-formed attributes")
}

fn key_object(algorithm: bool, length: bool) -> Item {
    let mut key_block = Vec::new();
    if algorithm {
        key_block.push(item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)));
    }
    if length {
        key_block.push(item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)));
    }
    let key_block = item(KEY_BLOCK, Value::structure(structure(key_block)));
    item(
        SYMMETRIC_KEY,
        Value::structure(structure([key_block])),
    )
}

fn symmetric_attributes(extra: impl IntoIterator<Item = Item>) -> AttributeSet {
    attrs(
        [item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(3))]
            .into_iter()
            .chain(extra),
    )
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one validated response batch item")
}

fn typed_response(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<RegisterResponse, RegisterError> {
    let message = response_message(operation, status, reason, None, payload);
    RegisterResponse::try_from_response_item(response_item(&message))
}

fn field_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

#[test]
fn request_emits_table_291_fields_in_order_and_preserves_object_item() {
    let sentinel = SECRET_SENTINEL.to_vec();
    let mut object_body = Structure::new();
    object_body
        .try_push(item(
            KEY_BLOCK,
            Value::structure(structure([item(
                CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(3),
            )])),
        ))
        .expect("nested fixture object fits TTLV structure limits");
    object_body
        .try_push(item(UNKNOWN_VENDOR_TAG, Value::byte_string(sentinel)))
        .expect("nested fixture object fits TTLV structure limits");
    let request = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([
            item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
            item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        ]),
        item(SYMMETRIC_KEY, Value::structure(object_body)),
    )
    .with_protection_storage_masks(structure([item(0x0042_0160, Value::integer(1))]));
    let payload = request
        .into_ttlv_payload()
        .expect("required Table 291 fields and Table 293 attributes are valid");

    assert_eq!(
        field_tags(&payload),
        [OBJECT_TYPE, ATTRIBUTES, SYMMETRIC_KEY, PROTECTION_STORAGE_MASKS]
    );
}

#[test]
fn request_requires_key_algorithm_and_length_in_attributes_or_key_block() {
    let result = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([]),
        key_object(false, false),
    )
    .into_ttlv_payload();
    assert!(result.is_err(), "Table 293 requires key algorithm and length");

    let encoded_in_key_block = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([]),
        key_object(true, true),
    )
    .into_ttlv_payload();
    assert!(encoded_in_key_block.is_ok());
}

#[test]
fn request_requires_algorithm_and_length_attributes_as_a_pair() {
    let algorithm_only = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3))]),
        key_object(false, false),
    )
    .into_ttlv_payload();
    let length_only = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))]),
        key_object(false, false),
    )
    .into_ttlv_payload();
    assert!(algorithm_only.is_err());
    assert!(length_only.is_err());
}

#[test]
fn request_requires_usage_mask_for_managed_cryptographic_objects() {
    let result = RegisterRequest::new(
        ObjectType::from_raw(2),
        attrs([
            item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
            item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        ]),
        key_object(false, false),
    )
    .into_ttlv_payload();
    assert!(result.is_err(), "Table 293 requires Cryptographic Usage Mask");
}

#[test]
fn request_requires_certificate_length_and_allows_embedded_signature_algorithm() {
    let certificate = item(CERTIFICATE, Value::structure(structure([])));
    let missing_length = RegisterRequest::new(
        ObjectType::from_raw(1),
        attrs([]),
        certificate,
    )
    .into_ttlv_payload();
    assert!(missing_length.is_err());

    let request = RegisterRequest::new(
        ObjectType::from_raw(1),
        attrs([
            item(CERTIFICATE_LENGTH, Value::integer(4096)),
            item(DIGITAL_SIGNATURE_ALGORITHM, Value::enumeration(1)),
        ]),
        item(CERTIFICATE, Value::structure(structure([]))),
    );
    assert!(request.into_ttlv_payload().is_ok());
}

#[test]
fn secret_data_does_not_require_key_algorithm_or_length() {
    let request = RegisterRequest::new(
        ObjectType::from_raw(7),
        attrs([item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(1))]),
        item(SECRET_DATA, Value::structure(structure([]))),
    );
    assert!(request.into_ttlv_payload().is_ok());
}

#[test]
fn request_debug_redacts_object_payload() {
    let object = item(
        SYMMETRIC_KEY,
        Value::structure(structure([item(
            UNKNOWN_VENDOR_TAG,
            Value::byte_string(SECRET_SENTINEL.to_vec()),
        )])),
    );
    let request = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([
            item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
            item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        ]),
        object,
    );
    assert!(!format!("{request:?}").contains("REGISTER_SECRET_OBJECT_SENTINEL"));
}

#[test]
fn successful_response_requires_and_preserves_unique_identifier_and_extensions() {
    let response = typed_response(
        REGISTER_OPERATION,
        SUCCESS,
        None,
        Some(structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string("registered-object".to_owned()),
            ),
            item(UNKNOWN_VENDOR_TAG, Value::integer(-7)),
        ])),
    )
    .expect("Table 292 success includes a Unique Identifier");
    assert_eq!(response.result().status().raw(), SUCCESS);
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString("registered-object".to_owned()))
    );
    assert!(response.with_ttlv(|view| view.children().iter().any(|field| {
        field.tag().raw() == UNKNOWN_VENDOR_TAG
    })));
}

#[test]
fn successful_response_rejects_missing_duplicate_or_mistyped_identifier() {
    let payloads = [
        structure([]),
        structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string("first".to_owned()),
            ),
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string("second".to_owned()),
            ),
        ]),
        structure([item(
            UNIQUE_IDENTIFIER,
            Value::byte_string(b"not-an-identifier".to_vec()),
        )]),
    ];
    for payload in payloads {
        assert!(typed_response(REGISTER_OPERATION, SUCCESS, None, Some(payload)).is_err());
    }
}

#[test]
fn failure_preserves_table_294_result_without_success_identifier() {
    let response = typed_response(
        REGISTER_OPERATION,
        OPERATION_FAILED,
        Some(INVALID_OBJECT_TYPE),
        None,
    )
    .expect("Table 294 failure remains a common operation result");
    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(INVALID_OBJECT_TYPE)
    );
    assert!(response.unique_identifier().is_none());
}

#[test]
fn response_conversion_rejects_a_different_operation() {
    let message = response_message(0x0000_0004, OPERATION_FAILED, Some(0x0000_0037), None, None);
    assert!(RegisterResponse::try_from_response_item(response_item(&message)).is_err());
}
