//! OASIS KMIP 2.1 §§3.1–3.2, Tables 12–13: Key Block and Key Value.

use kmipkit_protocol::{
    EncryptionKeyInformation, KeyBlock, KeyBlockValue, KeyCompressionType, KeyFormatType,
    KeyMaterial, KeyValue, KeyWrappingData, SecretBytes, TransparentSymmetricKey,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Value, ValueView};

#[test]
fn key_block_builder_emits_canonical_order_independent_of_setter_order() {
    let key_value = KeyValue::try_new(
        KeyMaterial::byte_string(1, vec![0xA1, 0xA2]).expect("Raw material is a Byte String"),
        None,
    )
    .expect("valid plaintext Key Value");
    let block = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("valid Key Format Type")
        .with_cryptographic_properties(Some(0x8000_0001), Some(256))
        .with_key_compression_type(KeyCompressionType::from_raw(1))
        .with_key_value(KeyBlockValue::Plaintext(key_value));
    let item = block.try_into_item().expect("paired cryptographic fields");
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
            0x0042_0042,
            0x0042_0041,
            0x0042_0045,
            0x0042_0028,
            0x0042_002A
        ]
    );
}

#[test]
fn plaintext_and_wrapped_key_values_have_distinct_ttlv_types() {
    let plaintext = KeyValue::try_new(
        KeyMaterial::byte_string(1, vec![1, 2]).expect("Raw material is a Byte String"),
        None,
    )
    .expect("valid plaintext");
    let plaintext_block = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("valid format")
        .with_key_value(KeyBlockValue::Plaintext(plaintext))
        .try_into_item()
        .expect("valid block");

    let wrapped_builder = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("valid format")
        .with_key_value(KeyBlockValue::Wrapped(SecretBytes::new(vec![3, 4])))
        .with_key_wrapping_data(
            KeyWrappingData::try_new(1)
                .expect("assigned Wrapping Method")
                .with_encryption_key_information(
                    EncryptionKeyInformation::try_new("wrapping-key".to_owned())
                        .expect("required Unique Identifier"),
                ),
        );
    assert_eq!(wrapped_builder.key_value_type(), Some(ItemType::ByteString));
    let wrapped_block = wrapped_builder
        .try_into_item()
        .expect("valid wrapped block");

    let key_value_type = |item: &kmipkit_ttlv::Item| {
        item.with_value(|value| match value {
            ValueView::Structure(structure) => structure
                .children()
                .iter()
                .find(|field| field.tag().raw() == 0x0042_0045)
                .map(kmipkit_ttlv::Item::item_type),
            _ => None,
        })
    };
    assert_eq!(key_value_type(&plaintext_block), Some(ItemType::Structure));
    assert_eq!(key_value_type(&wrapped_block), Some(ItemType::ByteString));
    assert!(wrapped_block.with_value(|value| match value {
        ValueView::Structure(block) => block.children().iter().any(|field| {
            field.tag().raw() == 0x0042_0045
                && field.with_value(
                    |value| matches!(value, ValueView::ByteString(bytes) if bytes == [3, 4]),
                )
        }),
        _ => false,
    }));
}

#[test]
fn key_block_rejects_unpaired_algorithm_and_length() {
    let item = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("valid format")
        .with_cryptographic_properties(Some(3), None)
        .try_into_item();
    assert!(item.is_err());
}

#[test]
fn wrapped_key_value_requires_key_wrapping_data() {
    let item = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("valid format")
        .with_key_value(KeyBlockValue::Wrapped(SecretBytes::new(vec![3, 4])))
        .try_into_item();
    assert!(item.is_err());
}

#[test]
fn plaintext_key_value_forbids_key_wrapping_data() {
    let key_value = KeyValue::try_new(
        KeyMaterial::byte_string(1, vec![1, 2]).expect("Raw material is a Byte String"),
        None,
    )
    .expect("valid plaintext Key Value");
    let item = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("valid format")
        .with_key_value(KeyBlockValue::Plaintext(key_value))
        .with_key_wrapping_data(
            KeyWrappingData::try_new(1)
                .expect("valid wrapper")
                .with_encryption_key_information(
                    EncryptionKeyInformation::try_new("wrapping-key".to_owned())
                        .expect("required Unique Identifier"),
                ),
        )
        .try_into_item();
    assert!(item.is_err());
}

#[test]
fn key_format_type_must_match_the_material_form() {
    let key_value = KeyValue::try_new(
        KeyMaterial::byte_string(2, vec![1, 2]).expect("Opaque material is a Byte String"),
        None,
    )
    .expect("valid Opaque Key Material");
    let item = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("valid Raw format")
        .with_key_value(KeyBlockValue::Plaintext(key_value))
        .try_into_item();
    assert!(item.is_err());
}

#[test]
fn key_block_reader_preserves_the_tree_and_exposes_typed_header_values() {
    let source = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("Raw format")
        .with_cryptographic_properties(Some(0x8000_0001), Some(128))
        .try_into_item()
        .expect("paired cryptographic fields");
    let typed = KeyBlock::try_from_item(source.try_clone().expect("explicit zeroizing copy"))
        .expect("valid Key Block can be read");
    assert_eq!(typed.key_format_type().raw(), 1);
    assert!(typed.has_cryptographic_properties());
    assert_eq!(typed.key_value_type(), None);
    let round_trip = typed.try_into_item().expect("parsed item remains lossless");
    assert_eq!(round_trip.tag().raw(), source.tag().raw());
    assert_eq!(round_trip.item_type(), source.item_type());
}

#[test]
fn key_block_and_key_value_accessors_cover_builder_and_reader_forms() {
    let value = KeyValue::try_new(
        KeyMaterial::byte_string(1, vec![0xD1, 0xD2]).expect("Raw bytes"),
        None,
    )
    .expect("valid Key Value");
    let builder = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("Raw format")
        .with_key_compression_type(KeyCompressionType::from_raw(1))
        .with_key_value(KeyBlockValue::Plaintext(value))
        .with_cryptographic_properties(Some(3), Some(128));
    assert_eq!(builder.key_format_type().raw(), 1);
    assert_eq!(builder.key_value_type(), Some(ItemType::Structure));
    assert_eq!(
        builder
            .with_raw(|view| view.children().len())
            .expect("builder exposes its current incomplete structure"),
        5
    );
    assert!(builder.has_cryptographic_properties());
    assert!(!format!("{builder:?}").contains("D1"));
    let item = builder.try_into_item().expect("valid Key Block");
    let parsed = KeyBlock::try_from_item(item).expect("read Key Block");
    assert_eq!(parsed.key_value_type(), Some(ItemType::Structure));
    assert!(parsed.with_raw(|view| view.children().len()).is_ok());
    assert!(!format!("{parsed:?}").contains("D1"));

    let value = KeyValue::try_new(
        KeyMaterial::byte_string(1, vec![0xE1]).expect("Raw bytes"),
        None,
    )
    .expect("valid Key Value");
    assert!(value.with_raw(|view| view.children().len()).is_ok());
    assert!(!format!("{value:?}").contains("E1"));

    let item = value.into_item();
    let parsed =
        KeyValue::try_from_item(item, KeyFormatType::from_raw(1)).expect("read typed Key Value");
    assert_eq!(parsed.into_item().tag().raw(), 0x0042_0045);
}

#[test]
fn incomplete_key_block_is_inspectable_but_conversion_still_validates() {
    let builder = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("Raw format")
        .with_cryptographic_properties(Some(3), None);
    assert_eq!(
        builder
            .with_raw(|view| view.children().len())
            .expect("incomplete builder can be inspected"),
        2
    );
    assert!(builder.try_into_item().is_err());
}

#[test]
fn key_block_setters_update_decoded_tree_and_preserve_extensions() {
    let source = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("Raw format")
        .with_cryptographic_properties(Some(3), Some(128))
        .try_into_item()
        .expect("paired cryptographic fields");
    let mut children = source.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .map(Item::try_clone)
            .collect::<Result<Vec<_>, _>>()
            .expect("explicit zeroizing copy"),
        _ => Vec::new(),
    });
    children.push(tagged(0x0054_0101, Value::integer(77)));
    let source = structure_item(0x0042_0040, children);

    let parsed = KeyBlock::try_from_item(source)
        .expect("read valid key block")
        .with_cryptographic_properties(Some(4), Some(256));
    assert_eq!(
        parsed
            .with_raw(|view| view.children()[1].with_value(|value| match value {
                ValueView::Enumeration(raw) => *raw,
                _ => 0,
            }))
            .expect("raw view reflects decoded-model mutation"),
        4
    );
    let item = parsed
        .try_into_item()
        .expect("mutated key block remains valid");

    item.with_value(|value| match value {
        ValueView::Structure(structure) => {
            let fields = structure.children();
            assert_eq!(
                fields[1].with_value(|v| match v {
                    ValueView::Enumeration(raw) => *raw,
                    _ => 0,
                }),
                4
            );
            assert_eq!(
                fields[2].with_value(|v| match v {
                    ValueView::Integer(raw) => *raw,
                    _ => 0,
                }),
                256
            );
            assert_eq!(fields[3].tag().raw(), 0x0054_0101);
        }
        _ => panic!("Key Block remains a Structure"),
    });
}

#[test]
fn key_block_and_key_value_readers_reject_malformed_trees() {
    let valid_format = || tagged(0x0042_0042, Value::enumeration(1));
    let missing_format = structure_item(0x0042_0040, []);
    assert!(KeyBlock::try_from_item(missing_format).is_err());

    let duplicate_format = structure_item(0x0042_0040, [valid_format(), valid_format()]);
    assert!(KeyBlock::try_from_item(duplicate_format).is_err());

    let out_of_order = structure_item(
        0x0042_0040,
        [
            tagged(0x0042_0045, Value::byte_string(vec![1])),
            valid_format(),
        ],
    );
    assert!(KeyBlock::try_from_item(out_of_order).is_err());

    let wrong_format_type = structure_item(
        0x0042_0040,
        [tagged(0x0042_0042, Value::text_string("Raw".to_owned()))],
    );
    assert!(KeyBlock::try_from_item(wrong_format_type).is_err());

    let vendor = tagged(0x0054_0101, Value::integer(1));
    let vendor_extension = structure_item(0x0042_0040, [valid_format(), vendor]);
    let parsed_vendor = KeyBlock::try_from_item(vendor_extension)
        .expect("vendor fields are retained without interpretation");
    assert_eq!(
        parsed_vendor
            .with_raw(|view| view.children().len())
            .unwrap(),
        2
    );

    let unpaired_algorithm = structure_item(
        0x0042_0040,
        [valid_format(), tagged(0x0042_0028, Value::enumeration(3))],
    );
    assert!(KeyBlock::try_from_item(unpaired_algorithm).is_err());

    let wrong_tag = structure_item(0x0042_0045, [valid_format()]);
    assert!(KeyBlock::try_from_item(wrong_tag).is_err());
    assert!(KeyBlock::try_from_item(tagged(0x0042_0040, Value::enumeration(1))).is_err());

    let no_material = structure_item(0x0042_0045, []);
    assert!(KeyValue::try_from_item(no_material, KeyFormatType::from_raw(1)).is_err());
    let material = tagged(0x0042_0043, Value::byte_string(vec![0xF1]));
    let attributes = tagged(0x0042_0125, Value::structure(Structure::new()));
    let attributes_first = structure_item(0x0042_0045, [attributes, material]);
    assert!(KeyValue::try_from_item(attributes_first, KeyFormatType::from_raw(1)).is_err());
    let wrong_material_format = structure_item(
        0x0042_0045,
        [tagged(0x0042_0043, Value::structure(Structure::new()))],
    );
    assert!(KeyValue::try_from_item(wrong_material_format, KeyFormatType::from_raw(1)).is_err());
    assert!(
        KeyValue::try_from_item(
            tagged(0x0042_0040, Value::structure(Structure::new())),
            KeyFormatType::from_raw(1),
        )
        .is_err()
    );
    assert!(
        KeyValue::try_from_item(
            tagged(0x0042_0045, Value::enumeration(1)),
            KeyFormatType::from_raw(1),
        )
        .is_err()
    );
    let vendor = tagged(0x0054_0101, Value::integer(1));
    let generic = structure_item(
        0x0042_0045,
        [tagged(0x0042_0043, Value::byte_string(vec![1])), vendor],
    );
    let parsed = KeyValue::try_from_item(generic, KeyFormatType::from_raw(1))
        .expect("unknown Key Value children remain valid");
    assert_eq!(parsed.with_raw(|view| view.children().len()).unwrap(), 2);
}

#[test]
fn key_formats_accept_transparent_and_future_structure_material() {
    let transparent = TransparentSymmetricKey::try_new(vec![0xA1])
        .expect("transparent structure")
        .into_item();
    let material = KeyMaterial::try_from_item(transparent, KeyFormatType::from_raw(7))
        .expect("transparent format accepts Structure material");
    let value = KeyValue::try_new(material, None).expect("typed transparent Key Value");
    let key_block = KeyBlock::try_new(KeyFormatType::from_raw(7))
        .expect("transparent format")
        .with_key_value(KeyBlockValue::Plaintext(value));
    assert_eq!(key_block.try_into_item().unwrap().tag().raw(), 0x0042_0040);

    let future_structure = KeyMaterial::structure(0x8000_0001, Structure::new())
        .expect("future format accepts caller-selected Structure");
    let future_value = KeyValue::try_new(future_structure, None).expect("future Key Value");
    let future_block = KeyBlock::try_new(KeyFormatType::from_raw(0x8000_0001))
        .expect("future Key Format Type")
        .with_key_value(KeyBlockValue::Plaintext(future_value));
    assert_eq!(
        future_block.try_into_item().unwrap().tag().raw(),
        0x0042_0040
    );
}

fn tagged(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("valid test Item")
}

fn structure_item(raw_tag: u32, fields: impl IntoIterator<Item = Item>) -> Item {
    let mut structure = Structure::new();
    for field in fields {
        structure.try_push(field).expect("valid test structure");
    }
    tagged(raw_tag, Value::structure(structure))
}

fn tag(raw: u32) -> kmipkit_ttlv::Tag {
    RawTag::new(raw)
        .expect("tag fits")
        .try_checked()
        .expect("assigned tag")
}
