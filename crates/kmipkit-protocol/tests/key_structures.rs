//! OASIS KMIP 2.1 §§3.1–3.2, Tables 12–13: Key Block and Key Value.

use kmipkit_protocol::{
    KeyBlock, KeyBlockValue, KeyCompressionType, KeyFormatType, KeyMaterial, KeyValue,
    KeyWrappingData,
};
use kmipkit_ttlv::{ItemType, ValueView};

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

    let wrapped_block = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("valid format")
        .with_key_value(KeyBlockValue::Wrapped(vec![3, 4]))
        .with_key_wrapping_data(KeyWrappingData::try_new(1).expect("assigned Wrapping Method"))
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
        .with_key_value(KeyBlockValue::Wrapped(vec![3, 4]))
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
        .with_key_wrapping_data(KeyWrappingData::try_new(1).expect("valid wrapper"))
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
