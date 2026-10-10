//! Get request and response tests derived from OASIS KMIP Specification v2.1
//! §6.1.19, Tables 220–221, and §6.1.19.1, Table 222. Valid Key Compression
//! Type, Key Format Type, and Key Wrap Type values come from §§11.24, 11.25
//! (Table 461), and 11.29; permitted Unique Identifier forms come from §4.58,
//! Tables 145–146. Any Object remains opaque as defined in §2. These are local
//! source-derived vectors, not official OASIS Test Cases.
//! Response traceability: KMIPKIT-ELEM-OP-C2S-GET; KMIPKIT-0017 FR-001–004,
//! FR-011; SC-001 and SC-002.
//!
//! Traceability: `KMIPKIT-ELEM-OP-C2S-GET`; KMIPKIT-0017 FR-002, FR-003;
//! SC-001 and SC-002.

use crate::{
    GetError, GetRequest, GetResponse, KeyCompressionType, KeyFormatType, KeyWrapType, ObjectType,
    ResponseBatchItemView, ResponseMessage, ResultReason, ResultStatus, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const GET_OPERATION: u32 = 0x0000_000A;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const KEY_FORMAT_TYPE_TAG: u32 = 0x0042_0042;
const KEY_WRAP_TYPE_TAG: u32 = 0x0042_00F8;
const KEY_COMPRESSION_TYPE_TAG: u32 = 0x0042_0041;
const KEY_WRAPPING_SPECIFICATION_TAG: u32 = 0x0042_0047;
const OBJECT_TYPE_TAG: u32 = 0x0042_0057;
const SYMMETRIC_KEY_TAG: u32 = 0x0042_008F;
const KEY_BLOCK_TAG: u32 = 0x0042_0040;
const KEY_VALUE_TAG: u32 = 0x0042_0045;
const KEY_MATERIAL_TAG: u32 = 0x0042_0043;
const TEST_EXTENSION_TAG: u32 = 0x0054_1234;
const SYMMETRIC_KEY_OBJECT_TYPE: u32 = 2;
const UNKNOWN_OBJECT_TYPE: u32 = 0xF123_4567;
const UNKNOWN_RESULT_STATUS: u32 = 0xF123_4568;
const UNKNOWN_RESULT_REASON: u32 = 0xF123_4569;
const OBJECT_IDENTIFIER: &str = "object-123";

const RESPONSE_HEADER_TAG: u32 = 0x0042_007A;
const PROTOCOL_VERSION_TAG: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR_TAG: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR_TAG: u32 = 0x0042_006B;
const TIME_STAMP_TAG: u32 = 0x0042_0092;
const BATCH_COUNT_TAG: u32 = 0x0042_000D;
const BATCH_ITEM_TAG: u32 = 0x0042_000F;
const OPERATION_TAG: u32 = 0x0042_005C;
const RESULT_STATUS_TAG: u32 = 0x0042_007F;
const RESULT_REASON_TAG: u32 = 0x0042_007E;
const RESULT_MESSAGE_TAG: u32 = 0x0042_007D;
const RESPONSE_PAYLOAD_TAG: u32 = 0x0042_007C;

fn tag(raw_tag: u32) -> Tag {
    RawTag::new(raw_tag)
        .expect("fixture tag fits the 24-bit KMIP field")
        .try_checked()
        .expect("fixture tag is assigned by the KMIP 2.1 catalog")
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("fixture item uses an allocated tag")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("fixture structure stays within the model depth limit");
    }
    structure
}

fn field_signature(payload: &Structure) -> Vec<(u32, ItemType)> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| (field.tag().raw(), field.item_type()))
        .collect()
}

fn assert_enumeration_field(request: GetRequest, tag: u32, value: u32) {
    let payload = request
        .to_ttlv_payload()
        .expect("a Table 220 Enumeration field is representable");
    let view = payload.view();
    let fields = view.children();

    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), tag);
    assert_eq!(fields[0].item_type(), ItemType::Enumeration);
    assert!(
        fields[0].with_value(
            |actual| matches!(actual, ValueView::Enumeration(actual) if *actual == value)
        )
    );
}

fn object_type_item(value: Value) -> Item {
    Item::new(
        RawTag::new(OBJECT_TYPE_TAG)
            .expect("fixture tag fits TTLV")
            .try_checked()
            .expect("Object Type is allocated in KMIP 2.1"),
        value,
    )
    .expect("fixture Object Type is representable")
}

fn unique_identifier_item(value: Value) -> Item {
    Item::new(
        RawTag::new(UNIQUE_IDENTIFIER_TAG)
            .expect("fixture tag fits TTLV")
            .try_checked()
            .expect("Unique Identifier is allocated in KMIP 2.1"),
        value,
    )
    .expect("fixture Unique Identifier is representable")
}

fn opaque_symmetric_key() -> Item {
    let key_material = item(
        KEY_MATERIAL_TAG,
        Value::byte_string(vec![0x00, 0x80, 0xFF, 0x01]),
    );
    let key_value = structure([key_material]);
    let key_block = structure([
        item(KEY_FORMAT_TYPE_TAG, Value::enumeration(1)),
        item(KEY_VALUE_TAG, Value::structure(key_value)),
        item(
            TEST_EXTENSION_TAG,
            Value::structure(structure([
                item(
                    TEST_EXTENSION_TAG,
                    Value::byte_string(vec![0x00, 0x81, 0xFE, 0x7F]),
                ),
                item(
                    TEST_EXTENSION_TAG,
                    Value::long_integer(i64::from_be_bytes([
                        0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
                    ])),
                ),
            ])),
        ),
        item(
            TEST_EXTENSION_TAG,
            Value::enumeration(UNKNOWN_RESULT_REASON),
        ),
    ]);

    item(
        SYMMETRIC_KEY_TAG,
        Value::structure(structure([item(
            KEY_BLOCK_TAG,
            Value::structure(key_block),
        )])),
    )
}

fn vendor_object() -> Item {
    item(
        TEST_EXTENSION_TAG,
        Value::structure(structure([
            item(
                TEST_EXTENSION_TAG,
                Value::byte_string(vec![0x00, 0x80, 0xFF]),
            ),
            item(TEST_EXTENSION_TAG, Value::enumeration(UNKNOWN_OBJECT_TYPE)),
        ])),
    )
}

fn response_message(
    status: u32,
    reason: Option<u32>,
    result_message: Option<&str>,
    payload: Option<Structure>,
) -> ResponseMessage {
    let protocol_version = structure([
        item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    let response_header = structure([
        item(PROTOCOL_VERSION_TAG, Value::structure(protocol_version)),
        item(TIME_STAMP_TAG, Value::date_time(1)),
        item(BATCH_COUNT_TAG, Value::integer(1)),
    ]);
    let mut batch_fields = vec![
        item(OPERATION_TAG, Value::enumeration(GET_OPERATION)),
        item(RESULT_STATUS_TAG, Value::enumeration(status)),
    ];
    if let Some(reason) = reason {
        batch_fields.push(item(RESULT_REASON_TAG, Value::enumeration(reason)));
    }
    if let Some(result_message) = result_message {
        batch_fields.push(item(
            RESULT_MESSAGE_TAG,
            Value::text_string(result_message.to_owned()),
        ));
    }
    if let Some(payload) = payload {
        batch_fields.push(item(RESPONSE_PAYLOAD_TAG, Value::structure(payload)));
    }

    ResponseMessage::try_from_ttlv(structure([
        item(RESPONSE_HEADER_TAG, Value::structure(response_header)),
        item(BATCH_ITEM_TAG, Value::structure(structure(batch_fields))),
    ]))
    .expect("fixture is a valid KMIP 2.1 response message")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one response batch item")
}

fn decode_get(message: &ResponseMessage) -> Result<GetResponse, GetError> {
    GetResponse::try_from_response_item(response_item(message))
}

fn successful_payload(fields: impl IntoIterator<Item = Item>) -> Structure {
    structure(fields)
}

fn valid_successful_payload() -> Structure {
    successful_payload([
        object_type_item(Value::enumeration(SYMMETRIC_KEY_OBJECT_TYPE)),
        unique_identifier_item(Value::text_string(OBJECT_IDENTIFIER.to_owned())),
        opaque_symmetric_key(),
    ])
}

fn malformed_success(payload: Structure) -> Result<GetResponse, GetError> {
    let message = response_message(SUCCESS, None, None, Some(payload));
    decode_get(&message)
}

#[derive(Eq, PartialEq)]
enum OpaqueValueSnapshot {
    Structure(Vec<(u32, Self)>),
    Enumeration(u32),
    ByteString(Vec<u8>),
    LongInteger(i64),
}

fn item_snapshot(item: &Item) -> (u32, OpaqueValueSnapshot) {
    (item.tag().raw(), item.with_value(value_snapshot))
}

fn value_snapshot(value: ValueView<'_>) -> OpaqueValueSnapshot {
    match value {
        ValueView::Structure(structure) => {
            OpaqueValueSnapshot::Structure(structure.children().iter().map(item_snapshot).collect())
        }
        ValueView::Enumeration(value) => OpaqueValueSnapshot::Enumeration(*value),
        ValueView::ByteString(value) => OpaqueValueSnapshot::ByteString(value.to_vec()),
        ValueView::LongInteger(value) => OpaqueValueSnapshot::LongInteger(*value),
        _ => panic!("opaque object fixture contains only captured TTLV value types"),
    }
}

#[test]
fn request_omits_every_optional_table_220_field_when_unspecified() {
    let payload = GetRequest::new()
        .to_ttlv_payload()
        .expect("an empty Table 220 request payload is valid");

    assert!(payload.view().children().is_empty());
}

#[test]
fn request_preserves_the_optional_unique_identifier() {
    let payload = GetRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("object-123".to_owned()))
        .to_ttlv_payload()
        .expect("Table 220 permits the optional Unique Identifier");
    let view = payload.view();
    let fields = view.children();

    assert_eq!(
        field_signature(&payload),
        [(UNIQUE_IDENTIFIER_TAG, ItemType::TextString)]
    );
    assert!(fields[0].with_value(
        |value| matches!(value, ValueView::TextString(actual) if actual == "object-123")
    ));
}

#[test]
fn request_preserves_the_optional_key_format_type_enumeration() {
    // §11.25, Table 461 assigns Raw the Enumeration value 1.
    assert_enumeration_field(
        GetRequest::new().with_key_format_type(KeyFormatType::from_raw(1)),
        KEY_FORMAT_TYPE_TAG,
        1,
    );
}

#[test]
fn request_preserves_the_optional_key_wrap_type_enumeration() {
    // §11.29 assigns Not Wrapped the Enumeration value 1.
    assert_enumeration_field(
        GetRequest::new().with_key_wrap_type(KeyWrapType::from_raw(1)),
        KEY_WRAP_TYPE_TAG,
        1,
    );
}

#[test]
fn request_preserves_the_optional_key_compression_type_enumeration() {
    // §11.24, Table 459 assigns EC Public Key Type Uncompressed the value 1.
    assert_enumeration_field(
        GetRequest::new().with_key_compression_type(KeyCompressionType::from_raw(1)),
        KEY_COMPRESSION_TYPE_TAG,
        1,
    );
}

#[test]
fn request_preserves_the_optional_key_wrapping_specification_structure() {
    let mut specification = Structure::new();
    specification
        .try_push(
            Item::new(
                RawTag::new(TEST_EXTENSION_TAG)
                    .expect("fixture tag fits TTLV")
                    .try_checked()
                    .expect("fixture tag uses the allocated extension range"),
                Value::integer(7),
            )
            .expect("fixture extension is a valid TTLV item"),
        )
        .expect("fixture specification stays within the nesting limit");
    let payload = GetRequest::new()
        .with_key_wrapping_specification(specification)
        .to_ttlv_payload()
        .expect("Table 220 permits the optional Key Wrapping Specification");
    let view = payload.view();
    let fields = view.children();

    assert_eq!(
        field_signature(&payload),
        [(KEY_WRAPPING_SPECIFICATION_TAG, ItemType::Structure)]
    );
    assert!(fields[0].with_value(|value| matches!(
        value,
        ValueView::Structure(structure)
            if structure.children().len() == 1
                && structure.children()[0].tag().raw() == TEST_EXTENSION_TAG
                && structure.children()[0].with_value(|nested| matches!(nested, ValueView::Integer(7)))
    )));
}

#[test]
fn request_serializes_present_table_220_fields_in_source_order() {
    let mut specification = Structure::new();
    specification
        .try_push(
            Item::new(
                RawTag::new(TEST_EXTENSION_TAG)
                    .expect("fixture tag fits TTLV")
                    .try_checked()
                    .expect("fixture tag uses the allocated extension range"),
                Value::integer(7),
            )
            .expect("fixture extension is a valid TTLV item"),
        )
        .expect("fixture specification stays within the nesting limit");
    let payload = GetRequest::new()
        .with_key_wrapping_specification(specification)
        .with_key_compression_type(KeyCompressionType::from_raw(1))
        .with_key_wrap_type(KeyWrapType::from_raw(1))
        .with_key_format_type(KeyFormatType::from_raw(1))
        .with_unique_identifier(UniqueIdentifier::TextString("object-123".to_owned()))
        .to_ttlv_payload()
        .expect("all supplied Table 220 fields are representable");

    assert_eq!(
        field_signature(&payload),
        [
            (UNIQUE_IDENTIFIER_TAG, ItemType::TextString),
            (KEY_FORMAT_TYPE_TAG, ItemType::Enumeration),
            (KEY_WRAP_TYPE_TAG, ItemType::Enumeration),
            (KEY_COMPRESSION_TYPE_TAG, ItemType::Enumeration),
            (KEY_WRAPPING_SPECIFICATION_TAG, ItemType::Structure),
        ]
    );
}

#[test]
fn successful_table_221_response_exposes_its_required_fields() {
    let message = response_message(SUCCESS, None, None, Some(valid_successful_payload()));
    let response = decode_get(&message).expect("Table 221 success contains all required fields");

    assert_eq!(response.result().status(), ResultStatus::from_raw(SUCCESS));
    assert_eq!(
        response.object_type(),
        Some(ObjectType::from_raw(SYMMETRIC_KEY_OBJECT_TYPE))
    );
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString(OBJECT_IDENTIFIER.to_owned()))
    );
    assert!(
        response.object().map(item_snapshot) == Some(item_snapshot(&opaque_symmetric_key())),
        "opaque object differs"
    );
}

#[test]
fn successful_table_221_response_preserves_each_permitted_unique_identifier_form() {
    // OASIS KMIP 2.1 §4.58, Tables 145–146 defines these permitted wire forms.
    let cases = [
        (
            Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            UniqueIdentifier::TextString(OBJECT_IDENTIFIER.to_owned()),
        ),
        (
            Value::enumeration(0xF123_4567),
            UniqueIdentifier::Enumeration(0xF123_4567),
        ),
        (Value::integer(-12_345), UniqueIdentifier::Integer(-12_345)),
    ];

    for (wire_value, expected) in cases {
        let payload = successful_payload([
            object_type_item(Value::enumeration(SYMMETRIC_KEY_OBJECT_TYPE)),
            unique_identifier_item(wire_value),
            opaque_symmetric_key(),
        ]);
        let message = response_message(SUCCESS, None, None, Some(payload));
        let response = decode_get(&message)
            .expect("Table 221 success preserves a permitted Unique Identifier form");

        assert_eq!(response.unique_identifier(), Some(&expected));
    }
}

#[test]
fn successful_table_221_response_preserves_the_nested_any_object_ttlv_tree() {
    let expected_object = opaque_symmetric_key();
    let message = response_message(SUCCESS, None, None, Some(valid_successful_payload()));
    let response = decode_get(&message).expect("Table 221 success preserves Any Object");

    assert!(
        response.object().map(item_snapshot) == Some(item_snapshot(&expected_object)),
        "opaque object differs; nested fields must retain order, repeated tags, unknown values, and scalar bits"
    );
}

#[test]
fn successful_table_221_response_preserves_unknown_object_type_and_allocated_vendor_tag() {
    let payload = successful_payload([
        object_type_item(Value::enumeration(UNKNOWN_OBJECT_TYPE)),
        unique_identifier_item(Value::text_string(OBJECT_IDENTIFIER.to_owned())),
        vendor_object(),
    ]);
    let message = response_message(SUCCESS, None, None, Some(payload));
    let response = decode_get(&message)
        .expect("a future object type and allocated vendor object tag remain representable");

    assert_eq!(
        response.object_type(),
        Some(ObjectType::from_raw(UNKNOWN_OBJECT_TYPE))
    );
    assert!(
        response.object().map(item_snapshot) == Some(item_snapshot(&vendor_object())),
        "opaque vendor object differs"
    );
}

#[test]
fn failed_table_222_response_preserves_result_message_and_omits_success_fields() {
    let reason = ResultReason::from_raw(0x0000_0013); // Key Value Not Present.
    let message = response_message(
        OPERATION_FAILED,
        Some(reason.raw()),
        Some("requested key value is unavailable"),
        None,
    );
    let response = decode_get(&message).expect("Table 222 errors have no success payload");

    assert_eq!(
        response.result().status(),
        ResultStatus::from_raw(OPERATION_FAILED)
    );
    assert_eq!(response.result().reason(), Some(reason));
    assert_eq!(
        response.result().message().map(|message| message.as_str()),
        Some("requested key value is unavailable")
    );
    assert!(response.object_type().is_none());
    assert!(response.unique_identifier().is_none());
    assert!(response.object().is_none());
}

#[test]
fn failed_get_response_accepts_each_result_reason_listed_in_table_222() {
    // OASIS KMIP 2.1 §6.1.19.1, Table 222 lists these Get error reasons.
    let table_222_reason_names = [
        "Bad Cryptographic Parameters",
        "Encoding Option Error",
        "Incompatible Cryptographic Usage Mask",
        "Invalid Object Type",
        "Key Compression Type Not Supported",
        "Key Format Type Not Supported",
        "Key Value Not Present",
        "Key Wrap Type Not Supported",
        "Not Extractable",
        "Object Not Found",
        "Sensitive",
        "Wrapping Object Archived",
        "Wrapping Object Destroyed",
        "Wrapping Object Not Found",
        "Attestation Failed",
        "Attestation Required",
        "Feature Not Supported",
        "Invalid Field",
        "Invalid Message",
        "Operation Not Supported",
        "Permission Denied",
        "Response Too Large",
    ];

    for expected_name in table_222_reason_names {
        let reason = ResultReason::known_values()
            .iter()
            .find_map(|(reason, name)| (*name == expected_name).then_some(*reason))
            .expect("Table 222 Result Reason is present in the pinned KMIP 2.1 catalog");
        let message = response_message(OPERATION_FAILED, Some(reason.raw()), None, None);
        let response = decode_get(&message)
            .expect("Table 222 Operation Failed preserves its applicable Result Reason");

        assert_eq!(
            response.result().status(),
            ResultStatus::from_raw(OPERATION_FAILED)
        );
        assert_eq!(response.result().reason(), Some(reason), "{expected_name}");
        assert!(response.object().is_none(), "{expected_name}");
    }
}

#[test]
fn unknown_get_result_status_and_reason_values_are_preserved() {
    let message = response_message(
        UNKNOWN_RESULT_STATUS,
        Some(UNKNOWN_RESULT_REASON),
        None,
        Some(valid_successful_payload()),
    );
    let response = decode_get(&message).expect("unknown result values remain representable");

    assert_eq!(response.result().status().raw(), UNKNOWN_RESULT_STATUS);
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(UNKNOWN_RESULT_REASON)
    );
    assert!(response.object_type().is_none());
    assert!(response.unique_identifier().is_none());
    assert!(response.object().is_none());
}

#[test]
fn successful_response_rejects_each_missing_table_221_required_field() {
    let cases = [
        successful_payload([
            unique_identifier_item(Value::text_string(OBJECT_IDENTIFIER.to_owned())),
            opaque_symmetric_key(),
        ]),
        successful_payload([
            object_type_item(Value::enumeration(SYMMETRIC_KEY_OBJECT_TYPE)),
            opaque_symmetric_key(),
        ]),
        successful_payload([
            object_type_item(Value::enumeration(SYMMETRIC_KEY_OBJECT_TYPE)),
            unique_identifier_item(Value::text_string(OBJECT_IDENTIFIER.to_owned())),
        ]),
    ];

    for payload in cases {
        assert!(
            malformed_success(payload).is_err(),
            "each Table 221 required field must be present"
        );
    }
}

#[test]
fn successful_response_rejects_duplicate_table_221_required_fields() {
    let duplicate_object_type = successful_payload([
        object_type_item(Value::enumeration(SYMMETRIC_KEY_OBJECT_TYPE)),
        object_type_item(Value::enumeration(SYMMETRIC_KEY_OBJECT_TYPE)),
        unique_identifier_item(Value::text_string(OBJECT_IDENTIFIER.to_owned())),
        opaque_symmetric_key(),
    ]);
    let duplicate_identifier = successful_payload([
        object_type_item(Value::enumeration(SYMMETRIC_KEY_OBJECT_TYPE)),
        unique_identifier_item(Value::text_string(OBJECT_IDENTIFIER.to_owned())),
        unique_identifier_item(Value::text_string("duplicate-id".to_owned())),
        opaque_symmetric_key(),
    ]);
    let duplicate_any_object = successful_payload([
        object_type_item(Value::enumeration(SYMMETRIC_KEY_OBJECT_TYPE)),
        unique_identifier_item(Value::text_string(OBJECT_IDENTIFIER.to_owned())),
        opaque_symmetric_key(),
        opaque_symmetric_key(),
    ]);

    for payload in [
        duplicate_object_type,
        duplicate_identifier,
        duplicate_any_object,
    ] {
        assert!(
            malformed_success(payload).is_err(),
            "each Table 221 required field occurs exactly once"
        );
    }
}

#[test]
fn successful_response_rejects_each_mistyped_table_221_required_field() {
    let wrong_object_type = successful_payload([
        object_type_item(Value::integer(SYMMETRIC_KEY_OBJECT_TYPE as i32)),
        unique_identifier_item(Value::text_string(OBJECT_IDENTIFIER.to_owned())),
        opaque_symmetric_key(),
    ]);
    let wrong_unique_identifier = successful_payload([
        object_type_item(Value::enumeration(SYMMETRIC_KEY_OBJECT_TYPE)),
        unique_identifier_item(Value::boolean(true)),
        opaque_symmetric_key(),
    ]);
    let wrong_any_object_type = successful_payload([
        object_type_item(Value::enumeration(SYMMETRIC_KEY_OBJECT_TYPE)),
        unique_identifier_item(Value::text_string(OBJECT_IDENTIFIER.to_owned())),
        item(SYMMETRIC_KEY_TAG, Value::boolean(true)),
    ]);

    for payload in [
        wrong_object_type,
        wrong_unique_identifier,
        wrong_any_object_type,
    ] {
        assert!(
            malformed_success(payload).is_err(),
            "Table 221 required fields use their defined TTLV item types"
        );
    }
}
