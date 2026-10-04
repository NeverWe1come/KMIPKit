//! Requirement traceability: KMIPKIT-0004-FR-010, FR-012, and SC-006.
//! Authority: approved KMIPKIT-0004 specification and Rust value-model contract.

use kmipkit_ttlv::{Item, RawTag, Value, ValueView};

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
