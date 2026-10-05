//! Requirement traceability: KMIPKIT-0006-FR-017 and SC-006.
//!
//! These tests cover a borrowed in-memory view only. They do not assert wire
//! encoding or OASIS schema validity.

use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

#[test]
fn public_view_preserves_child_insertion_order() {
    let tag = RawTag::new(0x0042_0173)
        .expect("the test tag fits the KMIP width")
        .try_checked()
        .expect("the test tag is assigned in KMIP 2.1");
    let mut structure = Structure::new();
    for number in [17, 29] {
        structure
            .try_push(
                Item::new(tag, Value::integer(number))
                    .expect("the checked tag and value form an item"),
            )
            .expect("a flat item fits the Structure depth limit");
    }

    let view = structure.view();
    let observed: Vec<i32> = view
        .children()
        .iter()
        .map(|item| {
            item.with_value(|value| match value {
                ValueView::Integer(number) => *number,
                _ => panic!("the test child must remain an Integer"),
            })
        })
        .collect();

    assert_eq!(observed, [17, 29]);
}

#[test]
fn public_view_exposes_the_original_child_items() {
    let tag = RawTag::new(0x0042_0173)
        .expect("the test tag fits the KMIP width")
        .try_checked()
        .expect("the test tag is assigned in KMIP 2.1");
    let mut structure = Structure::new();
    structure
        .try_push(
            Item::new(tag, Value::integer(17)).expect("the checked tag and value form an item"),
        )
        .expect("a flat item fits the Structure depth limit");

    let view = structure.view();

    assert_eq!(view.children()[0].tag(), tag);
}
