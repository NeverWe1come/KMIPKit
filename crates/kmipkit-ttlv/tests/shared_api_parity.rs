use kmipkit_ttlv::codec::{CodecLimits, LimitsError};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};
use serde_json::Value as JsonValue;

const SHARED_FIXTURES: &str =
    include_str!("../../../tests/fixtures/api/shared-capability-parity.json");

fn shared_case(id: &str) -> JsonValue {
    let fixture: JsonValue =
        serde_json::from_str(SHARED_FIXTURES).expect("shared parity fixture is valid JSON");
    fixture["cases"]
        .as_array()
        .expect("fixture cases are an array")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("shared parity case {id} exists"))
        .clone()
}

fn hex_bytes(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0, "hex byte strings have even length");
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty(), "hex byte strings have whole bytes");
    pairs
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("hex digit pair is UTF-8");
            u8::from_str_radix(text, 16).expect("fixture byte string contains hex digits")
        })
        .collect()
}

fn limit_field(value: &JsonValue, field: &str) -> usize {
    usize::try_from(value[field].as_u64().expect("limit is an unsigned integer"))
        .expect("limit fits usize")
}

#[test]
fn generic_ttlv_shared_fixture_preserves_open_values_and_child_order() {
    // KMIPKit generic model requirements are traced in the shared fixture:
    // KMIPKIT-0043-FR-011, KMIPKIT-0004-FR-004, and KMIPKIT-0004-FR-006.
    let case = shared_case("ttlv.open-values-and-child-order.v1");
    let source_children = case["input"]["structure"]["children"]
        .as_array()
        .expect("fixture Structure children are an array");
    let expected = &case["expectedOutput"];
    let expected_bytes = hex_bytes(
        expected["byteStringHex"]
            .as_str()
            .expect("expected byte string is hexadecimal text"),
    );
    let mut structure = Structure::new();

    for child in source_children {
        let raw_tag_text = child["tag"].as_str().expect("fixture child tag is text");
        let raw_tag = u32::from_str_radix(
            raw_tag_text
                .strip_prefix("0x")
                .expect("fixture tag uses 0x prefix"),
            16,
        )
        .expect("fixture tag is hexadecimal");
        let tag = RawTag::new(raw_tag)
            .expect("fixture tag fits 24-bit wire tag")
            .try_checked()
            .expect("fixture tag is allocated");
        let value = match child["type"].as_str().expect("fixture item type is text") {
            "Enumeration" => Value::enumeration(
                u32::try_from(child["value"].as_u64().expect("enum value is unsigned"))
                    .expect("enum value fits U32"),
            ),
            "ByteString" => Value::byte_string(hex_bytes(
                child["valueHex"]
                    .as_str()
                    .expect("byte string value is hexadecimal text"),
            )),
            other => panic!("unsupported fixture value type {other}"),
        };
        structure
            .try_push(Item::new(tag, value).expect("fixture child is a valid generic item"))
            .expect("fixture Structure is within generic model depth");
    }

    let view = structure.view();
    let actual_tags: Vec<String> = view
        .children()
        .iter()
        .map(|item| format!("0x{:06x}", item.tag().raw()))
        .collect();
    let expected_tags = expected["childTagsInOrder"]
        .as_array()
        .expect("expected ordered tags are an array")
        .iter()
        .map(|tag| tag.as_str().expect("expected tag is text").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(actual_tags, expected_tags);

    let first = &view.children()[0];
    let expected_enumeration = u32::try_from(
        expected["enumerationValue"]
            .as_u64()
            .expect("expected enum is unsigned"),
    )
    .expect("expected enum fits U32");
    assert!(first.with_value(
        |value| matches!(value, ValueView::Enumeration(actual) if *actual == expected_enumeration)
    ));
    let second = &view.children()[1];
    assert!(second.with_value(
        |value| matches!(value, ValueView::ByteString(actual) if actual == expected_bytes.as_slice())
    ));
    assert_eq!(source_children.len(), expected_tags.len());
    assert!(
        expected["unknownValuesPreserved"]
            .as_bool()
            .expect("unknown-value expectation is boolean")
    );
}

#[test]
fn codec_limits_shared_fixture_covers_defaults_ranges_and_rejection() {
    // KMIPKit resource limits are traced in the fixture to KMIPKIT-0005-FR-007.
    let case = shared_case("codec-limits.defaults-and-ranges.v1");
    assert!(
        case["input"]["defaults"]
            .as_bool()
            .expect("defaults input selector is boolean")
    );
    let defaults = CodecLimits::defaults();
    let configured_input = &case["input"]["configured"];
    let configured = CodecLimits::new(
        limit_field(configured_input, "maxMessageBytes"),
        limit_field(configured_input, "maxStructureDepth"),
        limit_field(configured_input, "maxElements"),
    )
    .expect("configured fixture depth is in range");
    let expected = &case["expectedOutput"];
    let expected_defaults = &expected["defaults"];
    let expected_configured = &expected["configured"];

    assert_eq!(
        defaults.max_message_bytes(),
        limit_field(expected_defaults, "maxMessageBytes")
    );
    assert_eq!(
        defaults.max_structure_depth(),
        limit_field(expected_defaults, "maxStructureDepth")
    );
    assert_eq!(
        defaults.max_elements(),
        limit_field(expected_defaults, "maxElements")
    );
    assert_eq!(
        configured.max_message_bytes(),
        limit_field(expected_configured, "maxMessageBytes")
    );
    assert_eq!(
        configured.max_structure_depth(),
        limit_field(expected_configured, "maxStructureDepth")
    );
    assert_eq!(
        configured.max_elements(),
        limit_field(expected_configured, "maxElements")
    );

    let invalid_depth = limit_field(&case["input"], "invalidDepth");
    let rejected = CodecLimits::new(
        defaults.max_message_bytes(),
        invalid_depth,
        defaults.max_elements(),
    );
    assert_eq!(
        expected["invalidDepth"]["value"].as_u64(),
        u64::try_from(invalid_depth).ok()
    );
    assert!(
        !expected["invalidDepth"]["accepted"]
            .as_bool()
            .expect("invalid-depth expectation is boolean")
    );
    assert_eq!(
        expected["invalidDepth"]["errorCategory"].as_str(),
        Some("resource_limit")
    );
    let error = rejected.expect_err("depth above the model maximum is rejected");
    let _: LimitsError = error;
    assert_eq!(error.to_string(), "Structure depth cannot exceed 64");
}
