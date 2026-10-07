//! Deep-copy behavior for the opaque TTLV value model.
//!
//! Traceability: KMIPKIT-0012-FR-007 preserves generic extension subtrees;
//! OASIS KMIP v2.1 §9.13, Table 418 defines the enclosing Vendor Extension.

use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Value, ValueView, try_clone_value};

fn tag() -> kmipkit_ttlv::Tag {
    RawTag::new(0x0042_0173)
        .expect("the clone fixture Tag fits in the wire field")
        .try_checked()
        .expect("the clone fixture Tag is allocated by KMIP 2.1")
}

fn assert_views_equal(left: ValueView<'_>, right: ValueView<'_>) {
    match (left, right) {
        (ValueView::Structure(left), ValueView::Structure(right)) => {
            assert_eq!(left.children().len(), right.children().len());
            for (left_item, right_item) in left.children().iter().zip(right.children()) {
                assert_eq!(left_item.tag(), right_item.tag());
                left_item.with_value(|left_value| {
                    right_item
                        .with_value(|right_value| assert_views_equal(left_value, right_value));
                });
            }
        }
        (ValueView::Integer(left), ValueView::Integer(right)) => assert_eq!(left, right),
        (ValueView::LongInteger(left), ValueView::LongInteger(right)) => assert_eq!(left, right),
        (ValueView::BigInteger(left), ValueView::BigInteger(right)) => assert_eq!(left, right),
        (ValueView::Enumeration(left), ValueView::Enumeration(right)) => assert_eq!(left, right),
        (ValueView::Boolean(left), ValueView::Boolean(right)) => assert_eq!(left, right),
        (ValueView::TextString(left), ValueView::TextString(right)) => assert_eq!(left, right),
        (ValueView::ByteString(left), ValueView::ByteString(right)) => assert_eq!(left, right),
        (ValueView::DateTime(left), ValueView::DateTime(right)) => assert_eq!(left, right),
        (ValueView::Interval(left), ValueView::Interval(right)) => assert_eq!(left, right),
        (ValueView::DateTimeExtended(left), ValueView::DateTimeExtended(right)) => {
            assert_eq!(left, right);
        }
        _ => panic!("a cloned TTLV value retains its Item Type"),
    }
}

#[test]
fn deep_clone_preserves_every_scalar_variant_and_nested_child_order() {
    let mut nested = Structure::default();
    nested
        .try_push(
            Item::new(tag(), Value::text_string("nested-value".to_owned()))
                .expect("a checked Tag and Text String form an Item"),
        )
        .expect("the nested Structure is within the depth limit");

    let values = [
        Value::structure(nested),
        Value::integer(-17),
        Value::long_integer(-9_000_000_001),
        Value::big_integer(vec![0x00, 0x80, 0x01]),
        Value::enumeration(0xDEAD_BEEF),
        Value::boolean(true),
        Value::text_string("vendor text".to_owned()),
        Value::byte_string(vec![0x00, 0x7F, 0xFF]),
        Value::date_time(-1_700_000_000),
        Value::interval(u32::MAX),
        Value::date_time_extended(i64::MIN),
    ];
    let expected_types = [
        ItemType::Structure,
        ItemType::Integer,
        ItemType::LongInteger,
        ItemType::BigInteger,
        ItemType::Enumeration,
        ItemType::Boolean,
        ItemType::TextString,
        ItemType::ByteString,
        ItemType::DateTime,
        ItemType::Interval,
        ItemType::DateTimeExtended,
    ];

    for (value, expected_type) in values.iter().zip(expected_types) {
        let cloned = try_clone_value(value).expect("every represented TTLV value can be cloned");
        assert_eq!(cloned.item_type(), expected_type);
        value.with_value(|left| cloned.with_value(|right| assert_views_equal(left, right)));
    }
}
