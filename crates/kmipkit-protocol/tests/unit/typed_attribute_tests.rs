#![cfg(test)]

//! Typed attribute API contract from the KMIPKIT-0048 specification.

use crate::attribute::{Attribute, AttributeSet, CurrentAttribute, NewAttribute, typed};
use crate::typed_attributes::{UniqueIdentifierValue, UniqueIdentifierView};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

fn checked_tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("test tag fits the KMIP tag field")
        .try_checked()
        .expect("test tag is assigned or within the vendor extension range")
}

#[test]
fn generated_typed_attribute_registry_covers_every_direct_definition() {
    let definitions = crate::typed_attributes::ATTRIBUTE_DEFINITIONS;

    assert_eq!(definitions.len(), 62);
    assert!(definitions.iter().any(|definition| {
        definition.element_id == "KMIPKIT-ELEM-ATTRIBUTE-DIGITAL-SIGNATURE-ALGORITHM"
            && definition.source_multiplicity
                == Some("Yes for PGP keys. No for X.509 certificates.")
    }));
    assert!(
        definitions
            .iter()
            .all(|definition| definition.tag <= 0x00FF_FFFF)
    );
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(checked_tag(raw_tag), value).expect("test item uses a checked tag")
}

fn structure_with_children(count: usize) -> Structure {
    let mut structure = Structure::new();
    for index in 0..count {
        structure
            .try_push(item(
                0x0054_1234,
                Value::integer(i32::try_from(index).expect("small test index fits i32")),
            ))
            .expect("test Structure remains within limits");
    }
    structure
}

#[test]
fn typed_scalar_attribute_builds_and_borrows_its_value() {
    let attribute = Attribute::<typed::Comment>::new("private-note-sentinel".to_owned())
        .expect("Comment accepts a Text String");

    let observed = attribute
        .with_value(|value| value.to_owned())
        .expect("typed Comment view matches its source encoding");

    assert_eq!(observed, "private-note-sentinel");
    assert_eq!(attribute.as_item().tag().raw(), 0x0042_00FD);
    assert!(!format!("{attribute:?}").contains("private-note-sentinel"));
}

#[test]
fn typed_enumeration_preserves_unknown_raw_values() {
    let attribute = Attribute::<typed::CryptographicAlgorithm>::new(0xF123_4567)
        .expect("open Enumeration values remain constructible");

    let raw = attribute
        .with_value(|value| value)
        .expect("Enumeration has the catalogued TTLV type");

    assert_eq!(raw, 0xF123_4567);
}

#[test]
fn typed_attribute_set_keeps_repeated_structures_in_wire_order() {
    let mut attributes = AttributeSet::new();
    attributes
        .try_push_typed(
            Attribute::<typed::AlternativeName>::new(structure_with_children(1)).unwrap(),
        )
        .unwrap();
    attributes
        .try_push_typed(
            Attribute::<typed::AlternativeName>::new(structure_with_children(2)).unwrap(),
        )
        .unwrap();

    let child_counts = attributes
        .typed::<typed::AlternativeName>()
        .map(|attribute| {
            attribute
                .with_value(|structure| structure.children().len())
                .expect("Alternative Name is a Structure")
        })
        .collect::<Vec<_>>();

    assert_eq!(child_counts, [1, 2]);
}

#[test]
fn typed_attribute_set_preserves_singleton_duplicates_for_operation_validation() {
    let mut attributes = AttributeSet::new();
    attributes
        .try_push_typed(Attribute::<typed::ActivationDate>::new(1).unwrap())
        .unwrap();

    attributes
        .try_push_typed(Attribute::<typed::ActivationDate>::new(2).unwrap())
        .unwrap();

    assert_eq!(attributes.typed::<typed::ActivationDate>().count(), 2);
    assert!(attributes.validate_single_instance_cardinality().is_err());
}

#[test]
fn unique_identifier_preserves_each_source_permitted_value_kind() {
    let cases = [
        UniqueIdentifierValue::TextString("id-text".to_owned()),
        UniqueIdentifierValue::Enumeration(0xF123_4567),
        UniqueIdentifierValue::Integer(-17),
    ];

    for case in cases {
        let expected = case.clone();
        let attribute = Attribute::<typed::UniqueIdentifier>::new(case)
            .expect("Unique Identifier accepts its source-permitted alternatives");
        let observed = attribute
            .with_value(|value| match value {
                UniqueIdentifierView::TextString(text) => {
                    UniqueIdentifierValue::TextString(text.to_owned())
                }
                UniqueIdentifierView::Enumeration(raw) => UniqueIdentifierValue::Enumeration(raw),
                UniqueIdentifierView::Integer(raw) => UniqueIdentifierValue::Integer(raw),
            })
            .expect("Unique Identifier value view matches the encoded alternative");
        assert_eq!(observed, expected);
    }
}

#[test]
fn typed_current_and_new_wrappers_keep_generic_construction_available() {
    let typed = Attribute::<typed::Comment>::new("replacement".to_owned()).unwrap();
    let current = CurrentAttribute::from_typed(typed);
    let current_value = current
        .typed::<typed::Comment>()
        .unwrap()
        .with_value(str::to_owned)
        .unwrap();
    assert_eq!(current_value, "replacement");

    let new =
        NewAttribute::from_typed(Attribute::<typed::Comment>::new("new-value".to_owned()).unwrap());
    let new_value = new
        .typed::<typed::Comment>()
        .unwrap()
        .with_value(str::to_owned)
        .unwrap();
    assert_eq!(new_value, "new-value");
}

#[test]
fn generic_wrapper_typed_mismatch_is_redacted() {
    let current = CurrentAttribute::new(item(
        0x0042_0022,
        Value::byte_string(b"do-not-print-this-value".to_vec()),
    ));

    let error = current.typed::<typed::Comment>().unwrap_err();

    assert!(!format!("{error:?}").contains("do-not-print-this-value"));
    assert!(!format!("{error}").contains("do-not-print-this-value"));
}

#[test]
fn typed_structure_view_preserves_unknown_child_tags() {
    let attribute = Attribute::<typed::Name>::new(structure_with_children(1)).unwrap();

    let observed = attribute
        .with_value(|structure| structure.children()[0].tag().raw())
        .unwrap();

    assert_eq!(observed, 0x0054_1234);
}
