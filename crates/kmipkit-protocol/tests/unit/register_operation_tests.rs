//! Register tests from OASIS KMIP v2.1 §6.1.43, Tables 291–294.
//! Traceability covers `KMIPKIT-REQ-SPEC-6.1.43-001`, `-005`, `-007`,
//! `-009-001`, `-009-002`, `-010-001`, `-010-002`, and `-011`.
//! These vectors are derived from the pinned tables, not official fixture runs.

use crate::async_operation_fixtures::{response_message, structure};
use crate::{
    AttributeSet, ObjectType, RegisterError, RegisterRequest, RegisterResponse,
    ResponseBatchItemView, ResponseMessage, ResultReason, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

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
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0044;
const NIST_KEY_TYPE: u32 = 0x0042_013A;
const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
const RANDOM_NUMBER_GENERATOR: u32 = 0x0042_00DE;
const RNG_PARAMETERS: u32 = 0x0042_00D9;
const RNG_ALGORITHM: u32 = 0x0042_00DA;
const CERTIFICATE_LENGTH: u32 = 0x0042_00AD;
const DIGITAL_SIGNATURE_ALGORITHM: u32 = 0x0042_00AE;
const KEY_BLOCK: u32 = 0x0042_0040;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
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
    item(SYMMETRIC_KEY, Value::structure(structure([key_block])))
}

fn key_object_with_key_value_attributes(algorithm: bool, length: bool) -> Item {
    let mut attributes = Vec::new();
    if algorithm {
        attributes.push(item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)));
    }
    if length {
        attributes.push(item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)));
    }

    let key_block = item(
        KEY_BLOCK,
        Value::structure(structure([
            item(KEY_FORMAT_TYPE, Value::enumeration(1)),
            item(
                KEY_VALUE,
                Value::structure(structure([
                    item(
                        KEY_MATERIAL,
                        Value::byte_string(vec![0x01, 0x02, 0x03, 0x04]),
                    ),
                    item(ATTRIBUTES, Value::structure(structure(attributes))),
                ])),
            ),
        ])),
    );
    item(SYMMETRIC_KEY, Value::structure(structure([key_block])))
}

fn valid_raw_symmetric_key_object() -> Item {
    let key_value = item(
        KEY_VALUE,
        Value::structure(structure([item(
            KEY_MATERIAL,
            Value::byte_string(vec![0x01, 0x02, 0x03, 0x04]),
        )])),
    );
    let key_block = item(
        KEY_BLOCK,
        Value::structure(structure([
            item(KEY_FORMAT_TYPE, Value::enumeration(1)),
            key_value,
        ])),
    );
    item(SYMMETRIC_KEY, Value::structure(structure([key_block])))
}

fn deeply_nested_structure() -> Structure {
    let mut nested = Structure::new();
    for _ in 1..64 {
        let mut parent = Structure::new();
        parent
            .try_push(item(UNKNOWN_VENDOR_TAG, Value::structure(nested)))
            .expect("each level remains within the model depth bound");
        nested = parent;
    }
    nested
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
        [
            OBJECT_TYPE,
            ATTRIBUTES,
            SYMMETRIC_KEY,
            PROTECTION_STORAGE_MASKS
        ]
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
    assert!(
        result.is_err(),
        "Table 293 requires key algorithm and length"
    );

    let encoded_in_key_block = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([]),
        key_object(true, true),
    )
    .into_ttlv_payload();
    assert!(encoded_in_key_block.is_ok());
}

#[test]
fn request_allows_algorithm_and_length_embedded_in_key_value() {
    // KMIP Specification v2.1 §3.1, Tables 11–12 and §3.2 permit omitting
    // Cryptographic Algorithm and Cryptographic Length when they are available
    // in the Key Value Structure.
    let embedded_pair = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([]),
        key_object_with_key_value_attributes(true, true),
    )
    .into_ttlv_payload();
    assert!(embedded_pair.is_ok());

    let algorithm_in_key_value = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))]),
        key_object_with_key_value_attributes(true, false),
    )
    .into_ttlv_payload();
    assert!(algorithm_in_key_value.is_ok());

    let length_in_key_value = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3))]),
        key_object_with_key_value_attributes(false, true),
    )
    .into_ttlv_payload();
    assert!(length_in_key_value.is_ok());

    for (algorithm, length) in [(true, false), (false, true)] {
        let incomplete = RegisterRequest::new(
            ObjectType::from_raw(2),
            symmetric_attributes([]),
            key_object_with_key_value_attributes(algorithm, length),
        )
        .into_ttlv_payload();
        assert!(
            incomplete.is_err(),
            "Algorithm and Length remain a required pair when omitted from Attributes"
        );
    }
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
fn request_encodes_nist_symmetric_data_encryption_key_type() {
    // OASIS KMIP v2.1 §4.34, Tables 97–98, §§11.32 and 11.56 permit NIST Key
    // Type on §6.1.43, Table 291, Register; Table 293 requires the algorithm/
    // length pair included here for a symmetric key. §3.1, Table 12, defines
    // the Raw Key Block used by this request.
    let request = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([
            item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
            item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
            item(NIST_KEY_TYPE, Value::enumeration(6)),
        ]),
        valid_raw_symmetric_key_object(),
    );
    let payload = request
        .into_ttlv_payload()
        .expect("Register includes the Table 293 algorithm/length prerequisites");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(fields[1].tag().raw(), ATTRIBUTES);
    let nist_key_type = fields[1].with_value(|value| match value {
        ValueView::Structure(attributes) => attributes
            .children()
            .iter()
            .find(|attribute| attribute.tag().raw() == NIST_KEY_TYPE)
            .and_then(|attribute| {
                attribute.with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                })
            }),
        _ => None,
    });
    assert_eq!(nist_key_type, Some(6));
}

#[test]
fn request_can_omit_original_creation_date_when_registering() {
    // OASIS KMIP v2.1 §4.38, Table 105, and §11.56 make Original Creation
    // Date optional for an object registered by §6.1.43, Table 291. Table 293
    // prerequisites and the §3.1, Table 12, Raw Key Block are included.
    let request = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([
            item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
            item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        ]),
        valid_raw_symmetric_key_object(),
    );
    let payload = request
        .into_ttlv_payload()
        .expect("Register can omit Original Creation Date");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(fields[1].tag().raw(), ATTRIBUTES);
    let attribute_tags = fields[1].with_value(|value| match value {
        ValueView::Structure(attributes) => Some(
            attributes
                .children()
                .iter()
                .map(|attribute| attribute.tag().raw())
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert!(
        !attribute_tags
            .expect("Register Attributes remain a Structure")
            .contains(&ORIGINAL_CREATION_DATE)
    );
}

#[test]
fn request_encodes_client_supplied_original_creation_date_on_register() {
    // OASIS KMIP v2.1 §4.38, Table 105, and §11.56 permit the client to set
    // this Date-Time in §6.1.43, Table 291, Register; Table 293 prerequisites
    // and the §3.1, Table 12, Raw Key Block are included. This tests request
    // serialization only.
    let request = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([
            item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
            item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
            item(ORIGINAL_CREATION_DATE, Value::date_time(1_700_000_000)),
        ]),
        valid_raw_symmetric_key_object(),
    );
    let payload = request
        .into_ttlv_payload()
        .expect("Register can carry a caller-supplied Original Creation Date");
    let payload_view = payload.view();
    let fields = payload_view.children();
    let original_creation_date = fields[1].with_value(|value| match value {
        ValueView::Structure(attributes) => attributes
            .children()
            .iter()
            .find(|attribute| attribute.tag().raw() == ORIGINAL_CREATION_DATE)
            .map(|attribute| {
                attribute.with_value(
                    |value| matches!(value, ValueView::DateTime(value) if *value == 1_700_000_000),
                )
            }),
        _ => None,
    });
    assert_eq!(original_creation_date, Some(true));
}

#[test]
fn request_encodes_rng_algorithm_directly_on_register() {
    // OASIS KMIP v2.1 §4.46, Table 121, §§7.36, 11.49, and 11.56 permit a
    // client Random Number Generator on §6.1.43, Table 291, Register. Its
    // encoding uses RNG Parameters fields directly, without a nested member.
    // Table 293 prerequisites and the §3.1, Table 12, Raw Key Block are included.
    let request = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([
            item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
            item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
            item(
                RANDOM_NUMBER_GENERATOR,
                Value::structure(structure([item(RNG_ALGORITHM, Value::enumeration(1))])),
            ),
        ]),
        valid_raw_symmetric_key_object(),
    );
    let payload = request
        .into_ttlv_payload()
        .expect("Register includes the Table 293 algorithm/length prerequisites");
    let payload_view = payload.view();
    let fields = payload_view.children();
    let rng_members = fields[1].with_value(|value| match value {
        ValueView::Structure(attributes) => attributes
            .children()
            .iter()
            .find(|attribute| attribute.tag().raw() == RANDOM_NUMBER_GENERATOR)
            .and_then(|attribute| {
                attribute.with_value(|value| match value {
                    ValueView::Structure(rng) => Some(
                        rng.children()
                            .iter()
                            .map(|member| member.tag().raw())
                            .collect::<Vec<_>>(),
                    ),
                    _ => None,
                })
            }),
        _ => None,
    });
    assert_eq!(rng_members, Some(vec![RNG_ALGORITHM]));
    assert!(
        !rng_members
            .expect("Random Number Generator remains a Structure")
            .contains(&RNG_PARAMETERS)
    );
    let rng_algorithm = fields[1].with_value(|value| match value {
        ValueView::Structure(attributes) => attributes
            .children()
            .iter()
            .find(|attribute| attribute.tag().raw() == RANDOM_NUMBER_GENERATOR)
            .and_then(|attribute| {
                attribute.with_value(|value| match value {
                    ValueView::Structure(rng) => rng.children().first().and_then(|member| {
                        member.with_value(|value| match value {
                            ValueView::Enumeration(value) => Some(*value),
                            _ => None,
                        })
                    }),
                    _ => None,
                })
            }),
        _ => None,
    });
    assert_eq!(rng_algorithm, Some(1));
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
    assert!(
        result.is_err(),
        "Table 293 requires Cryptographic Usage Mask"
    );
}

#[test]
fn request_requires_certificate_length_and_allows_embedded_signature_algorithm() {
    let certificate = item(CERTIFICATE, Value::structure(structure([])));
    let missing_length =
        RegisterRequest::new(ObjectType::from_raw(1), attrs([]), certificate).into_ttlv_payload();
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
fn request_accessors_preserve_type_attributes_and_object_and_unknown_types_remain_forward_compatible()
 {
    let attributes = symmetric_attributes([
        item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
    ]);
    let object = key_object(false, false);
    let request = RegisterRequest::new(ObjectType::from_raw(2), attributes, object);

    assert_eq!(request.object_type().raw(), 2);
    assert_eq!(request.attributes().as_items().len(), 3);
    assert_eq!(request.object().tag().raw(), SYMMETRIC_KEY);

    let future_object_type = RegisterRequest::new(
        ObjectType::from_raw(0x8000_0001),
        attrs([]),
        item(UNKNOWN_VENDOR_TAG, Value::structure(structure([]))),
    );
    assert!(future_object_type.into_ttlv_payload().is_ok());
}

#[test]
fn request_rejects_malformed_key_block_shape_and_objects_beyond_model_depth() {
    let malformed_key_block = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([]),
        item(
            SYMMETRIC_KEY,
            Value::structure(structure([item(KEY_BLOCK, Value::integer(1))])),
        ),
    )
    .into_ttlv_payload();
    assert!(malformed_key_block.is_err());

    let excessive_depth = RegisterRequest::new(
        ObjectType::from_raw(2),
        symmetric_attributes([
            item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
            item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        ]),
        item(SYMMETRIC_KEY, Value::structure(deeply_nested_structure())),
    )
    .into_ttlv_payload();
    assert!(excessive_depth.is_err());
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
        Some(&UniqueIdentifier::TextString(
            "registered-object".to_owned()
        ))
    );
    assert!(response.with_ttlv(|view| {
        let payload = view
            .children()
            .iter()
            .find(|field| field.tag().raw() == RESPONSE_PAYLOAD)
            .expect("Register response includes a Response Payload");
        payload.with_value(|value| match value {
            ValueView::Structure(payload) => payload
                .children()
                .iter()
                .any(|field| field.tag().raw() == UNKNOWN_VENDOR_TAG),
            _ => false,
        })
    }));
}

#[test]
fn successful_response_rejects_missing_duplicate_or_mistyped_identifier() {
    let payloads = [
        structure([]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            item(UNIQUE_IDENTIFIER, Value::text_string("second".to_owned())),
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
