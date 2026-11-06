//! OASIS KMIP 2.1 §3.2, Table 13: Key Material encoding and fallback.

use kmipkit_protocol::{KeyFormatType, KeyMaterial};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Value, ValueView};

#[test]
fn key_material_preserves_explicit_byte_string_and_structure_forms() {
    let bytes = KeyMaterial::byte_string(0x8000_0001, vec![0x91, 0x92])
        .expect("unknown future format retains byte-string material")
        .into_item();
    assert_eq!(bytes.item_type(), ItemType::ByteString);
    let structure = KeyMaterial::structure(0x8000_0002, kmipkit_ttlv::Structure::new())
        .expect("unknown extension format retains Structure material")
        .into_item();
    assert_eq!(structure.item_type(), ItemType::Structure);
}

#[test]
fn known_key_formats_reject_the_wrong_key_material_ttlv_form() {
    assert!(KeyMaterial::byte_string(7, vec![0x91]).is_err());
    assert!(KeyMaterial::structure(1, kmipkit_ttlv::Structure::new()).is_err());
}

#[test]
fn unknown_key_format_remains_open_and_raw_preserving() {
    let future = KeyFormatType::from_raw(0x8000_1001);
    assert_eq!(future.raw(), 0x8000_1001);
}

#[test]
fn key_material_reader_preserves_format_and_ttlv_type() {
    let item = KeyMaterial::byte_string(3, vec![0x91, 0x92])
        .expect("PKCS #1 material uses a Byte String")
        .into_item();
    let material = KeyMaterial::try_from_item(
        item.try_clone().expect("explicit zeroizing copy"),
        KeyFormatType::from_raw(3),
    )
    .expect("matching key material can be read");
    assert_eq!(material.format_raw(), 3);
    assert_eq!(material.item_type(), ItemType::ByteString);
    assert!(KeyMaterial::try_from_item(item, KeyFormatType::from_raw(7)).is_err());
}

#[test]
fn key_material_accessors_and_safe_formatting_cover_both_ttlv_forms() {
    let bytes = KeyMaterial::byte_string(1, vec![0xA1, 0xB2]).expect("Raw bytes");
    assert_eq!(bytes.format_raw(), 1);
    assert_eq!(bytes.item_type(), ItemType::ByteString);
    let debug = format!("{bytes:?}");
    assert!(!debug.contains("A1") && !debug.contains("B2"));
    assert!(bytes.with_raw(|value| matches!(value, ValueView::ByteString(_))));

    let symmetric = kmipkit_protocol::TransparentSymmetricKey::try_new(vec![0xC1])
        .expect("valid transparent symmetric material")
        .into_item();
    let structure = KeyMaterial::try_from_item(
        symmetric.try_clone().expect("explicit secret-owner clone"),
        KeyFormatType::from_raw(7),
    )
    .expect("transparent structure matches its format");
    assert_eq!(structure.item_type(), ItemType::Structure);
    assert!(structure.with_raw(|value| matches!(value, ValueView::Structure(_))));
    assert!(KeyMaterial::try_from_item(symmetric, KeyFormatType::from_raw(1)).is_err());

    let wrong_tag =
        Item::new(tag(0x0042_0042), Value::byte_string(vec![1])).expect("valid wrong-tag item");
    assert!(KeyMaterial::try_from_item(wrong_tag, KeyFormatType::from_raw(1)).is_err());
    assert!(KeyMaterial::structure(7, Structure::new()).is_err());
    assert!(KeyMaterial::structure(1, Structure::new()).is_err());
}

#[test]
fn key_material_rejects_malformed_known_transparent_structures() {
    let empty = Item::new(tag(0x0042_0043), Value::structure(Structure::new()))
        .expect("valid empty Key Material structure");
    assert!(KeyMaterial::try_from_item(empty, KeyFormatType::from_raw(7)).is_err());
}

#[test]
fn oasis_table_461_ec_formats_use_hexadecimal_values() {
    // KMIP 2.1 §11.25, Table 461 assigns 0x14 and 0x15 to transparent EC
    // private and public keys. Empty known EC structures must be schema-checked.
    let empty_private = Item::new(tag(0x0042_0043), Value::structure(Structure::new()))
        .expect("valid empty Key Material structure");
    assert!(KeyMaterial::try_from_item(empty_private, KeyFormatType::from_raw(0x14)).is_err());

    let empty_public = Item::new(tag(0x0042_0043), Value::structure(Structure::new()))
        .expect("valid empty Key Material structure");
    assert!(KeyMaterial::try_from_item(empty_public, KeyFormatType::from_raw(0x15)).is_err());

    // Decimal 14 and 15 are 0x0E and 0x0F, reserved values. They remain open
    // generic formats and must not be dispatched to either EC schema.
    for reserved in [14, 15] {
        let generic = Item::new(tag(0x0042_0043), Value::structure(Structure::new()))
            .expect("valid generic Key Material structure");
        assert!(
            KeyMaterial::try_from_item(generic, KeyFormatType::from_raw(reserved)).is_ok(),
            "reserved raw format {reserved} must retain generic Structure handling"
        );
    }
}

#[test]
fn transparent_rsa_private_reader_requires_a_source_defined_private_component_set() {
    let modulus =
        Item::new(tag(0x0042_0052), Value::big_integer(vec![0x11])).expect("valid Modulus");
    let mut fields = Structure::new();
    fields.try_push(modulus).expect("valid RSA field");
    let material =
        Item::new(tag(0x0042_0043), Value::structure(fields)).expect("valid outer Key Material");
    assert!(kmipkit_protocol::TransparentRsaPrivateKey::try_from_item(material).is_err());
}

#[test]
fn known_transparent_structure_builder_rejects_missing_required_fields() {
    assert!(KeyMaterial::structure(7, Structure::new()).is_err());
}

#[test]
fn key_value_keeps_optional_attributes_after_required_material() {
    let value = kmipkit_protocol::KeyValue::try_new(
        KeyMaterial::byte_string(1, vec![0xA0]).unwrap(),
        Some(kmipkit_protocol::AttributeSet::new()),
    )
    .expect("valid Key Value");
    let item = value.into_item();
    let tags = item.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    });
    assert_eq!(tags, [0x0042_0043, 0x0042_0125]);
}

fn tag(raw: u32) -> kmipkit_ttlv::Tag {
    RawTag::new(raw)
        .expect("tag fits")
        .try_checked()
        .expect("assigned tag")
}
