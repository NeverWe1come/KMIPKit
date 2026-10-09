//! Create payload tests derived from KMIP Specification v2.1 §6.1.8,
//! Tables 186–188, and §5.1, Table 157. The two hex fixtures are explicitly
//! Create-only derivations from the Create batch items in the pinned
//! `TC-CREATE-SD-1-21.xml`; the source test case also includes an out-of-scope
//! Get batch item, so these tests do not claim full official-case coverage.

use crate::{
    AttributeSet, CreateError, CreateRequest, CreateResponse, ObjectType, ResultReason,
    ResultStatus, ResultValidationError, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView, codec};

const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const RESPONSE_PAYLOAD: u32 = 0x0042_007c;
const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002a;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015f;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
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
const CREATE_OPERATION: u32 = 0x0000_0001;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const SENTINEL: &str = "CREATE_SECRET_SENTINEL_92847";

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

fn hex_bytes(fixture: &str) -> Vec<u8> {
    fixture
        .lines()
        .map(|line| line.split('#').next().unwrap_or_default())
        .flat_map(str::split_whitespace)
        .map(|byte| u8::from_str_radix(byte, 16).expect("fixture contains hexadecimal bytes"))
        .collect()
}

fn response_message(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    message: Option<&str>,
    payload: Option<Structure>,
) -> crate::ResponseMessage {
    crate::ResponseMessage::try_from_ttlv(response_tree(
        operation, status, reason, message, payload,
    ))
    .expect("fixture is a valid KMIP 2.1 response message")
}

fn response_tree(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    message: Option<&str>,
    payload: Option<Structure>,
) -> Structure {
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
        item(OPERATION, Value::enumeration(operation)),
        item(RESULT_STATUS, Value::enumeration(status)),
    ];
    if let Some(reason) = reason {
        fields.push(item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(message) = message {
        fields.push(item(RESULT_MESSAGE, Value::text_string(message.to_owned())));
    }
    if let Some(payload) = payload {
        fields.push(item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(structure(fields))),
    ])
}

fn decode_response(message: &crate::ResponseMessage) -> Result<CreateResponse, CreateError> {
    let response_item = message
        .batch_items()
        .next()
        .expect("fixture has one response batch item");
    CreateResponse::try_from_response_item(response_item)
}

fn success_payload(object_type: Value, unique_identifier: Value) -> Structure {
    structure([
        item(OBJECT_TYPE, object_type),
        item(UNIQUE_IDENTIFIER, unique_identifier),
    ])
}

fn response_for_payload(payload: Structure) -> crate::ResponseMessage {
    response_message(CREATE_OPERATION, SUCCESS, None, None, Some(payload))
}

fn assert_structure_field_types(structure: &Structure, expected: &[(u32, ItemType)]) {
    let view = structure.view();
    let fields = view.children();
    assert_eq!(fields.len(), expected.len());
    for (field, (expected_tag, expected_type)) in fields.iter().zip(expected) {
        assert_eq!(field.tag().raw(), *expected_tag);
        assert_eq!(field.item_type(), *expected_type);
    }
}

#[test]
fn create_only_oasis_fixtures_decode_with_the_expected_fields() {
    let request_bytes = hex_bytes(include_str!(
        "../fixtures/TC-CREATE-SD-1-21-create-only-request-payload.ttlv.hex"
    ));
    let request = codec::decode(&request_bytes).expect("derived request fixture is valid TTLV");
    assert_eq!(request.tag().raw(), REQUEST_PAYLOAD);
    request.with_value(|value| match value {
        ValueView::Structure(payload) => {
            let fields = payload.children();
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].tag().raw(), OBJECT_TYPE);
            assert!(fields[0].with_value(|value| matches!(
                value,
                ValueView::Enumeration(value) if *value == 7
            )));
            assert_eq!(fields[1].tag().raw(), ATTRIBUTES);
            fields[1].with_value(|value| match value {
                ValueView::Structure(attributes) => {
                    let items = attributes.children();
                    assert_eq!(items.len(), 1);
                    assert_eq!(items[0].tag().raw(), CRYPTOGRAPHIC_LENGTH);
                    assert!(items[0].with_value(|value| matches!(
                        value,
                        ValueView::Integer(value) if *value == 80
                    )));
                }
                _ => panic!("Attributes fixture field must be a Structure"),
            });
        }
        _ => panic!("Request Payload fixture must be a Structure"),
    });

    let attributes = AttributeSet::try_new([item(CRYPTOGRAPHIC_LENGTH, Value::integer(80))])
        .expect("Cryptographic Length is a direct §4 Object Attribute item");
    let typed_request = CreateRequest::new(ObjectType::from_raw(7), attributes)
        .into_ttlv_payload()
        .expect("typed Create request represents the derived official payload");
    let typed_view = typed_request.view();
    let typed_fields = typed_view.children();
    assert_eq!(typed_fields.len(), 2);
    assert!(typed_fields[0].with_value(|value| matches!(
        value,
        ValueView::Enumeration(value) if *value == 7
    )));
    typed_fields[1].with_value(|value| match value {
        ValueView::Structure(attributes) => {
            let items = attributes.children();
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].tag().raw(), CRYPTOGRAPHIC_LENGTH);
            assert!(items[0].with_value(|value| matches!(
                value,
                ValueView::Integer(value) if *value == 80
            )));
        }
        _ => panic!("typed Attributes must remain a Structure"),
    });

    let response_bytes = hex_bytes(include_str!(
        "../fixtures/TC-CREATE-SD-1-21-create-only-response-payload.ttlv.hex"
    ));
    let response = codec::decode(&response_bytes).expect("derived response fixture is valid TTLV");
    assert_eq!(response.tag().raw(), RESPONSE_PAYLOAD);
    response.with_value(|value| match value {
        ValueView::Structure(payload) => {
            let fields = payload.children();
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].tag().raw(), OBJECT_TYPE);
            assert_eq!(fields[1].tag().raw(), UNIQUE_IDENTIFIER);
            assert!(fields[1].with_value(|value| matches!(value, ValueView::TextString("id-001"))));
        }
        _ => panic!("Response Payload fixture must be a Structure"),
    });
}

#[test]
fn request_keeps_the_required_empty_attributes_structure_without_synthesizing_fields() {
    let request = CreateRequest::new(ObjectType::from_raw(7), AttributeSet::new());
    assert_eq!(request.object_type(), ObjectType::from_raw(7));
    assert!(request.attributes().as_items().is_empty());
    assert!(request.protection_storage_masks().is_none());
    let payload = request
        .into_ttlv_payload()
        .expect("Create payload uses assigned tags and valid model structures");
    assert_structure_field_types(
        &payload,
        &[
            (OBJECT_TYPE, ItemType::Enumeration),
            (ATTRIBUTES, ItemType::Structure),
        ],
    );
    let view = payload.view();
    let fields = view.children();
    assert!(fields[0].with_value(|value| matches!(
        value,
        ValueView::Enumeration(value) if *value == 7
    )));
    fields[1].with_value(|value| match value {
        ValueView::Structure(attributes) => assert!(attributes.children().is_empty()),
        _ => panic!("Attributes must remain a Structure"),
    });
}

#[test]
fn request_preserves_optional_protection_storage_masks_as_a_structure() {
    let masks = Structure::new();
    let request = CreateRequest::new(ObjectType::from_raw(7), AttributeSet::new())
        .with_protection_storage_masks(masks);
    assert!(request.protection_storage_masks().is_some());
    let payload = request
        .into_ttlv_payload()
        .expect("the optional generic Structure is representable");
    assert_structure_field_types(
        &payload,
        &[
            (OBJECT_TYPE, ItemType::Enumeration),
            (ATTRIBUTES, ItemType::Structure),
            (PROTECTION_STORAGE_MASKS, ItemType::Structure),
        ],
    );
    assert_eq!(
        payload.view().children()[2].with_value(|value| match value {
            ValueView::Structure(masks) => Some(masks.children().len()),
            _ => None,
        }),
        Some(0)
    );
}

#[test]
fn request_rejects_attribute_trees_that_exceed_the_ttlv_depth_limit() {
    let attributes =
        AttributeSet::try_new([item(0x0042_00bf, Value::structure(structure_at_depth(63)))])
            .expect("generic attribute values retain their nested Structure");

    let error = CreateRequest::new(ObjectType::from_raw(7), attributes)
        .into_ttlv_payload()
        .expect_err("the Request Payload wrapper must count toward the model depth limit");

    assert_ne!(error.to_string(), "");
}

#[test]
fn response_preserves_unknown_object_type_and_all_unique_identifier_wire_forms() {
    for identifier in [
        Value::text_string("id-001".to_owned()),
        Value::enumeration(0xf001_0001),
        Value::integer(-17),
    ] {
        let message =
            response_for_payload(success_payload(Value::enumeration(0xf002_0001), identifier));
        let response = decode_response(&message).expect("all Table 187 identifier types are valid");
        assert_eq!(
            response.object_type().map(ObjectType::raw),
            Some(0xf002_0001)
        );
        match (
            response.unique_identifier(),
            response.unique_identifier().unwrap(),
        ) {
            (_, UniqueIdentifier::TextString(value)) => assert_eq!(value, "id-001"),
            (_, UniqueIdentifier::Enumeration(value)) => assert_eq!(*value, 0xf001_0001),
            (_, UniqueIdentifier::Integer(value)) => assert_eq!(*value, -17),
        }
    }
}

#[test]
fn typed_response_conversion_leaves_unknown_generic_payload_fields_available() {
    let payload = structure([
        item(OBJECT_TYPE, Value::enumeration(7)),
        item(UNIQUE_IDENTIFIER, Value::text_string("id-001".to_owned())),
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(80)),
    ]);
    let message = response_for_payload(payload);
    let typed = decode_response(&message).expect("known fields remain decodable");
    assert_eq!(typed.object_type().map(ObjectType::raw), Some(7));

    let retained = message
        .batch_items()
        .next()
        .expect("fixture has one response batch item")
        .with_response_payload(|payload| {
            payload.children().iter().any(|field| {
                field.tag().raw() == CRYPTOGRAPHIC_LENGTH
                    && field.with_value(
                        |value| matches!(value, ValueView::Integer(length) if *length == 80),
                    )
            })
        })
        .expect("the validated Response Payload remains available");
    assert!(retained);
}

#[test]
fn response_rejects_a_different_operation_without_echoing_payload_values() {
    let message = response_message(
        0x0000_001f,
        SUCCESS,
        None,
        None,
        Some(success_payload(
            Value::text_string(SENTINEL.to_owned()),
            Value::text_string("id-001".to_owned()),
        )),
    );
    let error = decode_response(&message).expect_err("Get must not decode as Create");
    assert_eq!(error, CreateError::UnexpectedOperation);
    assert!(!error.to_string().contains(SENTINEL));
    assert!(!format!("{error:?}").contains(SENTINEL));
}

#[test]
fn successful_response_rejects_missing_duplicate_and_malformed_fields_redacted() {
    let malformed_payloads = [
        Structure::new(),
        structure([item(OBJECT_TYPE, Value::enumeration(7))]),
        structure([
            item(OBJECT_TYPE, Value::text_string(SENTINEL.to_owned())),
            item(UNIQUE_IDENTIFIER, Value::text_string("id-001".to_owned())),
        ]),
        structure([
            item(OBJECT_TYPE, Value::enumeration(7)),
            item(OBJECT_TYPE, Value::enumeration(8)),
            item(UNIQUE_IDENTIFIER, Value::text_string("id-001".to_owned())),
        ]),
        structure([item(OBJECT_TYPE, Value::enumeration(7))]),
        structure([
            item(OBJECT_TYPE, Value::enumeration(7)),
            item(UNIQUE_IDENTIFIER, Value::boolean(true)),
        ]),
        structure([
            item(OBJECT_TYPE, Value::enumeration(7)),
            item(UNIQUE_IDENTIFIER, Value::text_string("id-001".to_owned())),
            item(UNIQUE_IDENTIFIER, Value::integer(2)),
        ]),
    ];

    for payload in malformed_payloads {
        let error = decode_response(&response_for_payload(payload))
            .expect_err("malformed success payload must be rejected");
        assert!(!error.to_string().contains(SENTINEL));
        assert!(!format!("{error:?}").contains(SENTINEL));
    }
}

#[test]
fn successful_response_requires_a_payload() {
    let error = crate::ResponseMessage::try_from_ttlv(response_tree(
        CREATE_OPERATION,
        SUCCESS,
        None,
        None,
        None,
    ))
    .expect_err("the shared message layer rejects success without Response Payload");
    assert_eq!(
        error.kind(),
        crate::MessageValidationErrorKind::InvalidResult
    );
}

#[test]
fn operation_failed_preserves_each_create_result_reason_without_a_success_payload() {
    let create_reasons = [
        "Attribute Read Only",
        "Attribute Single Valued",
        "Cryptographic Failure",
        "Invalid Attribute",
        "Invalid Attribute Value",
        "Invalid Object Type",
        "Non Unique Name Attribute",
        "Read Only Attribute",
        "Server Limit Exceeded",
        "Unsupported Attribute",
        "Attestation Failed",
        "Attestation Required",
        "Feature Not Supported",
        "Invalid Field",
        "Invalid Message",
        "Operation Not Supported",
        "Permission Denied",
        "Protection Storage Unavailable",
        "Response Too Large",
    ];

    for expected_name in create_reasons {
        let reason = ResultReason::known_values()
            .iter()
            .find_map(|(reason, name)| (*name == expected_name).then_some(*reason))
            .expect("Table 188 reason is represented in the pinned result catalog");
        let message = response_message(
            CREATE_OPERATION,
            OPERATION_FAILED,
            Some(reason.raw()),
            None,
            None,
        );
        let response = decode_response(&message).expect("a Failure with its reason is valid");
        assert_eq!(
            response.result().status(),
            ResultStatus::from_raw(OPERATION_FAILED)
        );
        assert_eq!(response.result().reason(), Some(reason));
        assert!(response.object_type().is_none());
        assert!(response.unique_identifier().is_none());
    }
}

#[test]
fn non_create_result_shapes_retain_the_existing_result_validation_contract() {
    let missing_reason = crate::ResponseMessage::try_from_ttlv(response_tree(
        CREATE_OPERATION,
        OPERATION_FAILED,
        None,
        None,
        None,
    ));
    assert_eq!(
        missing_reason.unwrap_err().kind(),
        crate::MessageValidationErrorKind::InvalidResult
    );

    let success_with_reason = crate::ResponseMessage::try_from_ttlv(response_tree(
        CREATE_OPERATION,
        SUCCESS,
        Some(0xf001_0001),
        None,
        Some(success_payload(
            Value::enumeration(7),
            Value::text_string("id-001".to_owned()),
        )),
    ));
    assert_eq!(
        success_with_reason.unwrap_err().kind(),
        crate::MessageValidationErrorKind::InvalidResult
    );
}

#[test]
fn create_error_display_and_source_preserve_the_public_error_contract() {
    let errors = [
        CreateError::UnexpectedOperation,
        CreateError::MissingResultStatus,
        CreateError::InvalidOperationResult(ResultValidationError::SuccessForbidsReason),
        CreateError::MissingSuccessPayload,
        CreateError::MalformedSuccessPayload,
    ];

    for error in errors {
        assert_ne!(error.to_string(), "");
        if matches!(error, CreateError::InvalidOperationResult(_)) {
            assert!(std::error::Error::source(&error).is_some());
        } else {
            assert!(std::error::Error::source(&error).is_none());
        }
    }
}
