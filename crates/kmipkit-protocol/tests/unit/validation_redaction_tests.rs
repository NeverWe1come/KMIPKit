use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

use crate::{
    NestedStructureSchema, RequestPayloadSchema, UsernameAndPasswordCredential, ValidationSchema,
    validate_structure,
};

fn item(tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(tag)
            .expect("test tag fits the TTLV tag field")
            .try_checked()
            .expect("test tag is assigned or a vendor extension"),
        value,
    )
    .expect("test item is valid generic TTLV")
}

#[test]
fn validation_is_read_only_preserves_unknown_values_and_redacts_input() {
    const CANARY: &str = "KMIPKIT_0051_VALIDATION_SECRET_CANARY";
    const OBJECT_TYPE: u32 = 0x0042_0057;
    const ATTRIBUTES: u32 = 0x0042_0125;
    const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
    const VENDOR_EXTENSION: u32 = 0x0054_4321;

    let mut attributes = Structure::new();
    attributes
        .try_push(item(
            CRYPTOGRAPHIC_USAGE_MASK,
            Value::integer(i32::MIN | 0x05),
        ))
        .expect("mask attribute is valid TTLV");
    attributes
        .try_push(item(VENDOR_EXTENSION, Value::byte_string(vec![0xCA, 0xFE])))
        .expect("first vendor extension is valid TTLV");
    attributes
        .try_push(item(VENDOR_EXTENSION, Value::enumeration(0xF123_4567)))
        .expect("repeated vendor extension is valid TTLV");

    let mut payload = Structure::new();
    payload
        .try_push(item(OBJECT_TYPE, Value::text_string(CANARY.to_owned())))
        .expect("malformed known field remains valid generic TTLV");
    payload
        .try_push(item(ATTRIBUTES, Value::structure(attributes)))
        .expect("Attributes is a Structure");
    payload
        .try_push(item(VENDOR_EXTENSION, Value::byte_string(vec![0x11, 0x22])))
        .expect("first top-level extension is valid TTLV");
    payload
        .try_push(item(VENDOR_EXTENSION, Value::byte_string(vec![0x33, 0x44])))
        .expect("repeated top-level extension is valid TTLV");

    let report = validate_structure(
        &payload.view(),
        Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::Create,
        )),
    );

    assert!(!report.is_valid());
    assert_eq!(report.issues().len(), 1);
    let issue = &report.issues()[0];
    assert_eq!(issue.path().to_string(), "Request Payload.Object Type");
    for diagnostic in [format!("{issue:?}"), format!("{report:?}")] {
        assert!(!diagnostic.contains(CANARY));
        assert!(!diagnostic.contains("CAFE"));
    }

    let payload_view = payload.view();
    let children = payload_view.children();
    assert_eq!(children.len(), 4);
    assert_eq!(children[0].tag().raw(), OBJECT_TYPE);
    assert_eq!(children[1].tag().raw(), ATTRIBUTES);
    assert_eq!(children[2].tag().raw(), VENDOR_EXTENSION);
    assert_eq!(children[3].tag().raw(), VENDOR_EXTENSION);
    assert!(children[2].with_value(|value| {
        matches!(value, ValueView::ByteString(value) if value == [0x11, 0x22])
    }));
    assert!(children[3].with_value(|value| {
        matches!(value, ValueView::ByteString(value) if value == [0x33, 0x44])
    }));
    let attribute_children = children[1].with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .map(|child| {
                let value = child.with_value(|value| match value {
                    ValueView::Integer(value) => Some(i64::from(*value)),
                    ValueView::Enumeration(value) => Some(i64::from(*value)),
                    _ => None,
                });
                (child.tag().raw(), value)
            })
            .collect::<Vec<_>>(),
        _ => panic!("Attributes remains a Structure"),
    });
    assert_eq!(attribute_children.len(), 3);
    assert_eq!(
        attribute_children[0],
        (CRYPTOGRAPHIC_USAGE_MASK, Some(i64::from(i32::MIN | 0x05)))
    );
    assert_eq!(attribute_children[1].0, VENDOR_EXTENSION);
    assert_eq!(
        attribute_children[2],
        (VENDOR_EXTENSION, Some(i64::from(0xF123_4567_u32)))
    );
    assert!(children[1].with_value(|value| match value {
        ValueView::Structure(structure) => structure.children()[1].with_value(|value| {
            matches!(value, ValueView::ByteString(value) if value == [0xCA, 0xFE])
        }),
        _ => false,
    }));
}

#[test]
fn credential_and_key_diagnostics_do_not_disclose_secret_canaries() {
    const CREDENTIAL_CANARY: &str = "KMIPKIT_0051_PASSWORD_SECRET_CANARY";
    const KEY_CANARY: &str = "KMIPKIT_0051_KEY_MATERIAL_SECRET_CANARY";

    let mut credential = Structure::new();
    credential
        .try_push(item(
            0x0042_0099,
            Value::byte_string(CREDENTIAL_CANARY.as_bytes().to_vec()),
        ))
        .expect("malformed Username remains valid generic TTLV");
    let credential_error = UsernameAndPasswordCredential::try_from_ttlv(credential)
        .expect_err("Username must be a Text String");
    assert!(!credential_error.to_string().contains(CREDENTIAL_CANARY));
    assert!(!format!("{credential_error:?}").contains(CREDENTIAL_CANARY));

    let mut key_value = Structure::new();
    key_value
        .try_push(item(0x0042_0043, Value::text_string(KEY_CANARY.to_owned())))
        .expect("malformed Key Material remains valid generic TTLV");
    let mut key_block = Structure::new();
    key_block
        .try_push(item(0x0042_0042, Value::enumeration(1)))
        .expect("Raw is a known Key Format Type");
    key_block
        .try_push(item(0x0042_0045, Value::structure(key_value)))
        .expect("Key Value is a Structure");

    let report = validate_structure(
        &key_block.view(),
        Some(ValidationSchema::Nested(NestedStructureSchema::KeyBlock)),
    );

    assert!(!report.is_valid());
    assert!(!format!("{report:?}").contains(KEY_CANARY));
}
