//! Requirement traceability: KMIPKIT-0004-FR-009, FR-010, FR-013, and SC-004.
//! Authority: approved KMIPKIT-0004 specification and Rust value-model contract.

use std::fmt::Debug;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, Tag, Value, ValueView};

fn checked_tag() -> Tag {
    RawTag::new(0x0042_0173)
        .expect("the test tag fits the KMIP width")
        .try_checked()
        .expect("the test tag is assigned in KMIP 2.1")
}

fn debug(value: &impl Debug) -> String {
    format!("{value:?}")
}

fn assert_sentinels_absent(formatted: &[String], sentinels: &[String]) {
    for output in formatted {
        for sentinel in sentinels {
            assert!(
                !output.contains(sentinel),
                "formatted output contained a payload sentinel"
            );
        }
    }
}

#[test]
fn debug_redacts_direct_and_nested_payloads_on_every_public_surface() {
    let big_integer_bytes = vec![0xD1, 0xE2, 0xA3, 0xB4];
    let byte_string_bytes = vec![0x53, 0x45, 0x43, 0x52, 0x45, 0x54];
    let values = vec![
        (
            Value::integer(-1_234_567_890),
            vec![String::from("-1234567890")],
        ),
        (
            Value::long_integer(-6_543_210_987_654_321),
            vec![String::from("-6543210987654321")],
        ),
        (
            Value::big_integer(big_integer_bytes.clone()),
            vec![format!("{big_integer_bytes:?}")],
        ),
        (
            Value::enumeration(0xDEAD_BEEF),
            vec![String::from("3735928559")],
        ),
        (Value::boolean(true), vec![String::from("true")]),
        (
            Value::text_string(String::from("KMIPKIT_T016_TEXT_SENTINEL_71A9")),
            vec![String::from("KMIPKIT_T016_TEXT_SENTINEL_71A9")],
        ),
        (
            Value::byte_string(byte_string_bytes.clone()),
            vec![format!("{byte_string_bytes:?}")],
        ),
        (
            Value::date_time(9_876_543_210_987_654),
            vec![String::from("9876543210987654")],
        ),
        (
            Value::interval(0xA1B2_C3D4),
            vec![String::from("2712847316")],
        ),
        (
            Value::date_time_extended(-8_765_432_109_876_543),
            vec![String::from("-8765432109876543")],
        ),
    ];

    let mut sentinels = Vec::new();
    let mut formatted = Vec::new();
    let tag = checked_tag();
    let mut nested = Structure::new();

    for (value, value_sentinels) in values {
        formatted.push(debug(&value));
        let item = Item::new(tag, value).expect("the checked tag and value form an item");
        formatted.push(debug(&item));
        formatted.push(item.with_value(|view| debug(&view)));
        nested
            .try_push(item)
            .expect("leaf values do not deepen the structure");
        sentinels.extend(value_sentinels);
    }

    formatted.push(debug(&nested));
    let nested_value = Value::structure(nested);
    formatted.push(debug(&nested_value));
    let root = Item::new(tag, nested_value).expect("the checked tag and value form an item");
    formatted.push(debug(&root));
    let (value_view_debug, structure_view_debug, child_debug) = root.with_value(|view| {
        let view_debug = debug(&view);
        match view {
            ValueView::Structure(structure_view) => {
                let structure_debug = debug(&structure_view);
                let child_debug = structure_view
                    .children()
                    .first()
                    .map_or_else(String::new, debug);
                (view_debug, structure_debug, child_debug)
            }
            _ => (view_debug, String::new(), String::new()),
        }
    });
    formatted.push(value_view_debug);
    formatted.push(structure_view_debug);
    formatted.push(child_debug);

    assert_sentinels_absent(&formatted, &sentinels);
}

#[test]
fn model_error_debug_and_display_do_not_retain_payload_sentinels() {
    let sentinel = String::from("KMIPKIT_T016_ERROR_SENTINEL_93C2");
    let tag = checked_tag();
    let mut nested = Structure::new();
    let leaf = Item::new(tag, Value::text_string(sentinel.clone()))
        .expect("the checked tag and value form an item");
    nested
        .try_push(leaf)
        .expect("the text leaf does not deepen the structure");

    for _ in 0..63 {
        let mut parent = Structure::new();
        let child = Item::new(tag, Value::structure(nested))
            .expect("the checked tag and value form an item");
        parent
            .try_push(child)
            .expect("the nested tree remains within the depth bound");
        nested = parent;
    }

    let raw_tag_error: ModelError = RawTag::new(u32::MAX).expect_err("the raw tag exceeds 24 bits");
    let mut outer = Structure::new();
    let depth_error = outer
        .try_push(
            Item::new(tag, Value::structure(nested))
                .expect("the checked tag and value form an item"),
        )
        .expect_err("the candidate tree exceeds the local depth bound");

    for error in [raw_tag_error, depth_error] {
        assert!(!debug(&error).contains(&sentinel));
        assert!(!error.to_string().contains(&sentinel));
    }
}
