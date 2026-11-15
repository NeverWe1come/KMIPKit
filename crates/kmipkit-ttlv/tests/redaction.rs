//! Requirement traceability: KMIPKIT-0004-FR-009, FR-010, FR-013, and SC-004.
//! Authority: approved KMIPKIT-0004 specification and Rust value-model contract.

use std::fmt::{Binary, Debug, Display, LowerHex, Octal, UpperHex, Write as _};

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

fn numeric_sentinels<T>(value: T) -> Vec<String>
where
    T: Binary + Display + LowerHex + Octal + UpperHex,
{
    vec![
        value.to_string(),
        format!("{value:b}"),
        format!("{value:#b}"),
        format!("{value:o}"),
        format!("{value:#o}"),
        format!("{value:x}"),
        format!("{value:#x}"),
        format!("{value:X}"),
        format!("{value:#X}"),
    ]
}

fn byte_sentinels(bytes: &[u8]) -> Vec<String> {
    let mut lower_hex = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        write!(lower_hex, "{byte:02x}").expect("writing formatted bytes to a String cannot fail");
    }
    let upper_hex = lower_hex.to_uppercase();
    let spaced_lower_hex = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    let spaced_upper_hex = spaced_lower_hex.to_uppercase();

    vec![
        format!("{bytes:?}"),
        lower_hex.clone(),
        upper_hex.clone(),
        format!("0x{lower_hex}"),
        format!("0x{upper_hex}"),
        spaced_lower_hex,
        spaced_upper_hex,
    ]
}

#[test]
fn debug_redacts_direct_and_nested_payloads_on_every_public_surface() {
    let big_integer_bytes = vec![0xD1, 0xE2, 0xA3, 0xB4];
    let byte_string_bytes = vec![0x53, 0x45, 0x43, 0x52, 0x45, 0x54];
    let values = vec![
        (
            Value::integer(-1_234_567_890),
            numeric_sentinels(-1_234_567_890_i32),
        ),
        (
            Value::long_integer(-6_543_210_987_654_321),
            numeric_sentinels(-6_543_210_987_654_321_i64),
        ),
        (
            Value::big_integer(big_integer_bytes.clone()),
            byte_sentinels(&big_integer_bytes),
        ),
        (
            Value::enumeration(0xDEAD_BEEF),
            numeric_sentinels(0xDEAD_BEEF_u32),
        ),
        (Value::boolean(true), vec![String::from("true")]),
        (Value::boolean(false), vec![String::from("false")]),
        (
            Value::text_string(String::from("KMIPKIT_T016_TEXT_SENTINEL_71A9")),
            vec![String::from("KMIPKIT_T016_TEXT_SENTINEL_71A9")],
        ),
        (Value::byte_string(byte_string_bytes.clone()), {
            let mut sentinels = byte_sentinels(&byte_string_bytes);
            sentinels.push(String::from("SECRET"));
            sentinels
        }),
        (
            Value::date_time(9_876_543_210_987_654),
            numeric_sentinels(9_876_543_210_987_654_i64),
        ),
        (
            Value::interval(0xA1B2_C3D4),
            numeric_sentinels(0xA1B2_C3D4_u32),
        ),
        (
            Value::date_time_extended(-8_765_432_109_876_543),
            numeric_sentinels(-8_765_432_109_876_543_i64),
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
