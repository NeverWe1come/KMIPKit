//! Conversion contracts for the open, lossless managed-object API.

use kmipkit_protocol::{Certificate, ManagedObject, ObjectType};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw_tag)
            .expect("tag fits the KMIP 24-bit field")
            .try_checked()
            .expect("test tag is allocated"),
        value,
    )
    .expect("valid generic Item")
}

#[test]
fn malformed_known_object_has_payload_free_error() {
    let mut fields = Structure::new();
    fields
        .try_push(item(0x0042_001D, Value::enumeration(1)))
        .expect("valid enumeration field");
    fields
        .try_push(item(
            0x0042_001E,
            Value::byte_string(b"managed-object-secret-sentinel".to_vec()),
        ))
        .expect("valid byte string field");
    fields
        .try_push(item(
            0x0042_001E,
            Value::byte_string(b"duplicate-secret-sentinel".to_vec()),
        ))
        .expect("valid duplicate TTLV field");
    let malformed = item(0x0042_0013, Value::structure(fields));
    let error = Certificate::try_from_item(malformed)
        .expect_err("duplicate singleton fields violate Table 2");
    let diagnostics = format!("{error}; {error:?}");
    assert!(!diagnostics.contains("managed-object-secret-sentinel"));
    assert!(!diagnostics.contains("duplicate-secret-sentinel"));
}

#[test]
fn typed_and_generic_conversion_preserves_unknown_descendants_and_order() {
    let mut object = Structure::new();
    object
        .try_push(item(0x0042_001D, Value::enumeration(0x8000_0001)))
        .unwrap();
    object
        .try_push(item(0x0054_0101, Value::byte_string(vec![1, 2])))
        .unwrap();
    object
        .try_push(item(0x0042_001E, Value::byte_string(vec![3, 4])))
        .unwrap();
    let owned = item(0x0042_0013, Value::structure(object));
    let typed = Certificate::try_from_item(owned).expect("unknown child is preserved");
    let managed = ManagedObject::try_from_item(ObjectType::from_raw(1), typed.into_item())
        .expect("type and object tag match");
    let result = managed.into_item();
    let tags = result.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    });
    assert_eq!(tags, [0x0042_001D, 0x0054_0101, 0x0042_001E]);
}
