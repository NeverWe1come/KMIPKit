//! OASIS KMIP v2.1 §§7.39, 9.4, 9.11; Tables 403, 410–412, 414, 416, and 442.
//!
//! These derived contract cases validate in-memory Authentication and
//! Credential models. They do not claim official OASIS test-vector coverage
//! or server-side credential satisfaction.
//!
//! Traceability: KMIPKIT-REQ-SPEC-9.4-001-001, KMIPKIT-REQ-SPEC-9.4-001-002,
//! KMIPKIT-REQ-SPEC-9.4-002, KMIPKIT-REQ-SPEC-9.11-004-001/-002/-003;
//! KMIPKIT-0008-FR-001, FR-002, FR-004, FR-005, FR-009; SC-001, SC-002.

use kmipkit_protocol::{Authentication, CredentialType, CredentialValue, RequestMessage};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const CREDENTIAL: u32 = 0x0042_0023;
const CREDENTIAL_TYPE: u32 = 0x0042_0024;
const CREDENTIAL_VALUE: u32 = 0x0042_0025;
const EXTENSION_CHILD: u32 = 0x0054_0003;
const USERNAME: u32 = 0x0042_0099;
const PASSWORD: u32 = 0x0042_00A1;
const DEVICE_IDENTIFIER: u32 = 0x0042_00A2;
const MACHINE_IDENTIFIER: u32 = 0x0042_00A9;
const MEDIA_IDENTIFIER: u32 = 0x0042_00AA;
const NETWORK_IDENTIFIER: u32 = 0x0042_00AB;
const DEVICE_SERIAL_NUMBER: u32 = 0x0042_00B0;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const TIME_STAMP: u32 = 0x0042_0092;
const TICKET: u32 = 0x0042_0149;
const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;
const ONE_TIME_PASSWORD: u32 = 0x0042_0156;
const HASHED_PASSWORD: u32 = 0x0042_0157;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_HEADER: u32 = 0x0042_0077;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("the fixture tag is allocated by KMIP 2.1")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("the checked tag and value form an Item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("the fixture stays within the model depth limit");
    }
    structure
}

fn credential(raw_type: u32, opaque_value: u32) -> Structure {
    structure([
        item(CREDENTIAL_TYPE, Value::enumeration(raw_type)),
        item(
            CREDENTIAL_VALUE,
            Value::structure(structure([item(opaque_value, Value::boolean(true))])),
        ),
    ])
}

fn credential_value(
    raw_type: u32,
    value: Structure,
) -> Result<CredentialValue, kmipkit_protocol::CredentialValidationError> {
    CredentialValue::try_from_ttlv(CredentialType::from_raw(raw_type), value)
}

fn value_tags(value: CredentialValue) -> Vec<u32> {
    let tree: Structure = value.into_ttlv();
    let view = tree.view();
    view.children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

fn request_without_authentication() -> RequestMessage {
    let version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(version)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch_item = structure([
        item(OPERATION, Value::enumeration(1)),
        item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
    ]);
    RequestMessage::try_from_ttlv(structure([
        item(REQUEST_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(batch_item)),
    ]))
    .expect("the request fixture has a valid single-item KMIP envelope")
}

#[test]
fn authentication_absence_is_distinct_from_present_value() {
    let request = request_without_authentication();

    assert!(request.header().with_authentication(|_| ()).is_none());
}

#[test]
fn authentication_rejects_present_empty_credentials() {
    let result = Authentication::try_from_ttlv(Structure::new());

    assert!(result.is_err());
}

#[test]
fn authentication_constructor_rejects_empty_list() {
    assert!(Authentication::new(Vec::new()).is_err());
}

#[test]
fn authentication_preserves_nonempty_credential_order() {
    let authentication = structure([
        item(
            CREDENTIAL,
            Value::structure(credential(0xF123_4567, 0x0054_0001)),
        ),
        item(
            CREDENTIAL,
            Value::structure(credential(0xE234_5678, 0x0054_0002)),
        ),
    ]);

    let parsed = Authentication::try_from_ttlv(authentication)
        .expect("repeated Credential structures remain in source order");
    let raw_types: Vec<_> = parsed
        .credentials()
        .map(|entry| entry.credential_type_raw())
        .collect();

    assert_eq!(raw_types, [0xF123_4567, 0xE234_5678]);
}

fn assert_authentication_tree_preserves_unknown_fields_and_order(roundtrip: &Structure) {
    let view = roundtrip.view();
    let fields: &[Item] = view.children();
    let tags: Vec<u32> = fields.iter().map(|field| field.tag().raw()).collect();
    let extension_values: Vec<Vec<u8>> = fields
        .iter()
        .filter(|field| field.tag().raw() == EXTENSION_CHILD)
        .map(|field| {
            field.with_value(|value| match value {
                ValueView::ByteString(bytes) => bytes.to_vec(),
                _ => Vec::new(),
            })
        })
        .collect();

    assert_eq!(
        tags,
        [EXTENSION_CHILD, CREDENTIAL, CREDENTIAL, EXTENSION_CHILD]
    );
    assert_eq!(extension_values, [vec![0x80, 0x01], vec![0xFE, 0x02]]);
}

#[test]
fn authentication_roundtrip_preserves_unknown_fields_and_credential_order() {
    let source = structure([
        item(EXTENSION_CHILD, Value::byte_string(vec![0x80, 0x01])),
        item(
            CREDENTIAL,
            Value::structure(credential(0xF123_4567, 0x0054_0001)),
        ),
        item(
            CREDENTIAL,
            Value::structure(credential(0xE234_5678, 0x0054_0002)),
        ),
        item(EXTENSION_CHILD, Value::byte_string(vec![0xFE, 0x02])),
    ]);
    let parsed = Authentication::try_from_ttlv(source)
        .expect("repeated Credentials and unknown Authentication fields are retained");

    let roundtrip = parsed.into_ttlv();
    assert_authentication_tree_preserves_unknown_fields_and_order(&roundtrip);
}

#[test]
fn authentication_does_not_claim_server_satisfaction() {
    let authentication = structure([
        item(
            CREDENTIAL,
            Value::structure(credential(0xF123_4567, 0x0054_0001)),
        ),
        item(
            CREDENTIAL,
            Value::structure(credential(0xE234_5678, 0x0054_0002)),
        ),
    ]);

    let parsed = Authentication::try_from_ttlv(authentication)
        .expect("the client model must not decide server-side credential satisfaction");

    assert_eq!(parsed.credentials().len(), 2);
}

#[test]
fn username_and_password_requires_username_and_preserves_optional_password_presence() {
    let minimum = credential_value(
        1,
        structure([item(USERNAME, Value::text_string("alice".to_owned()))]),
    )
    .expect("Username is required while Password is optional");
    assert!(matches!(minimum, CredentialValue::UsernameAndPassword(_)));
    assert_eq!(value_tags(minimum), [USERNAME]);

    let present_empty_password = credential_value(
        1,
        structure([
            item(USERNAME, Value::text_string("alice".to_owned())),
            item(PASSWORD, Value::text_string(String::new())),
        ]),
    )
    .expect("an empty Text String remains a present optional field");
    assert_eq!(value_tags(present_empty_password), [USERNAME, PASSWORD]);

    for invalid in [
        structure([]),
        structure([item(USERNAME, Value::boolean(true))]),
        structure([
            item(USERNAME, Value::text_string("alice".to_owned())),
            item(PASSWORD, Value::boolean(true)),
        ]),
    ] {
        assert!(credential_value(1, invalid).is_err());
    }
}

#[test]
fn one_time_password_requires_username_and_otp_and_validates_optional_password() {
    let valid = credential_value(
        4,
        structure([
            item(USERNAME, Value::text_string("alice".to_owned())),
            item(ONE_TIME_PASSWORD, Value::text_string("123456".to_owned())),
        ]),
    )
    .expect("Username and One Time Password are required");
    assert!(matches!(valid, CredentialValue::OneTimePassword(_)));
    assert_eq!(value_tags(valid), [USERNAME, ONE_TIME_PASSWORD]);

    let present_empty_password = credential_value(
        4,
        structure([
            item(USERNAME, Value::text_string("alice".to_owned())),
            item(PASSWORD, Value::text_string(String::new())),
            item(ONE_TIME_PASSWORD, Value::text_string("123456".to_owned())),
        ]),
    )
    .expect("the optional Password may be present with empty text");
    assert_eq!(
        value_tags(present_empty_password),
        [USERNAME, PASSWORD, ONE_TIME_PASSWORD]
    );

    for invalid in [
        structure([item(
            ONE_TIME_PASSWORD,
            Value::text_string("123456".to_owned()),
        )]),
        structure([item(USERNAME, Value::text_string("alice".to_owned()))]),
        structure([
            item(USERNAME, Value::boolean(true)),
            item(ONE_TIME_PASSWORD, Value::text_string("123456".to_owned())),
        ]),
        structure([
            item(USERNAME, Value::text_string("alice".to_owned())),
            item(ONE_TIME_PASSWORD, Value::boolean(true)),
        ]),
        structure([
            item(USERNAME, Value::text_string("alice".to_owned())),
            item(PASSWORD, Value::boolean(true)),
            item(ONE_TIME_PASSWORD, Value::text_string("123456".to_owned())),
        ]),
    ] {
        assert!(credential_value(4, invalid).is_err());
    }
}

#[test]
fn device_requires_a_named_identifier_and_preserves_all_six_text_fields() {
    let source = structure([
        item(DEVICE_SERIAL_NUMBER, Value::text_string(String::new())),
        item(PASSWORD, Value::text_string(String::new())),
        item(
            DEVICE_IDENTIFIER,
            Value::text_string("device-id".to_owned()),
        ),
        item(
            NETWORK_IDENTIFIER,
            Value::text_string("network-id".to_owned()),
        ),
        item(
            MACHINE_IDENTIFIER,
            Value::text_string("machine-id".to_owned()),
        ),
        item(MEDIA_IDENTIFIER, Value::text_string("media-id".to_owned())),
    ]);
    let typed =
        credential_value(2, source).expect("all six Table 412 Text String fields are valid");
    assert!(matches!(typed, CredentialValue::Device(_)));
    let roundtrip: Structure = typed.into_ttlv();
    let view = roundtrip.view();
    let fields: Vec<(u32, String)> = view
        .children()
        .iter()
        .map(|field| {
            let value = field.with_value(|value| match value {
                ValueView::TextString(text) => Some(text.to_owned()),
                _ => None,
            });
            (
                field.tag().raw(),
                value.expect("all Table 412 members are Text Strings"),
            )
        })
        .collect();
    assert_eq!(
        fields,
        [
            (DEVICE_SERIAL_NUMBER, String::new()),
            (PASSWORD, String::new()),
            (DEVICE_IDENTIFIER, "device-id".to_owned()),
            (NETWORK_IDENTIFIER, "network-id".to_owned()),
            (MACHINE_IDENTIFIER, "machine-id".to_owned()),
            (MEDIA_IDENTIFIER, "media-id".to_owned()),
        ]
    );

    for identifier in [
        DEVICE_SERIAL_NUMBER,
        NETWORK_IDENTIFIER,
        MACHINE_IDENTIFIER,
        MEDIA_IDENTIFIER,
    ] {
        let value = credential_value(
            2,
            structure([item(identifier, Value::text_string(String::new()))]),
        )
        .expect("presence of each Table 412 identifier member counts even when empty");
        assert!(matches!(value, CredentialValue::Device(_)));
    }

    for (field_tag, fallback_identifier) in [
        (DEVICE_SERIAL_NUMBER, NETWORK_IDENTIFIER),
        (PASSWORD, DEVICE_SERIAL_NUMBER),
        (DEVICE_IDENTIFIER, DEVICE_SERIAL_NUMBER),
        (NETWORK_IDENTIFIER, DEVICE_SERIAL_NUMBER),
        (MACHINE_IDENTIFIER, DEVICE_SERIAL_NUMBER),
        (MEDIA_IDENTIFIER, DEVICE_SERIAL_NUMBER),
    ] {
        let mut fields = Vec::new();
        if fallback_identifier != field_tag {
            fields.push(item(
                fallback_identifier,
                Value::text_string("valid-identifier".to_owned()),
            ));
        }
        fields.push(item(field_tag, Value::boolean(true)));
        assert!(credential_value(2, structure(fields)).is_err());
    }
}

#[test]
fn device_password_or_device_identifier_alone_does_not_satisfy_identifier_presence() {
    assert!(credential_value(2, Structure::new()).is_err());
    assert!(
        credential_value(
            2,
            structure([item(PASSWORD, Value::text_string("secret".to_owned()))]),
        )
        .is_err()
    );
    assert!(
        credential_value(
            2,
            structure([item(
                DEVICE_IDENTIFIER,
                Value::text_string("device".to_owned()),
            )]),
        )
        .is_err()
    );
}

#[test]
fn ticket_requires_nested_ticket_type_and_ticket_value_with_table_types() {
    let value = credential_value(
        6,
        structure([item(
            TICKET,
            Value::structure(structure([
                item(TICKET_TYPE, Value::enumeration(0xF123_4567)),
                item(TICKET_VALUE, Value::byte_string(vec![0x00, 0x80, 0xFE])),
            ])),
        )]),
    )
    .expect("Ticket Type is an Enumeration and Ticket Value is a Byte String");
    assert!(matches!(value, CredentialValue::Ticket(_)));

    for invalid in [
        Structure::new(),
        structure([item(TICKET, Value::text_string("ticket".to_owned()))]),
        structure([item(TICKET, Value::structure(Structure::new()))]),
        structure([item(
            TICKET,
            Value::structure(structure([
                item(TICKET_TYPE, Value::text_string("type".to_owned())),
                item(TICKET_VALUE, Value::byte_string(vec![0x01])),
            ])),
        )]),
        structure([item(
            TICKET,
            Value::structure(structure([
                item(TICKET_TYPE, Value::enumeration(1)),
                item(TICKET_VALUE, Value::text_string("value".to_owned())),
            ])),
        )]),
    ] {
        assert!(credential_value(6, invalid).is_err());
    }
}

#[test]
fn generic_ttlv_retains_unvalidated_device_values() {
    let source = structure([
        item(CREDENTIAL_TYPE, Value::enumeration(2)),
        item(CREDENTIAL_VALUE, Value::structure(Structure::new())),
    ]);
    let parsed = kmipkit_protocol::Credential::try_from_ttlv(source)
        .expect("the generic Credential envelope remains structurally valid");
    let roundtrip = parsed.into_ttlv();
    let view = roundtrip.view();
    let fields = view.children();

    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].tag().raw(), CREDENTIAL_TYPE);
    assert_eq!(fields[1].tag().raw(), CREDENTIAL_VALUE);
    assert!(credential_value(2, Structure::new()).is_err());
}

#[test]
fn hashed_password_requires_username_timestamp_and_hash_bytes_with_table_types() {
    let valid = structure([
        item(USERNAME, Value::text_string("alice".to_owned())),
        item(TIME_STAMP, Value::date_time_extended(0x0102_0304_0506_0708)),
        item(HASHED_PASSWORD, Value::byte_string(vec![0x00, 0x80, 0xFE])),
    ]);
    assert!(credential_value(5, valid).is_ok());

    for invalid in [
        structure([
            item(TIME_STAMP, Value::date_time_extended(1)),
            item(HASHED_PASSWORD, Value::byte_string(vec![0x01])),
        ]),
        structure([
            item(USERNAME, Value::text_string("alice".to_owned())),
            item(HASHED_PASSWORD, Value::byte_string(vec![0x01])),
        ]),
        structure([
            item(USERNAME, Value::text_string("alice".to_owned())),
            item(TIME_STAMP, Value::date_time_extended(1)),
        ]),
        structure([
            item(USERNAME, Value::boolean(true)),
            item(TIME_STAMP, Value::date_time_extended(1)),
            item(HASHED_PASSWORD, Value::byte_string(vec![0x01])),
        ]),
        structure([
            item(USERNAME, Value::text_string("alice".to_owned())),
            item(TIME_STAMP, Value::date_time(1)),
            item(HASHED_PASSWORD, Value::byte_string(vec![0x01])),
        ]),
        structure([
            item(USERNAME, Value::text_string("alice".to_owned())),
            item(TIME_STAMP, Value::date_time_extended(1)),
            item(HASHED_PASSWORD, Value::text_string("hash".to_owned())),
        ]),
        structure([
            item(USERNAME, Value::text_string("alice".to_owned())),
            item(TIME_STAMP, Value::date_time_extended(1)),
            item(HASHING_ALGORITHM, Value::boolean(true)),
            item(HASHED_PASSWORD, Value::byte_string(vec![0x01])),
        ]),
    ] {
        assert!(credential_value(5, invalid).is_err());
    }
}

#[test]
fn hashed_password_omission_exposes_sha256_default_without_materializing_field() {
    let timestamp = 0x1122_3344_5566_7788;
    let secret_bytes = vec![0x00, 0x80, 0xFE, 0x7F];
    let parsed = credential_value(
        5,
        structure([
            item(USERNAME, Value::text_string("alice".to_owned())),
            item(TIME_STAMP, Value::date_time_extended(timestamp)),
            item(HASHED_PASSWORD, Value::byte_string(secret_bytes.clone())),
        ]),
    )
    .expect("required Hashed Password fields are present");
    let CredentialValue::HashedPassword(typed) = parsed else {
        panic!("Credential Type 5 must produce the Hashed Password variant");
    };
    assert_eq!(typed.hashing_algorithm_raw(), None);
    assert_eq!(typed.effective_hashing_algorithm_raw(), 6);

    let roundtrip = CredentialValue::HashedPassword(typed).into_ttlv();
    let view = roundtrip.view();
    let fields = view.children();
    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [USERNAME, TIME_STAMP, HASHED_PASSWORD]
    );
    let roundtrip_timestamp = fields[1].with_value(|value| match value {
        ValueView::DateTimeExtended(value) => Some(*value),
        _ => None,
    });
    let roundtrip_hash = fields[2].with_value(|value| match value {
        ValueView::ByteString(bytes) => Some(bytes.to_vec()),
        _ => None,
    });
    assert_eq!(roundtrip_timestamp, Some(timestamp));
    assert_eq!(roundtrip_hash, Some(secret_bytes));
}

#[test]
fn hashed_password_preserves_explicit_and_unknown_algorithm_values() {
    let timestamp = -0x0102_0304_0506_0708;
    let secret_bytes = vec![0xFF, 0x00, 0x80, 0x01];
    for algorithm in [1, 6, 0xF123_4567] {
        let parsed = credential_value(
            5,
            structure([
                item(USERNAME, Value::text_string("alice".to_owned())),
                item(TIME_STAMP, Value::date_time_extended(timestamp)),
                item(HASHING_ALGORITHM, Value::enumeration(algorithm)),
                item(HASHED_PASSWORD, Value::byte_string(secret_bytes.clone())),
            ]),
        )
        .expect("explicit assigned and unknown algorithms remain valid raw values");
        let CredentialValue::HashedPassword(typed) = parsed else {
            panic!("Credential Type 5 must produce the Hashed Password variant");
        };
        assert_eq!(typed.hashing_algorithm_raw(), Some(algorithm));
        assert_eq!(typed.effective_hashing_algorithm_raw(), algorithm);

        let roundtrip = CredentialValue::HashedPassword(typed).into_ttlv();
        let view = roundtrip.view();
        let fields = view.children();
        assert_eq!(
            fields
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>(),
            [USERNAME, TIME_STAMP, HASHING_ALGORITHM, HASHED_PASSWORD]
        );
        assert_eq!(
            fields[2].with_value(|value| match value {
                ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            }),
            Some(algorithm)
        );
        assert_eq!(
            fields[1].with_value(|value| match value {
                ValueView::DateTimeExtended(raw) => Some(*raw),
                _ => None,
            }),
            Some(timestamp)
        );
        assert_eq!(
            fields[3].with_value(|value| match value {
                ValueView::ByteString(bytes) => Some(bytes.to_vec()),
                _ => None,
            }),
            Some(secret_bytes.clone())
        );
    }
}
