//! Create Split Key payload tests derived from KMIP Specification v2.1
//! §6.1.10, Tables 193–195, and the Table 9 Split Key object definition.
//! Prime Field Size enforcement for method 3 is the accepted `KMIPKit` FR-015
//! caller-input policy; Table 193 itself marks the request field optional.

use crate::{
    AttributeSet, CreateSplitKeyError, CreateSplitKeyRequest, CreateSplitKeyResponse, ObjectType,
    ResultReason, ResultValidationError, SplitKeyMethod, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const CREATE_SPLIT_KEY_OPERATION: u32 = 0x0000_0003;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;

const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const SPLIT_KEY_PARTS: u32 = 0x0042_008b;
const SPLIT_KEY_THRESHOLD: u32 = 0x0042_008c;
const SPLIT_KEY_METHOD: u32 = 0x0042_008a;
const PRIME_FIELD_SIZE: u32 = 0x0042_0062;
const ATTRIBUTES: u32 = 0x0042_0125;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015f;

const RESPONSE_HEADER: u32 = 0x0042_007a;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006a;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006b;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000d;
const BATCH_ITEM: u32 = 0x0042_000f;
const OPERATION: u32 = 0x0042_005c;
const RESULT_STATUS: u32 = 0x0042_007f;
const RESULT_REASON: u32 = 0x0042_007e;
const RESULT_MESSAGE: u32 = 0x0042_007d;
const RESPONSE_PAYLOAD: u32 = 0x0042_007c;

const TABLE_195_REASONS: &[u32] = &[
    0x0000_0024, // Bad Cryptographic Parameters
    0x0000_000a, // Cryptographic Failure
    0x0000_002c, // Invalid Attribute
    0x0000_002d, // Invalid Attribute Value
    0x0000_0030, // Invalid Object Type
    0x0000_0001, // Item Not Found
    0x0000_0035, // Non Unique Name Attribute
    0x0000_003a, // Server Limit Exceeded
    0x0000_003e, // Unsupported Cryptographic Parameters
    0x0000_0015, // Attestation Failed
    0x0000_0014, // Attestation Required
    0x0000_0008, // Feature Not Supported
    0x0000_0007, // Invalid Field
    0x0000_0004, // Invalid Message
    0x0000_0005, // Operation Not Supported
    0x0000_000c, // Permission Denied
    0x0000_0044, // Protection Storage Unavailable
    0x0000_0002, // Response Too Large
];

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits the 24-bit KMIP field")
        .try_checked()
        .expect("fixture tag is allocated by the KMIP 2.1 catalog")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("fixture item uses a checked tag")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("fixture structure remains within the model depth limit");
    }
    structure
}

fn structure_at_depth(depth: usize) -> Structure {
    assert!((1..=64).contains(&depth));

    let mut structure = Structure::new();
    for _ in 1..depth {
        let child = Item::new(tag(0x0042_0173), Value::structure(structure))
            .expect("a checked tag and nested Structure must construct an item");
        let mut parent = Structure::new();
        parent
            .try_push(child)
            .expect("nesting within the model depth limit must succeed");
        structure = parent;
    }
    structure
}

fn attributes(items: impl IntoIterator<Item = Item>) -> AttributeSet {
    AttributeSet::try_new(items).expect("fixture contains valid direct Object Attribute items")
}

fn request(method: u32) -> CreateSplitKeyRequest {
    CreateSplitKeyRequest::new(
        ObjectType::from_raw(7),
        3,
        2,
        SplitKeyMethod::from_raw(method),
        AttributeSet::new(),
    )
}

fn response_message(
    status: u32,
    reason: Option<u32>,
    result_message: Option<&str>,
    response_payload: Option<Structure>,
) -> crate::ResponseMessage {
    let protocol_version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(protocol_version)),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut fields = vec![
        item(OPERATION, Value::enumeration(CREATE_SPLIT_KEY_OPERATION)),
        item(RESULT_STATUS, Value::enumeration(status)),
    ];
    if let Some(reason) = reason {
        fields.push(item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(message) = result_message {
        fields.push(item(RESULT_MESSAGE, Value::text_string(message.to_owned())));
    }
    if let Some(payload) = response_payload {
        fields.push(item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    crate::ResponseMessage::try_from_ttlv(structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(structure(fields))),
    ]))
    .expect("fixture is a structurally valid KMIP 2.1 response")
}

fn decode_response(
    response: &crate::ResponseMessage,
) -> Result<CreateSplitKeyResponse, CreateSplitKeyError> {
    CreateSplitKeyResponse::try_from_response_item(
        response
            .batch_items()
            .next()
            .expect("fixture has one response batch item"),
    )
}

#[test]
fn request_preserves_table_193_fields_order_and_exact_input_identifier() {
    let request = request(2)
        .with_unique_identifier(UniqueIdentifier::Enumeration(41))
        .with_protection_storage_masks(Structure::new());
    assert_eq!(request.object_type(), ObjectType::from_raw(7));
    assert_eq!(
        request.unique_identifier(),
        Some(&UniqueIdentifier::Enumeration(41))
    );
    assert_eq!(request.split_key_parts(), 3);
    assert_eq!(request.split_key_threshold(), 2);
    assert_eq!(
        request.split_key_method(),
        SplitKeyMethod::POLYNOMIAL_SHARING_GF_2_16
    );
    assert_eq!(request.prime_field_size(), None);
    assert!(request.attributes().as_items().is_empty());
    assert!(request.protection_storage_masks().is_some());
    let payload = request
        .into_ttlv_payload()
        .expect("non-polynomial method does not require Prime Field Size");
    let payload_view = payload.view();
    let fields = payload_view.children();

    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [
            OBJECT_TYPE,
            UNIQUE_IDENTIFIER,
            SPLIT_KEY_PARTS,
            SPLIT_KEY_THRESHOLD,
            SPLIT_KEY_METHOD,
            ATTRIBUTES,
            PROTECTION_STORAGE_MASKS,
        ]
    );
    assert_eq!(fields[0].item_type(), ItemType::Enumeration);
    assert_eq!(fields[1].item_type(), ItemType::Enumeration);
    assert_eq!(fields[2].item_type(), ItemType::Integer);
    assert_eq!(fields[3].item_type(), ItemType::Integer);
    assert_eq!(fields[4].item_type(), ItemType::Enumeration);
    assert_eq!(fields[5].item_type(), ItemType::Structure);
    fields[5].with_value(|value| match value {
        ValueView::Structure(attributes) => assert!(attributes.children().is_empty()),
        other => panic!("required Attributes is a Structure, got {other:?}"),
    });
    assert_eq!(fields[6].item_type(), ItemType::Structure);
}

#[test]
fn request_always_encodes_required_empty_attributes_without_synthesizing_choices() {
    let payload = request(1)
        .into_ttlv_payload()
        .expect("XOR request omits optional Prime Field Size");
    let payload_view = payload.view();
    let fields = payload_view.children();

    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [
            OBJECT_TYPE,
            SPLIT_KEY_PARTS,
            SPLIT_KEY_THRESHOLD,
            SPLIT_KEY_METHOD,
            ATTRIBUTES,
        ]
    );
    fields[0].with_value(|value| assert!(matches!(value, ValueView::Enumeration(7))));
    fields[1].with_value(|value| assert!(matches!(value, ValueView::Integer(3))));
    fields[2].with_value(|value| assert!(matches!(value, ValueView::Integer(2))));
    fields[3].with_value(|value| assert!(matches!(value, ValueView::Enumeration(1))));
    fields[4].with_value(|value| match value {
        ValueView::Structure(attributes) => assert!(attributes.children().is_empty()),
        other => panic!("required Attributes is a Structure, got {other:?}"),
    });
}

#[test]
fn request_preserves_text_and_integer_input_identifier_forms() {
    for (identifier, expected_type) in [
        (
            UniqueIdentifier::TextString("source-key".to_owned()),
            ItemType::TextString,
        ),
        (UniqueIdentifier::Integer(-17), ItemType::Integer),
    ] {
        let payload = request(1)
            .with_unique_identifier(identifier)
            .into_ttlv_payload()
            .expect("valid input identifiers retain their exact wire forms");
        let payload_view = payload.view();
        let fields = payload_view.children();
        let unique_identifier = fields
            .iter()
            .find(|field| field.tag().raw() == UNIQUE_IDENTIFIER)
            .expect("the optional input Unique Identifier is present");

        assert_eq!(unique_identifier.item_type(), expected_type);
    }
}

#[test]
fn request_rejects_attribute_trees_that_exceed_the_ttlv_depth_limit() {
    let attributes = attributes([item(0x0042_00bf, Value::structure(structure_at_depth(63)))]);

    let error = request(1)
        .with_attributes(attributes)
        .into_ttlv_payload()
        .expect_err("the Request Payload wrapper must count toward the model depth limit");

    assert_ne!(error.to_string(), "");
}

#[test]
fn polynomial_prime_field_request_requires_the_explicit_client_input() {
    let error = request(3)
        .into_ttlv_payload()
        .expect_err("KMIPKit FR-015 requires a caller value for this method");

    assert!(!error.to_string().contains("Prime Field Size"));
    assert!(!format!("{error:?}").contains("Prime Field Size"));
}

#[test]
fn polynomial_prime_field_size_is_preserved_as_big_integer() {
    let size = vec![0x01, 0x00, 0x01];
    let request = request(3).with_prime_field_size(size.clone());
    assert_eq!(request.prime_field_size(), Some(size.as_slice()));
    let payload = request
        .into_ttlv_payload()
        .expect("explicit Prime Field Size satisfies FR-015");
    let payload_view = payload.view();
    let fields = payload_view.children();
    let prime_field_size = fields
        .iter()
        .find(|field| field.tag().raw() == PRIME_FIELD_SIZE)
        .expect("the caller-supplied Prime Field Size is encoded");

    assert_eq!(prime_field_size.item_type(), ItemType::BigInteger);
    prime_field_size.with_value(|value| match value {
        ValueView::BigInteger(encoded) => assert_eq!(encoded, size.as_slice()),
        other => panic!("Prime Field Size remains a Big Integer, got {other:?}"),
    });
}

#[test]
fn table_193_prime_field_size_remains_optional_for_other_methods_and_future_values() {
    for method in [1, 2, 0x8000_0000] {
        let payload = request(method)
            .into_ttlv_payload()
            .expect("only method 3 has the accepted FR-015 caller-input policy");
        let payload_view = payload.view();
        let fields = payload_view.children();
        assert!(
            fields
                .iter()
                .all(|field| field.tag().raw() != PRIME_FIELD_SIZE)
        );
        fields
            .iter()
            .find(|field| field.tag().raw() == SPLIT_KEY_METHOD)
            .expect("the caller's method is encoded")
            .with_value(|value| {
                assert!(matches!(value, ValueView::Enumeration(raw) if *raw == method));
            });
    }
}

#[test]
fn response_preserves_repeated_identifiers_and_wire_order() {
    let payload = structure([
        item(UNIQUE_IDENTIFIER, Value::text_string("part-1".to_owned())),
        item(UNIQUE_IDENTIFIER, Value::integer(23)),
        item(UNIQUE_IDENTIFIER, Value::enumeration(0x8000_0024)),
        item(UNIQUE_IDENTIFIER, Value::text_string("part-1".to_owned())),
    ]);
    let response = response_message(SUCCESS, None, None, Some(payload));
    let typed = decode_response(&response).expect("one or more identifiers form a success");

    assert_eq!(
        typed.unique_identifiers(),
        &[
            UniqueIdentifier::TextString("part-1".to_owned()),
            UniqueIdentifier::Integer(23),
            UniqueIdentifier::Enumeration(0x8000_0024),
            UniqueIdentifier::TextString("part-1".to_owned()),
        ]
    );
}

#[test]
fn successful_response_requires_identifiers_and_rejects_wrong_identifier_types() {
    let missing = response_message(SUCCESS, None, None, Some(Structure::new()));
    assert!(decode_response(&missing).is_err());

    let malformed = response_message(
        SUCCESS,
        None,
        None,
        Some(structure([item(
            UNIQUE_IDENTIFIER,
            Value::byte_string(b"invalid-identifier".to_vec()),
        )])),
    );
    let error = decode_response(&malformed)
        .expect_err("Unique Identifier does not accept Byte String in Table 194");
    assert!(!format!("{error:?}").contains("invalid-identifier"));
    assert!(!error.to_string().contains("invalid-identifier"));
}

#[test]
fn failure_response_preserves_every_table_195_result_reason() {
    for reason in TABLE_195_REASONS {
        let response = response_message(OPERATION_FAILED, Some(*reason), None, None);
        let typed = decode_response(&response).expect("Failure retains the shared result model");
        assert_eq!(
            typed.result().reason().map(ResultReason::raw),
            Some(*reason)
        );
        assert_eq!(typed.unique_identifiers(), []);
    }
}

#[test]
fn create_split_key_error_display_and_source_preserve_the_public_error_contract() {
    let errors = [
        CreateSplitKeyError::UnexpectedOperation,
        CreateSplitKeyError::MissingResultStatus,
        CreateSplitKeyError::InvalidOperationResult(ResultValidationError::SuccessForbidsReason),
        CreateSplitKeyError::MissingSuccessPayload,
        CreateSplitKeyError::MalformedSuccessPayload,
        CreateSplitKeyError::PolynomialMethodRequiresPrimeFieldSize,
    ];

    for error in errors {
        assert_ne!(error.to_string(), "");
        if matches!(error, CreateSplitKeyError::InvalidOperationResult(_)) {
            assert!(std::error::Error::source(&error).is_some());
        } else {
            assert!(std::error::Error::source(&error).is_none());
        }
    }
}

#[test]
fn request_debug_redacts_vendor_attribute_values() {
    let sentinel = b"KMIP_SPLIT_KEY_ATTRIBUTE_SECRET_5713";
    let vendor_attribute = item(
        0x0042_0008,
        Value::structure(structure([
            item(0x0042_009d, Value::text_string("TestVendor".to_owned())),
            item(0x0042_000a, Value::text_string("Sensitive".to_owned())),
            item(0x0042_000b, Value::byte_string(sentinel.to_vec())),
        ])),
    );
    let request = request(2).with_attributes(attributes([vendor_attribute]));

    assert!(!format!("{request:?}").contains("KMIP_SPLIT_KEY_ATTRIBUTE_SECRET_5713"));
}

#[test]
fn response_rejects_a_different_operation_without_echoing_payload_values() {
    let wrong_operation = crate::ResponseMessage::try_from_ttlv(structure([
        item(
            RESPONSE_HEADER,
            Value::structure(structure([
                item(
                    PROTOCOL_VERSION,
                    Value::structure(structure([
                        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
                        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
                    ])),
                ),
                item(TIME_STAMP, Value::date_time(1)),
                item(BATCH_COUNT, Value::integer(1)),
            ])),
        ),
        item(
            BATCH_ITEM,
            Value::structure(structure([
                item(OPERATION, Value::enumeration(0x0000_0001)),
                item(RESULT_STATUS, Value::enumeration(SUCCESS)),
                item(
                    RESPONSE_PAYLOAD,
                    Value::structure(structure([item(
                        UNIQUE_IDENTIFIER,
                        Value::text_string("WRONG_OPERATION_IDENTIFIER_SENTINEL".to_owned()),
                    )])),
                ),
            ])),
        ),
    ]))
    .expect("the wrong-operation response is a valid generic message");
    let error = decode_response(&wrong_operation)
        .expect_err("typed response conversion requires Create Split Key");

    assert!(
        !error
            .to_string()
            .contains("WRONG_OPERATION_IDENTIFIER_SENTINEL")
    );
}
