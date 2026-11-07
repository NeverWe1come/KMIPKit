//! OASIS KMIP 2.1 §3.3, Tables 14–17: typed wrapping metadata.

use kmipkit_protocol::{EncryptionKeyInformation, KeyWrappingData, MacSignatureKeyInformation};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

#[test]
fn key_wrapping_data_emits_method_before_optional_metadata() {
    let wrapping = KeyWrappingData::try_new(0x8000_0001)
        .expect("unknown wrapping method is preserved")
        .with_encryption_key_information(
            EncryptionKeyInformation::try_new("wrap-key".to_owned())
                .expect("required Unique Identifier"),
        )
        .with_mac_signature_key_information(
            MacSignatureKeyInformation::try_new("mac-key".to_owned())
                .expect("required Unique Identifier"),
        )
        .with_mac_signature(vec![0xA1])
        .with_iv_counter_nonce(vec![0xB1])
        .with_encoding_option(2);
    let item = wrapping.into_item().expect("valid wrapping data");
    let tags = item.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    });
    assert_eq!(
        tags,
        [
            0x0042_009E,
            0x0042_0036,
            0x0042_004E,
            0x0042_004D,
            0x0042_003D,
            0x0042_00A3,
        ]
    );
}

#[test]
fn wrapping_data_retains_raw_metadata_and_never_performs_crypto() {
    let builder = KeyWrappingData::try_new(1)
        .expect("valid wrapper")
        .with_encryption_key_information(
            EncryptionKeyInformation::try_new("wrapping-key".to_owned())
                .expect("required Unique Identifier"),
        )
        .with_iv_counter_nonce(vec![0xA1, 0xA2])
        .with_mac_signature(vec![0xB1, 0xB2]);
    assert!(builder.with_raw(|_| ()).is_err());
    let item = builder.into_item().expect("valid wrapping data");
    assert_eq!(item.tag().raw(), 0x0042_0046);
}

#[test]
fn wrapping_data_requires_encryption_or_mac_signature_key_information() {
    let constructed = KeyWrappingData::try_new(1)
        .expect("known Wrapping Method")
        .into_item();
    assert!(constructed.is_err());

    let mut children = Structure::new();
    children
        .try_push(tagged(0x0042_009E, Value::enumeration(1)))
        .expect("valid Wrapping Method");
    let parsed = KeyWrappingData::try_from_item(
        Item::new(tag(0x0042_0046), Value::structure(children)).expect("valid Key Wrapping Data"),
    );
    assert!(parsed.is_err());
}

#[test]
fn wrapping_reader_checks_required_identifiers_and_keeps_unknown_fields() {
    let item = KeyWrappingData::try_new(0x8000_0001)
        .expect("future Wrapping Method remains open")
        .with_encryption_key_information(
            EncryptionKeyInformation::try_new("wrapping-key".to_owned())
                .expect("key information has a Unique Identifier"),
        )
        .into_item()
        .expect("valid wrapping structure");
    let parsed = KeyWrappingData::try_from_item(item.try_clone().expect("explicit zeroizing copy"))
        .expect("valid wrapping data can be read");
    let tags = parsed
        .with_raw(|view| {
            view.children()
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>()
        })
        .expect("parsed wrapper lends its tree");
    assert_eq!(tags, [0x0042_009E, 0x0042_0036]);

    let mut malformed = kmipkit_ttlv::Structure::new();
    malformed
        .try_push(
            kmipkit_ttlv::Item::new(
                kmipkit_ttlv::RawTag::new(0x0042_0036)
                    .expect("tag fits")
                    .try_checked()
                    .expect("assigned tag"),
                kmipkit_ttlv::Value::structure(kmipkit_ttlv::Structure::new()),
            )
            .expect("valid child"),
        )
        .expect("valid nesting");
    let malformed = kmipkit_ttlv::Item::new(
        kmipkit_ttlv::RawTag::new(0x0042_0046)
            .expect("tag fits")
            .try_checked()
            .expect("assigned tag"),
        kmipkit_ttlv::Value::structure(malformed),
    )
    .expect("valid test structure");
    assert!(KeyWrappingData::try_from_item(malformed).is_err());
}

#[test]
fn wrapping_reader_covers_optional_fields_and_key_information_variants() {
    let encryption = EncryptionKeyInformation::try_from_unique_identifier(
        kmipkit_protocol::UniqueIdentifier::Enumeration(1),
    )
    .expect("enumerated identifiers are open")
    .with_cryptographic_parameters(Structure::new())
    .expect("optional parameters follow the identifier");
    let mac = MacSignatureKeyInformation::try_from_unique_identifier(
        kmipkit_protocol::UniqueIdentifier::Integer(7),
    )
    .expect("integer identifiers are open")
    .with_cryptographic_parameters(Structure::new())
    .expect("optional parameters follow the identifier");
    let item = KeyWrappingData::try_new(1)
        .expect("Wrapping Method")
        .with_encryption_key_information(encryption)
        .with_mac_signature_key_information(mac)
        .with_mac_signature(vec![0xA1])
        .with_iv_counter_nonce(vec![0xB1])
        .with_encoding_option(2)
        .into_item()
        .expect("valid complete wrapping data");
    let mut fields = item.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .map(kmipkit_ttlv::Item::try_clone)
            .collect::<Result<Vec<_>, _>>()
            .expect("test-only metadata clone"),
        _ => Vec::new(),
    });
    fields.push(tagged(0x0054_0101, Value::integer(1)));
    let item = structure_item(0x0042_0046, fields);
    let encryption_source = key_information(0x0042_0036, 0x0042_0094, Value::enumeration(1));
    let encryption = EncryptionKeyInformation::try_from_item(encryption_source)
        .expect("read encryption key information");
    assert!(encryption.with_raw(|view| view.children().len()).is_ok());
    let mac_source = key_information(0x0042_004E, 0x0042_0094, Value::integer(7));
    let mac = MacSignatureKeyInformation::try_from_item(mac_source)
        .expect("read MAC/Signature key information");
    assert!(mac.with_raw(|view| view.children().len()).is_ok());
    let parsed = KeyWrappingData::try_from_item(item).expect("all optional fields parse");
    assert_eq!(
        parsed
            .with_raw(|view| view.children().len())
            .expect("parsed structure is borrowed"),
        7
    );
    assert_eq!(parsed.into_item().unwrap().tag().raw(), 0x0042_0046);
}

#[test]
fn key_information_readers_reject_wrong_tags_types_and_missing_identifiers() {
    assert!(
        EncryptionKeyInformation::try_from_item(tagged(0x0042_0036, Value::enumeration(1),))
            .is_err()
    );
    assert!(
        MacSignatureKeyInformation::try_from_item(tagged(
            0x0042_0036,
            Value::structure(Structure::new()),
        ))
        .is_err()
    );
    assert!(EncryptionKeyInformation::try_from_item(structure_item(0x0042_0036, [])).is_err());

    let wrong_identifier = structure_item(
        0x0042_0036,
        [tagged(0x0042_0094, Value::byte_string(vec![1]))],
    );
    assert!(EncryptionKeyInformation::try_from_item(wrong_identifier).is_err());
    let wrong_parameters = structure_item(
        0x0042_0036,
        [
            tagged(0x0042_0094, Value::text_string("id".to_owned())),
            tagged(0x0042_002B, Value::integer(1)),
        ],
    );
    assert!(EncryptionKeyInformation::try_from_item(wrong_parameters).is_err());
}

#[test]
fn wrapping_reader_rejects_bad_outer_schema_and_member_order() {
    assert!(KeyWrappingData::try_from_item(tagged(0x0042_009E, Value::enumeration(1))).is_err());
    assert!(KeyWrappingData::try_from_item(tagged(0x0042_0046, Value::enumeration(1))).is_err());

    let missing_method = structure_item(
        0x0042_0046,
        [key_information(
            0x0042_0036,
            0x0042_0094,
            Value::text_string("id".to_owned()),
        )],
    );
    assert!(KeyWrappingData::try_from_item(missing_method).is_err());

    let wrong_order = structure_item(
        0x0042_0046,
        [
            key_information(
                0x0042_0036,
                0x0042_0094,
                Value::text_string("id".to_owned()),
            ),
            tagged(0x0042_009E, Value::enumeration(1)),
        ],
    );
    assert!(KeyWrappingData::try_from_item(wrong_order).is_err());

    let wrong_field_type = structure_item(
        0x0042_0046,
        [
            tagged(0x0042_009E, Value::enumeration(1)),
            key_information(
                0x0042_0036,
                0x0042_0094,
                Value::text_string("id".to_owned()),
            ),
            tagged(0x0042_004D, Value::integer(1)),
        ],
    );
    assert!(KeyWrappingData::try_from_item(wrong_field_type).is_err());

    let duplicate_method = structure_item(
        0x0042_0046,
        [
            tagged(0x0042_009E, Value::enumeration(1)),
            tagged(0x0042_009E, Value::enumeration(2)),
            key_information(
                0x0042_0036,
                0x0042_0094,
                Value::text_string("id".to_owned()),
            ),
        ],
    );
    assert!(KeyWrappingData::try_from_item(duplicate_method).is_err());
}

fn structure_item(raw_tag: u32, fields: impl IntoIterator<Item = Item>) -> Item {
    let mut structure = Structure::new();
    for field in fields {
        structure.try_push(field).expect("valid test structure");
    }
    tagged(raw_tag, Value::structure(structure))
}

fn key_information(raw_tag: u32, identifier_tag: u32, identifier: Value) -> Item {
    structure_item(raw_tag, [tagged(identifier_tag, identifier)])
}

fn tagged(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("valid test Item")
}

fn tag(raw: u32) -> kmipkit_ttlv::Tag {
    RawTag::new(raw)
        .expect("tag fits")
        .try_checked()
        .expect("assigned tag")
}
