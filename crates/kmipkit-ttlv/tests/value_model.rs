//! OASIS KMIP Specification v2.1: §10.1.1 (Tag), §10.1.2 (Type), §11.23 (Item
//! Type Enumeration), §11.56 (Tag Enumeration), §12.1 (Cryptographic Usage
//! Mask), §12.2 (Protection Storage Mask), and §12.3 (Storage Status Mask).
//! These tests cover the in-memory semantic model only, not TTLV wire encoding
//! or schema validity.
//!
//! Traceability: KMIPKIT-0004-FR-001–FR-008, KMIPKIT-0004-FR-010, and
//! KMIPKIT-0004-NR-001–NR-007.

use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};
use std::fmt::{Debug, Display};

fn checked_tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the test tag must fit in 24 bits")
        .try_checked()
        .expect("the test tag must be allocated by KMIP 2.1")
}

fn assert_derived_item_type(value: Value, expected: ItemType) {
    let item = Item::new(checked_tag(0x0042_0173), value)
        .expect("a checked tag and value must construct an item");

    assert_eq!(item.item_type(), expected);
}

fn assert_error_does_not_reveal_raw_tag(error: &(impl Debug + Display), raw: u32) {
    let debug = format!("{error:?}");
    let display = error.to_string();
    let representations = [
        format!("0x{raw:08X}"),
        format!("0x{raw:08x}"),
        format!("{raw:#x}"),
        raw.to_string(),
    ];

    for rejected_tag in representations {
        assert!(!debug.contains(&rejected_tag));
        assert!(!display.contains(&rejected_tag));
    }
}

#[test]
fn every_value_constructor_derives_its_item_type() {
    assert_derived_item_type(Value::structure(Structure::new()), ItemType::Structure);
    assert_derived_item_type(Value::integer(i32::MIN), ItemType::Integer);
    assert_derived_item_type(Value::long_integer(i64::MIN), ItemType::LongInteger);
    assert_derived_item_type(Value::big_integer(vec![0x00]), ItemType::BigInteger);
    assert_derived_item_type(Value::enumeration(u32::MAX), ItemType::Enumeration);
    assert_derived_item_type(Value::boolean(true), ItemType::Boolean);
    assert_derived_item_type(
        Value::text_string(String::from("text")),
        ItemType::TextString,
    );
    assert_derived_item_type(Value::byte_string(vec![0xA5]), ItemType::ByteString);
    assert_derived_item_type(Value::date_time(i64::MAX), ItemType::DateTime);
    assert_derived_item_type(Value::interval(u32::MAX), ItemType::Interval);
    assert_derived_item_type(
        Value::date_time_extended(i64::MIN),
        ItemType::DateTimeExtended,
    );
}

#[test]
fn integer_preserves_signed_boundaries_and_high_bit_patterns() {
    for expected in [i32::MIN, -1, 0, i32::MAX, 0x8000_0105_u32 as i32] {
        let item = Item::new(checked_tag(0x0042_0173), Value::integer(expected))
            .expect("a checked tag and Integer value must construct an item");

        item.with_value(|view| {
            assert!(matches!(view, ValueView::Integer(actual) if *actual == expected));
        });
    }
}

#[test]
fn long_integer_preserves_signed_boundaries() {
    for expected in [i64::MIN, -1, 0, i64::MAX] {
        let item = Item::new(checked_tag(0x0042_0173), Value::long_integer(expected))
            .expect("a checked tag and Long Integer value must construct an item");

        item.with_value(|view| {
            assert!(matches!(view, ValueView::LongInteger(actual) if *actual == expected));
        });
    }
}

#[test]
fn enumeration_preserves_unsigned_boundaries_and_unknown_high_bits() {
    for expected in [0, 1, 0x8000_0001, 0xDEAD_BEEF, u32::MAX] {
        let item = Item::new(checked_tag(0x0042_0173), Value::enumeration(expected))
            .expect("a checked tag and Enumeration value must construct an item");

        item.with_value(|view| {
            assert!(matches!(view, ValueView::Enumeration(actual) if *actual == expected));
        });
    }
}

#[test]
fn boolean_preserves_both_values() {
    for expected in [false, true] {
        let item = Item::new(checked_tag(0x0042_0173), Value::boolean(expected))
            .expect("a checked tag and Boolean value must construct an item");

        item.with_value(|view| {
            assert!(matches!(view, ValueView::Boolean(actual) if *actual == expected));
        });
    }
}

#[test]
fn date_time_preserves_signed_i64_boundaries() {
    for expected in [i64::MIN, -1, 0, i64::MAX] {
        let item = Item::new(checked_tag(0x0042_0173), Value::date_time(expected))
            .expect("a checked tag and Date Time value must construct an item");

        item.with_value(|view| {
            assert!(matches!(view, ValueView::DateTime(actual) if *actual == expected));
        });
    }
}

#[test]
fn date_time_extended_preserves_signed_i64_boundaries() {
    for expected in [i64::MIN, -1, 0, i64::MAX] {
        let item = Item::new(
            checked_tag(0x0042_0173),
            Value::date_time_extended(expected),
        )
        .expect("a checked tag and Date Time Extended value must construct an item");

        item.with_value(|view| {
            assert!(matches!(
                view,
                ValueView::DateTimeExtended(actual) if *actual == expected
            ));
        });
    }
}

#[test]
fn interval_preserves_unsigned_boundaries() {
    for expected in [0, 1, 0x8000_0001, u32::MAX] {
        let item = Item::new(checked_tag(0x0042_0173), Value::interval(expected))
            .expect("a checked tag and Interval value must construct an item");

        item.with_value(|view| {
            assert!(matches!(view, ValueView::Interval(actual) if *actual == expected));
        });
    }
}

#[test]
fn text_string_preserves_non_ascii_unicode_without_normalization() {
    let composed = String::from("é");
    let decomposed = String::from("e\u{301}");
    let other_unicode = String::from("雪🙂");

    for expected in [composed, decomposed, other_unicode] {
        let item = Item::new(
            checked_tag(0x0042_0173),
            Value::text_string(expected.clone()),
        )
        .expect("a checked tag and Text String value must construct an item");

        item.with_value(|view| {
            assert!(matches!(view, ValueView::TextString(actual) if actual == expected.as_str()));
        });
    }
}

#[test]
fn byte_string_preserves_arbitrary_bytes_and_empty_values() {
    for expected in [vec![0x00, 0x80, 0xFF, 0x41], Vec::new()] {
        let item = Item::new(
            checked_tag(0x0042_0173),
            Value::byte_string(expected.clone()),
        )
        .expect("a checked tag and Byte String value must construct an item");

        item.with_value(|view| {
            assert!(matches!(view, ValueView::ByteString(actual) if actual == expected.as_slice()));
        });
    }
}

#[test]
fn big_integer_preserves_exact_item_value_octets() {
    for expected in [
        vec![0x00, 0x00, 0x80, 0x01],
        vec![0xFF, 0xFF, 0x7F],
        Vec::new(),
    ] {
        let item = Item::new(
            checked_tag(0x0042_0173),
            Value::big_integer(expected.clone()),
        )
        .expect("a checked tag and Big Integer value must construct an item");

        item.with_value(|view| {
            assert!(matches!(view, ValueView::BigInteger(actual) if actual == expected.as_slice()));
        });
    }
}

#[test]
fn structure_preserves_caller_order_repeated_tags_and_child_values() {
    let first_tag = 0x0042_0174;
    let repeated_tag = 0x0042_0173;
    let mut structure = Structure::new();
    structure
        .try_push(
            Item::new(
                checked_tag(first_tag),
                Value::text_string(String::from("first")),
            )
            .expect("the first checked child must construct"),
        )
        .expect("the first child must append");
    structure
        .try_push(
            Item::new(checked_tag(repeated_tag), Value::integer(-37))
                .expect("the second checked child must construct"),
        )
        .expect("the second child must append");
    structure
        .try_push(
            Item::new(checked_tag(first_tag), Value::byte_string(vec![0xC3, 0x28]))
                .expect("the third checked child must construct"),
        )
        .expect("the repeated-tag child must append");

    let parent = Item::new(checked_tag(0x0042_0175), Value::structure(structure))
        .expect("a checked tag and Structure value must construct an item");

    parent.with_value(|view| {
        let ValueView::Structure(structure_view) = view else {
            panic!("the constructed Structure must remain a Structure");
        };
        let children = structure_view.children();

        assert_eq!(children.len(), 3);
        assert_eq!(children[0].tag().raw(), first_tag);
        assert_eq!(children[1].tag().raw(), repeated_tag);
        assert_eq!(children[2].tag().raw(), first_tag);
        assert!(
            children[0].with_value(
                |child| matches!(child, ValueView::TextString(text) if text == "first")
            )
        );
        assert!(
            children[1].with_value(
                |child| matches!(child, ValueView::Integer(integer) if *integer == -37)
            )
        );
        assert!(children[2].with_value(
            |child| matches!(child, ValueView::ByteString(bytes) if bytes == &[0xC3, 0x28])
        ));
    });
}

#[test]
fn out_of_width_raw_tag_error_does_not_format_rejected_value() {
    let rejected_tag = 0x01AB_CDEF;
    let error = RawTag::new(rejected_tag).expect_err("a 25-bit tag must be rejected");

    assert_error_does_not_reveal_raw_tag(&error, rejected_tag);
}

#[test]
fn reserved_tag_error_does_not_format_rejected_value() {
    let rejected_tag = 0x0042_0009;
    let raw_tag = RawTag::new(rejected_tag).expect("the reserved tag fits in 24 bits");
    let error = raw_tag
        .try_checked()
        .expect_err("a Reserved tag must be rejected by allocation checking");

    assert_error_does_not_reveal_raw_tag(&error, rejected_tag);
}
