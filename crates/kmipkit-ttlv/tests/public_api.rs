//! Requirement traceability: KMIPKIT-0004-FR-010, FR-012, and SC-006.
//! Authority: approved KMIPKIT-0004 specification and Rust value-model contract.

use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

#[test]
fn payload_types_reject_unapproved_automatic_traits_and_borrow_escape() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}

#[test]
fn caller_can_explicitly_copy_payloads_from_borrowed_views() {
    let tag = RawTag::new(0x0042_0173)
        .expect("the test tag fits the KMIP width")
        .try_checked()
        .expect("the test tag is assigned in KMIP 2.1");
    let text_item = Item::new(
        tag,
        Value::text_string(String::from("caller-copy-sentinel")),
    )
    .expect("the checked tag and value form an item");
    let copied_text = text_item.with_value(|view| match view {
        ValueView::TextString(text) => text.to_owned(),
        _ => String::new(),
    });

    let bytes_item = Item::new(tag, Value::byte_string(vec![0x31, 0x72, 0xA4]))
        .expect("the checked tag and value form an item");
    let copied_bytes = bytes_item.with_value(|view| match view {
        ValueView::ByteString(bytes) => bytes.to_vec(),
        _ => Vec::new(),
    });

    assert_eq!(copied_text, "caller-copy-sentinel");
    assert_eq!(copied_bytes, [0x31, 0x72, 0xA4]);
}

#[test]
fn item_exposes_a_borrowed_nested_structure_without_copying() {
    let parent_tag = RawTag::new(0x0042_0173)
        .expect("the test tag fits the KMIP width")
        .try_checked()
        .expect("the test tag is assigned in KMIP 2.1");
    let child_tag = RawTag::new(0x0054_1234)
        .expect("the vendor tag fits the KMIP width")
        .try_checked()
        .expect("the vendor tag is allocation-valid");
    let mut nested = Structure::new();
    nested
        .try_push(
            Item::new(child_tag, Value::integer(73))
                .expect("the checked tag and value form an item"),
        )
        .expect("a scalar child fits the Structure depth limit");
    let item = Item::new(parent_tag, Value::structure(nested))
        .expect("the checked tag and Structure form an item");

    let nested_view = item
        .as_structure()
        .expect("a Structure-valued item exposes its borrowed node");
    assert_eq!(nested_view.children().len(), 1);
    assert_eq!(nested_view.children()[0].tag(), child_tag);
    assert!(nested_view.children()[0]
        .with_value(|value| matches!(value, ValueView::Integer(number) if *number == 73)));

    let scalar = Item::new(parent_tag, Value::integer(1))
        .expect("the checked tag and value form an item");
    assert!(scalar.as_structure().is_none());
}
