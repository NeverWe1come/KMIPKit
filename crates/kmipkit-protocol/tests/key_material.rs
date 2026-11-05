//! OASIS KMIP 2.1 §3.2, Table 13: Key Material encoding and fallback.

use kmipkit_protocol::{KeyFormatType, KeyMaterial};
use kmipkit_ttlv::{ItemType, ValueView};

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
