//! Create Key Pair payload tests for KMIP Specification v2.1 §6.1.9,
//! Tables 189–192, and attribute groups in §§5.2–5.4, Tables 158–160.

use crate::{
    AttributeSet, CreateKeyPairRequest, CreateKeyPairResponse, ResultReason, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const CREATE_KEY_PAIR_OPERATION: u32 = 0x0000_0002;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;

const COMMON_ATTRIBUTES: u32 = 0x0042_0126;
const PRIVATE_KEY_ATTRIBUTES: u32 = 0x0042_0127;
const PUBLIC_KEY_ATTRIBUTES: u32 = 0x0042_0128;
const COMMON_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0163;
const PRIVATE_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0164;
const PUBLIC_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0165;

const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_DOMAIN_PARAMETERS: u32 = 0x0042_0029;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002a;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002b;

const PRIVATE_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_0066;
const PUBLIC_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_006f;
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

const TABLE_192_REASONS: &[&str] = &[
    "Attribute Read Only",
    "Attribute Single Valued",
    "Cryptographic Failure",
    "Invalid Attribute",
    "Invalid Attribute Value",
    "Non Unique Name Attribute",
    "Server Limit Exceeded",
    "Attestation Failed",
    "Attestation Required",
    "Feature Not Supported",
    "Invalid Field",
    "Invalid Message",
    "Operation Not Supported",
    "Permission Denied",
    "Private Protection Storage Unavailable",
    "Public Protection Storage Unavailable",
    "Response Too Large",
];

const TABLE_191_ATTRIBUTES: &[u32] = &[
    CRYPTOGRAPHIC_ALGORITHM,
    CRYPTOGRAPHIC_LENGTH,
    CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
    CRYPTOGRAPHIC_PARAMETERS,
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

fn attributes(items: impl IntoIterator<Item = Item>) -> AttributeSet {
    AttributeSet::try_new(items).expect("fixture contains valid direct Object Attribute items")
}

fn request_with_groups(
    common: Option<AttributeSet>,
    private: Option<AttributeSet>,
    public: Option<AttributeSet>,
) -> CreateKeyPairRequest {
    let mut request = CreateKeyPairRequest::new();
    if let Some(attributes) = common {
        request = request.with_common_attributes(attributes);
    }
    if let Some(attributes) = private {
        request = request.with_private_key_attributes(attributes);
    }
    if let Some(attributes) = public {
        request = request.with_public_key_attributes(attributes);
    }
    request
}

fn table_191_value(raw_tag: u32, generation: u32) -> Value {
    match raw_tag {
        CRYPTOGRAPHIC_ALGORITHM => Value::enumeration(generation + 1),
        CRYPTOGRAPHIC_LENGTH => Value::integer(
            i32::try_from((generation + 1) * 1024).expect("fixture length fits KMIP Integer"),
        ),
        CRYPTOGRAPHIC_DOMAIN_PARAMETERS | CRYPTOGRAPHIC_PARAMETERS => {
            Value::structure(structure([item(
                CRYPTOGRAPHIC_LENGTH,
                Value::integer(
                    i32::try_from((generation + 1) * 1024)
                        .expect("fixture nested length fits KMIP Integer"),
                ),
            )]))
        }
        _ => panic!("fixture tag is one of the Table 191 attributes"),
    }
}

fn table_191_attributes(generation: u32) -> AttributeSet {
    attributes(
        TABLE_191_ATTRIBUTES
            .iter()
            .map(|raw_tag| item(*raw_tag, table_191_value(*raw_tag, generation))),
    )
}

fn table_191_single_attribute(raw_tag: u32, generation: u32) -> AttributeSet {
    attributes([item(raw_tag, table_191_value(raw_tag, generation))])
}

fn vendor_attribute(value: &[u8]) -> Item {
    item(
        0x0042_0008,
        Value::structure(structure([
            item(0x0042_009d, Value::text_string("TestVendor".to_owned())),
            item(
                0x0042_000a,
                Value::text_string("FixtureAttribute".to_owned()),
            ),
            item(0x0042_000b, Value::byte_string(value.to_vec())),
        ])),
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
        item(OPERATION, Value::enumeration(CREATE_KEY_PAIR_OPERATION)),
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

fn response_payload(private_id: Value, public_id: Value) -> Structure {
    structure([
        item(PRIVATE_KEY_UNIQUE_IDENTIFIER, private_id),
        item(PUBLIC_KEY_UNIQUE_IDENTIFIER, public_id),
    ])
}

fn decode_response(
    response: &crate::ResponseMessage,
) -> Result<CreateKeyPairResponse, crate::CreateKeyPairError> {
    let item = response
        .batch_items()
        .next()
        .expect("fixture has one response batch item");
    CreateKeyPairResponse::try_from_response_item(item)
}

#[test]
fn request_preserves_six_distinct_optional_groups_and_repeated_items_in_wire_order() {
    let repeated_common = attributes([
        vendor_attribute(b"common-first"),
        vendor_attribute(b"common-second"),
    ]);
    let private = attributes([item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3))]);
    let public = attributes([item(CRYPTOGRAPHIC_LENGTH, Value::integer(2048))]);
    let request = request_with_groups(Some(repeated_common), Some(private), Some(public))
        .with_common_protection_storage_masks(Structure::new())
        .with_private_protection_storage_masks(Structure::new())
        .with_public_protection_storage_masks(Structure::new());

    let payload = request
        .into_ttlv_payload()
        .expect("all six Table 189 groups are representable");
    let fields = payload.view().children();
    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [
            COMMON_ATTRIBUTES,
            PRIVATE_KEY_ATTRIBUTES,
            PUBLIC_KEY_ATTRIBUTES,
            COMMON_PROTECTION_STORAGE_MASKS,
            PRIVATE_PROTECTION_STORAGE_MASKS,
            PUBLIC_PROTECTION_STORAGE_MASKS,
        ]
    );
    assert!(
        fields[..3]
            .iter()
            .all(|field| field.item_type() == ItemType::Structure)
    );
    fields[0].with_value(|value| match value {
        ValueView::Structure(group) => {
            assert_eq!(group.children().len(), 2);
            assert!(
                group
                    .children()
                    .iter()
                    .all(|attribute| attribute.tag().raw() == 0x0042_0008)
            );
        }
        _ => panic!("Common Attributes remains a Structure"),
    });
    assert!(
        fields[3..]
            .iter()
            .all(|field| field.item_type() == ItemType::Structure)
    );
}

#[test]
fn request_distinguishes_absent_groups_from_present_empty_groups_without_synthesizing_choices() {
    let absent = CreateKeyPairRequest::new()
        .into_ttlv_payload()
        .expect("all optional Table 189 fields may be absent");
    assert!(absent.view().children().is_empty());

    let present_empty = request_with_groups(
        Some(AttributeSet::new()),
        Some(AttributeSet::new()),
        Some(AttributeSet::new()),
    )
    .into_ttlv_payload()
    .expect("optional attribute groups may be present and empty");
    let fields = present_empty.view().children();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].tag().raw(), COMMON_ATTRIBUTES);
    assert_eq!(fields[1].tag().raw(), PRIVATE_KEY_ATTRIBUTES);
    assert_eq!(fields[2].tag().raw(), PUBLIC_KEY_ATTRIBUTES);
    for field in fields {
        field.with_value(|value| match value {
            ValueView::Structure(group) => assert!(group.children().is_empty()),
            _ => panic!("present-empty optional groups remain Structures"),
        });
    }
    assert!(
        !fields
            .iter()
            .any(|field| { TABLE_191_ATTRIBUTES.contains(&field.tag().raw()) })
    );
}

#[test]
fn table_191_accepts_absent_values_and_common_attribute_fallback_without_copying() {
    let absent = CreateKeyPairRequest::new()
        .into_ttlv_payload()
        .expect("unspecified Table 191 values are not synthesized");
    assert!(absent.view().children().is_empty());

    let common = table_191_attributes(0);
    let payload = request_with_groups(Some(common), None, None)
        .into_ttlv_payload()
        .expect("both keys inherit the same supplied Common Attributes values");
    let fields = payload.view().children();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), COMMON_ATTRIBUTES);
    fields[0].with_value(|value| match value {
        ValueView::Structure(group) => {
            assert_eq!(group.children().len(), TABLE_191_ATTRIBUTES.len());
            for (attribute, expected_tag) in
                group.children().iter().zip(TABLE_191_ATTRIBUTES.iter())
            {
                assert_eq!(attribute.tag().raw(), *expected_tag);
            }
        }
        _ => panic!("Common Attributes remains a Structure"),
    });
}

#[test]
fn table_191_accepts_equal_key_specific_overrides_that_differ_from_common_values() {
    let payload = request_with_groups(
        Some(table_191_attributes(0)),
        Some(table_191_attributes(1)),
        Some(table_191_attributes(1)),
    )
    .into_ttlv_payload()
    .expect("equal private/public overrides take precedence over Common Attributes");

    let fields = payload.view().children();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].tag().raw(), COMMON_ATTRIBUTES);
    assert_eq!(fields[1].tag().raw(), PRIVATE_KEY_ATTRIBUTES);
    assert_eq!(fields[2].tag().raw(), PUBLIC_KEY_ATTRIBUTES);
    for field in fields {
        field.with_value(|value| match value {
            ValueView::Structure(group) => assert_eq!(group.children().len(), 4),
            _ => panic!("attribute groups remain Structures"),
        });
    }
}

#[test]
fn table_191_rejects_one_sided_effective_values_and_conflicts_for_each_attribute() {
    for attribute_tag in TABLE_191_ATTRIBUTES {
        let one_sided = request_with_groups(
            None,
            Some(table_191_single_attribute(*attribute_tag, 0)),
            None,
        )
        .into_ttlv_payload();
        assert!(
            one_sided.is_err(),
            "one-sided Table 191 tag {attribute_tag:#x}"
        );

        let differing_overrides = request_with_groups(
            None,
            Some(table_191_single_attribute(*attribute_tag, 0)),
            Some(table_191_single_attribute(*attribute_tag, 1)),
        )
        .into_ttlv_payload();
        assert!(
            differing_overrides.is_err(),
            "conflicting Table 191 tag {attribute_tag:#x}"
        );

        let private_override_over_common = request_with_groups(
            Some(table_191_single_attribute(*attribute_tag, 0)),
            Some(table_191_single_attribute(*attribute_tag, 1)),
            None,
        )
        .into_ttlv_payload();
        assert!(
            private_override_over_common.is_err(),
            "private override conflicts with public Common fallback for {attribute_tag:#x}"
        );
    }
}

#[test]
fn attribute_group_debug_redacts_a_vendor_attribute_value_sentinel() {
    let sentinel = b"KMIP_KEY_PAIR_ATTRIBUTE_SECRET_SENTINEL_5792";
    let request = request_with_groups(Some(attributes([vendor_attribute(sentinel)])), None, None);

    assert!(!format!("{request:?}").contains("KMIP_KEY_PAIR_ATTRIBUTE_SECRET_SENTINEL"));
}

#[test]
fn successful_response_preserves_private_and_public_identifier_meaning() {
    let response = response_message(
        SUCCESS,
        None,
        None,
        Some(response_payload(
            Value::text_string("private-key-id".to_owned()),
            Value::text_string("public-key-id".to_owned()),
        )),
    );
    let typed = decode_response(&response).expect("Table 190 requires both identifiers");

    assert_eq!(typed.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.private_key_unique_identifier(),
        Some(&UniqueIdentifier::TextString("private-key-id".to_owned()))
    );
    assert_eq!(
        typed.public_key_unique_identifier(),
        Some(&UniqueIdentifier::TextString("public-key-id".to_owned()))
    );
}

#[test]
fn malformed_success_response_rejects_missing_duplicate_and_wrong_type_identifiers() {
    let missing_public = response_message(
        SUCCESS,
        None,
        None,
        Some(structure([item(
            PRIVATE_KEY_UNIQUE_IDENTIFIER,
            Value::text_string("private-key-id".to_owned()),
        )])),
    );
    assert!(decode_response(&missing_public).is_err());

    let duplicate_private = response_message(
        SUCCESS,
        None,
        None,
        Some(structure([
            item(
                PRIVATE_KEY_UNIQUE_IDENTIFIER,
                Value::text_string("private-one".to_owned()),
            ),
            item(
                PRIVATE_KEY_UNIQUE_IDENTIFIER,
                Value::text_string("private-two".to_owned()),
            ),
            item(
                PUBLIC_KEY_UNIQUE_IDENTIFIER,
                Value::text_string("public-key-id".to_owned()),
            ),
        ])),
    );
    assert!(decode_response(&duplicate_private).is_err());

    let wrong_type = response_message(
        SUCCESS,
        None,
        None,
        Some(response_payload(
            Value::integer(17),
            Value::text_string("public-key-id".to_owned()),
        )),
    );
    assert!(decode_response(&wrong_type).is_err());
}

#[test]
fn failure_response_preserves_every_table_192_result_reason_without_success_payload() {
    let known_reasons = ResultReason::known_values();
    let mut preserved = Vec::new();
    for expected_name in TABLE_192_REASONS {
        let (reason, actual_name) = known_reasons
            .iter()
            .find(|(_, name)| name == expected_name)
            .unwrap_or_else(|| panic!("Table 192 reason {expected_name} is in the shared catalog"));
        let response = response_message(
            OPERATION_FAILED,
            Some(reason.raw()),
            Some("redacted result message"),
            None,
        );
        let typed = decode_response(&response).expect("Failure keeps a shared typed result");
        assert_eq!(
            typed.result().reason().map(ResultReason::raw),
            Some(reason.raw())
        );
        assert_eq!(*actual_name, *expected_name);
        assert!(typed.private_key_unique_identifier().is_none());
        assert!(typed.public_key_unique_identifier().is_none());
        preserved.push(*actual_name);
    }
    assert_eq!(preserved, TABLE_192_REASONS);
}
