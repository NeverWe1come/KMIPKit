//! Requirement traceability: KMIPKIT-0004-FR-012 and SC-006.
//! Authority: approved KMIPKIT-0004 specification and Rust value-model contract.

use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

fn checked_tag() -> Tag {
    RawTag::new(0x0042_0173)
        .expect("the test tag fits the KMIP width")
        .try_checked()
        .expect("the test tag is assigned in KMIP 2.1")
}

fn integer_payload_address(item: &Item) -> *const i32 {
    item.with_value(|view| match view {
        ValueView::Integer(value) => value as *const i32,
        _ => std::ptr::null(),
    })
}

#[test]
fn payload_address_stays_stable_when_the_containing_structure_grows() {
    let tag = checked_tag();
    let original = Item::new(tag, Value::integer(0x1357_2468))
        .expect("the checked tag and value form an item");
    let address_before_growth = integer_payload_address(&original);
    let mut structure = Structure::new();
    structure
        .try_push(original)
        .expect("the first child fits the nesting limit");

    for value in 0..1_024 {
        let item =
            Item::new(tag, Value::integer(value)).expect("the checked tag and value form an item");
        structure
            .try_push(item)
            .expect("integer children do not deepen the structure");
    }

    let root = Item::new(tag, Value::structure(structure))
        .expect("the checked tag and value form an item");
    let address_after_growth = root.with_value(|view| match view {
        ValueView::Structure(structure) => structure
            .children()
            .first()
            .map_or(std::ptr::null(), integer_payload_address),
        _ => std::ptr::null(),
    });

    // Pointer identity is compared only; the pointer is never dereferenced.
    assert!(std::ptr::eq(address_before_growth, address_after_growth));
}
