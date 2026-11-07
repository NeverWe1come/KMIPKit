use kmipkit_ttlv::{ItemType, RawTag, Structure, Value, ValueView};

use crate::{Certificate, KeyFormatType, ManagedValueKind, ManagedValueReader, ObjectType};

fn tag(raw: u32) -> kmipkit_ttlv::Tag {
    RawTag::new(raw)
        .expect("fixture tag fits")
        .try_checked()
        .expect("fixture tag is allocated")
}

#[test]
fn typed_reader_keeps_open_context_repeats_and_source_order() {
    let field_tag = 0x0054_0001;
    let mut structure = Structure::new();
    structure
        .try_push(
            kmipkit_ttlv::Item::new(tag(field_tag), Value::enumeration(0x8000_0101))
                .expect("first vendor value"),
        )
        .expect("first vendor field fits");
    structure
        .try_push(
            kmipkit_ttlv::Item::new(tag(field_tag), Value::enumeration(0x8000_0102))
                .expect("repeated vendor value"),
        )
        .expect("repeated vendor field fits");
    let item = kmipkit_ttlv::Item::new(tag(0x0054_0002), Value::structure(structure))
        .expect("vendor object");
    let reader = ManagedValueReader::try_from_item(
        ManagedValueKind::ManagedObject(ObjectType::from_raw(0x8000_0001)),
        item,
    )
    .expect("future object context stays open");

    assert!(reader.has_field(field_tag));
    assert_eq!(reader.field_type(field_tag), Some(ItemType::Enumeration));
    assert_eq!(reader.field_enumeration(field_tag), Some(0x8000_0101));
    assert_eq!(reader.field_integer(field_tag), None);

    let item = reader.into_item();
    item.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            panic!("vendor object remains a Structure")
        };
        assert_eq!(
            structure
                .children()
                .iter()
                .map(|child| child.tag().raw())
                .collect::<Vec<_>>(),
            [field_tag, field_tag]
        );
    });
}

#[test]
fn typed_reader_zeroizes_and_redacts_copied_secret_bytes() {
    let item = Certificate::try_new(0x8000_0042, b"secret-canary".to_vec())
        .expect("certificate")
        .into_item();
    let reader = ManagedValueReader::try_from_item(
        ManagedValueKind::ManagedObject(ObjectType::from_raw(1)),
        item,
    )
    .expect("certificate model validates");
    let value = reader
        .read_field(0x0042_001e)
        .expect("certificate bytes are present");
    assert_eq!(value.item_type(), ItemType::ByteString);
    assert_eq!(value.bytes(), Some(b"secret-canary".as_slice()));
    assert!(!format!("{value:?}").contains("secret-canary"));
    assert!(!format!("{reader:?}").contains("secret-canary"));
    assert!(!reader.has_field(0x0054_0001));
}

#[test]
fn typed_reader_returns_owned_nested_item_with_repeated_vendor_children() {
    let vendor_tag = 0x0054_0001;
    let mut attributes = Structure::new();
    for value in [b"first".as_slice(), b"second".as_slice()] {
        attributes
            .try_push(
                kmipkit_ttlv::Item::new(tag(vendor_tag), Value::byte_string(value.to_vec()))
                    .expect("vendor child"),
            )
            .expect("vendor child fits");
    }
    let attributes = kmipkit_ttlv::Item::new(tag(0x0042_0125), Value::structure(attributes))
        .expect("Attributes");
    let mut key_value = Structure::new();
    key_value
        .try_push(
            crate::KeyMaterial::byte_string(1, b"key-data".to_vec())
                .expect("Key Material")
                .into_item(),
        )
        .expect("Key Material child fits");
    key_value
        .try_push(attributes)
        .expect("Attributes child fits");
    let item =
        kmipkit_ttlv::Item::new(tag(0x0042_0045), Value::structure(key_value)).expect("Key Value");
    let reader = ManagedValueReader::try_from_item(
        ManagedValueKind::KeyValue(KeyFormatType::from_raw(1)),
        item,
    )
    .expect("Key Value validates");

    let nested = reader
        .read_field_item(0x0042_0125)
        .expect("nested item clone succeeds")
        .expect("Attributes is present");
    assert_eq!(nested.tag().raw(), 0x0042_0125);
    nested.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            panic!("Attributes remains a Structure")
        };
        assert_eq!(
            structure
                .children()
                .iter()
                .map(|child| child.tag().raw())
                .collect::<Vec<_>>(),
            [vendor_tag, vendor_tag]
        );
    });
    assert!(
        reader
            .read_field_item(0x0054_0002)
            .expect("missing lookup succeeds")
            .is_none()
    );
}

#[test]
fn typed_reader_rejects_known_object_mismatch_and_preserves_future_format() {
    let certificate = Certificate::try_new(1, vec![1])
        .expect("certificate")
        .into_item();
    assert!(
        ManagedValueReader::try_from_item(
            ManagedValueKind::ManagedObject(ObjectType::from_raw(8)),
            certificate,
        )
        .is_err()
    );

    let key_material = crate::KeyMaterial::byte_string(0x8000_0001, b"future".to_vec())
        .expect("open Key Format Type")
        .into_item();
    let reader = ManagedValueReader::try_from_item(
        ManagedValueKind::KeyMaterial(KeyFormatType::from_raw(0x8000_0001)),
        key_material,
    )
    .expect("future byte-string format remains open");
    assert_eq!(
        reader.kind(),
        ManagedValueKind::KeyMaterial(KeyFormatType::from_raw(0x8000_0001))
    );
    assert_eq!(reader.field_type(0x0042_0043), None);
    assert_eq!(reader.into_item().item_type(), ItemType::ByteString);
}
