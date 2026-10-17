use std::fs;
use std::path::PathBuf;

use kmipkit_ttlv::{Item, ItemType, ValueView, codec};

use super::*;

// OASIS KMIP Test Cases v2.1 CN01 §§2.68–2.69 and KMIP Specification v2.1
// §6.1.19 Tables 220–222. These are four selected operation-item checks,
// not complete official Test Case passes.
const TC_PKCS12_1_21: &str =
    include_str!("../../../../specification/oasis/kmip-2.1/fixtures/TC-PKCS12-1-21.xml");
const TC_PKCS12_2_21: &str =
    include_str!("../../../../specification/oasis/kmip-2.1/fixtures/TC-PKCS12-2-21.xml");

const MISPAIRED_RESPONSE_CORRELATION_XML: &str = r#"
<KMIP>
  <RequestMessage><RequestHeader><ClientCorrelationValue type="TextString" value="T037-CORRELATION step=0"/></RequestHeader><BatchItem><Operation type="Enumeration" value="Get"/></BatchItem></RequestMessage>
  <ResponseMessage><ResponseHeader><ClientCorrelationValue type="TextString" value="T037-CORRELATION step=1"/></ResponseHeader><BatchItem><Operation type="Enumeration" value="Get"/></BatchItem></ResponseMessage>
  <RequestMessage><RequestHeader><ClientCorrelationValue type="TextString" value="T037-CORRELATION step=1"/></RequestHeader><BatchItem><Operation type="Enumeration" value="Get"/></BatchItem></RequestMessage>
  <ResponseMessage><ResponseHeader><ClientCorrelationValue type="TextString" value="T037-CORRELATION step=0"/></ResponseHeader><BatchItem><Operation type="Enumeration" value="Get"/></BatchItem></ResponseMessage>
</KMIP>
"#;

const NON_ADJACENT_REQUEST_RESPONSE_XML: &str = r#"
<KMIP>
  <RequestMessage><RequestHeader><ClientCorrelationValue type="TextString" value="T037-ADJACENCY step=0"/></RequestHeader><BatchItem><Operation type="Enumeration" value="Get"/></BatchItem></RequestMessage>
  <RequestMessage><RequestHeader><ClientCorrelationValue type="TextString" value="T037-ADJACENCY step=1"/></RequestHeader><BatchItem><Operation type="Enumeration" value="Get"/></BatchItem></RequestMessage>
  <ResponseMessage><ResponseHeader><ClientCorrelationValue type="TextString" value="T037-ADJACENCY step=0"/></ResponseHeader><BatchItem><Operation type="Enumeration" value="Get"/></BatchItem></ResponseMessage>
  <ResponseMessage><ResponseHeader><ClientCorrelationValue type="TextString" value="T037-ADJACENCY step=1"/></ResponseHeader><BatchItem><Operation type="Enumeration" value="Get"/></BatchItem></ResponseMessage>
</KMIP>
"#;

const REQUEST_PAYLOAD_TAG: u32 = 0x0042_0079;
const RESPONSE_PAYLOAD_TAG: u32 = 0x0042_007C;
const ATTRIBUTES_TAG: u32 = 0x0042_0125;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const KEY_FORMAT_TYPE_TAG: u32 = 0x0042_0042;
const OBJECT_TYPE_TAG: u32 = 0x0042_0057;
const PRIVATE_KEY_TAG: u32 = 0x0042_0064;
const CERTIFICATE_TAG: u32 = 0x0042_0013;
const KEY_VALUE_LOCATION_TAG: u32 = 0x0042_00B8;
const KEY_VALUE_LOCATION_VALUE_TAG: u32 = 0x0042_00B9;
const KEY_VALUE_LOCATION_TYPE_TAG: u32 = 0x0042_00BA;
const APPLICATION_SPECIFIC_INFORMATION_TAG: u32 = 0x0042_0004;
const APPLICATION_NAMESPACE_TAG: u32 = 0x0042_0003;
const APPLICATION_DATA_TAG: u32 = 0x0042_0002;
const OFFSET_ITEMS_TAG: u32 = 0x0042_00D4;
const PKCS_12_FORMAT: u32 = 0x0000_0016;
const URI_LOCATION_TYPE: u32 = 0x0000_0002;

#[test]
fn pkcs12_case_1_preserves_get_item_at_correlation_step_8() {
    let selected = selected_get_items(TC_PKCS12_1_21, "TC-PKCS12-1-21", &[8]);

    assert_eq!(selected.len(), 1, "one in-scope Get item is selected");
    assert_eq!(selected[0].step_identity, "TC-PKCS12-1-21 step=8");
    assert_get_item(
        &selected[0],
        &[UNIQUE_IDENTIFIER_TAG, KEY_FORMAT_TYPE_TAG],
        PRIVATE_KEY_TAG,
        4,
    );
}

#[test]
fn pkcs12_case_2_preserves_get_items_at_correlation_steps_5_7_and_9() {
    let selected = selected_get_items(TC_PKCS12_2_21, "TC-PKCS12-2-21", &[5, 7, 9]);

    assert_eq!(
        selected.len(),
        3,
        "all three in-scope Get items are selected"
    );
    assert_eq!(
        selected
            .iter()
            .map(|item| item.step_identity.as_str())
            .collect::<Vec<_>>(),
        [
            "TC-PKCS12-2-21 step=5",
            "TC-PKCS12-2-21 step=7",
            "TC-PKCS12-2-21 step=9",
        ]
    );
    assert_get_item(&selected[0], &[UNIQUE_IDENTIFIER_TAG], PRIVATE_KEY_TAG, 4);
    assert_get_item(&selected[1], &[UNIQUE_IDENTIFIER_TAG], CERTIFICATE_TAG, 1);
    assert_get_item(
        &selected[2],
        &[UNIQUE_IDENTIFIER_TAG, KEY_FORMAT_TYPE_TAG],
        PRIVATE_KEY_TAG,
        4,
    );
}

#[test]
fn selected_get_items_rejects_reordered_response_correlations() {
    let result = std::panic::catch_unwind(|| {
        selected_get_items(
            MISPAIRED_RESPONSE_CORRELATION_XML,
            "T037-CORRELATION",
            &[0, 1],
        )
    });

    assert!(
        result.is_err(),
        "a reordered response correlation must not pair with the request by list order"
    );
}

#[test]
fn selected_get_items_rejects_non_adjacent_request_response_messages() {
    let result = std::panic::catch_unwind(|| {
        selected_get_items(NON_ADJACENT_REQUEST_RESPONSE_XML, "T037-ADJACENCY", &[0, 1])
    });

    assert!(
        result.is_err(),
        "request and response messages must remain adjacent in source order"
    );
}

#[test]
fn synthetic_big_integer_fixture_item_is_parsed_without_exposing_its_bytes() {
    let document = Document::parse(r#"<Modulus type="BigInteger" value="01020304"/>"#)
        .expect("synthetic BigInteger XML parses");
    let catalog = CatalogMaps::load().expect("pinned catalog resolves Modulus");
    let mut item_count = 0;
    let item = parse_item_with_get_test_symbols(
        document.root_element(),
        &catalog,
        1,
        &mut item_count,
        true,
        OasisProfileFixtureBindings::default(),
        false,
    )
    .expect("synthetic BigInteger fixture item parses");

    assert_eq!(item.item_type(), ItemType::BigInteger);
    let byte_count = item.with_value(|value| match value {
        ValueView::BigInteger(bytes) => bytes.len(),
        _ => 0,
    });
    assert_eq!(byte_count, 4, "synthetic BigInteger has four opaque bytes");
}

#[test]
fn derived_profile_tape_library_vector_covers_only_locate_and_get_payloads() {
    // OASIS KMIP Profiles v2.1 §§5.12.2 and 5.12.6.3; KMIP Specification
    // v2.1 §§6.1.19 and 6.1.28. The sample does not claim profile support.
    let vector = load_derived_vector(
        "KMIPKIT-TEST-PROF-5-12-6-3-TL-M-3-21-derived.json",
        "KMIPKIT-TEST-PROF-5-12-6-3",
        "TL-M-3-21",
        &["Locate", "Get"],
        2,
    );
    let payloads = decoded_payloads(&vector);

    let locate = &payloads[0];
    assert_eq!(direct_tags(locate), [ATTRIBUTES_TAG]);
    let attributes = required_child(locate, ATTRIBUTES_TAG);
    let application_info = required_child(&attributes, APPLICATION_SPECIFIC_INFORMATION_TAG);
    assert_eq!(
        direct_tags(&application_info),
        [APPLICATION_NAMESPACE_TAG, APPLICATION_DATA_TAG]
    );
    assert_text_value(
        &required_child(&application_info, APPLICATION_NAMESPACE_TAG),
        "LIBRARY-LTO",
    );
    assert_text_value(
        &required_child(&application_info, APPLICATION_DATA_TAG),
        "A1B2",
    );

    let get = &payloads[1];
    assert_eq!(direct_tags(get), [UNIQUE_IDENTIFIER_TAG]);
}

#[test]
fn derived_metadata_only_case_vector_contains_a_locate_item_without_server_results() {
    // OASIS KMIP Test Cases v2.1 CN01 §2.62; KMIP Specification v2.1 §6.1.28.
    let vector = load_derived_vector(
        "KMIPKIT-TEST-CN01-2-62-TC-MDO-2-21-derived.json",
        "KMIPKIT-TEST-CN01-2-62",
        "TC-MDO-2-21",
        &["Locate"],
        1,
    );
    let payloads = decoded_payloads(&vector);

    assert_eq!(direct_tags(&payloads[0]), [ATTRIBUTES_TAG]);
    assert_eq!(
        direct_tags(&required_child(&payloads[0], ATTRIBUTES_TAG)),
        Vec::<u32>::new()
    );
}

#[test]
fn derived_key_value_location_case_vector_contains_url_locate_criteria() {
    // OASIS KMIP Test Cases v2.1 CN01 §2.63; KMIP Specification v2.1 §4.27
    // Table 82, §11.27 Table 463, and §6.1.28.
    let vector = load_derived_vector(
        "KMIPKIT-TEST-CN01-2-63-TC-MDO-3-21-derived.json",
        "KMIPKIT-TEST-CN01-2-63",
        "TC-MDO-3-21",
        &["Locate"],
        1,
    );
    let payloads = decoded_payloads(&vector);
    let attributes = required_child(&payloads[0], ATTRIBUTES_TAG);
    let location = required_child(&attributes, KEY_VALUE_LOCATION_TAG);

    assert_eq!(
        direct_tags(&location),
        [KEY_VALUE_LOCATION_VALUE_TAG, KEY_VALUE_LOCATION_TYPE_TAG]
    );
    assert_text_value(
        &required_child(&location, KEY_VALUE_LOCATION_VALUE_TAG),
        "https://example.invalid/key-material",
    );
    assert!(matches!(
        enumeration_value(&required_child(&location, KEY_VALUE_LOCATION_TYPE_TAG)),
        Some(URI_LOCATION_TYPE)
    ));
}

#[test]
fn derived_offset_case_vector_contains_locate_offset_input() {
    // OASIS KMIP Test Cases v2.1 CN01 §2.64; KMIP Specification v2.1 §6.1.28.
    let vector = load_derived_vector(
        "KMIPKIT-TEST-CN01-2-64-TC-OFFSET-1-21-derived.json",
        "KMIPKIT-TEST-CN01-2-64",
        "TC-OFFSET-1-21",
        &["Locate"],
        1,
    );
    let payloads = decoded_payloads(&vector);

    assert_eq!(
        direct_tags(&payloads[0]),
        [OFFSET_ITEMS_TAG, ATTRIBUTES_TAG]
    );
    assert!(matches!(
        integer_value(&required_child(&payloads[0], OFFSET_ITEMS_TAG)),
        Some(1)
    ));
}

#[test]
fn derived_symmetric_key_offset_case_vector_contains_locate_offset_input() {
    // OASIS KMIP Test Cases v2.1 CN01 §2.65; KMIP Specification v2.1 §6.1.28.
    let vector = load_derived_vector(
        "KMIPKIT-TEST-CN01-2-65-TC-OFFSET-2-21-derived.json",
        "KMIPKIT-TEST-CN01-2-65",
        "TC-OFFSET-2-21",
        &["Locate"],
        1,
    );
    let payloads = decoded_payloads(&vector);

    assert_eq!(
        direct_tags(&payloads[0]),
        [OFFSET_ITEMS_TAG, ATTRIBUTES_TAG]
    );
    assert!(matches!(
        integer_value(&required_child(&payloads[0], OFFSET_ITEMS_TAG)),
        Some(1)
    ));
}

struct SelectedGetItem {
    step_identity: String,
    request: Structure,
    response: Structure,
}

fn selected_get_items(xml: &str, case_id: &str, steps: &[usize]) -> Vec<SelectedGetItem> {
    let document = Document::parse(xml).expect("pinned source XML parses");
    let root = document.root_element();
    let messages: Vec<_> = root.children().filter(Node::is_element).collect();
    assert_eq!(
        messages.len() % 2,
        0,
        "every source request has an adjacent response message"
    );

    let catalog = CatalogMaps::load().expect("pinned catalog resolves source tags");
    let mut selected = Vec::new();
    for pair in messages.as_chunks::<2>().0 {
        let request_node = pair[0];
        let response_node = pair[1];
        assert!(
            request_node.tag_name().name() == "RequestMessage",
            "source message pairs begin with a request"
        );
        assert!(
            response_node.tag_name().name() == "ResponseMessage",
            "source request is immediately followed by its response"
        );

        let header = unique_child(request_node, "RequestHeader").expect("request has a header");
        let identity = value_attribute(
            unique_child(header, "ClientCorrelationValue").expect("request has correlation id"),
        )
        .expect("correlation id has value");

        let response_header =
            unique_child(response_node, "ResponseHeader").expect("response has a header");
        let mut response_correlations = response_header
            .children()
            .filter(Node::is_element)
            .filter(|node| node.tag_name().name() == "ClientCorrelationValue");
        let response_correlation = response_correlations.next();
        assert!(
            response_correlations.next().is_none(),
            "response has at most one client correlation value"
        );
        if let Some(response_correlation) = response_correlation {
            let response_identity =
                value_attribute(response_correlation).expect("response correlation id has a value");
            assert!(
                response_identity == identity,
                "response correlation id matches its adjacent request"
            );
        }

        let Some((identity_case, numeric_step)) = identity.rsplit_once(" step=") else {
            continue;
        };
        let Ok(step) = numeric_step.parse::<usize>() else {
            continue;
        };
        if identity_case != case_id || !steps.contains(&step) {
            continue;
        }

        let batch = unique_child(request_node, "BatchItem").expect("request has one batch item");
        let operation = unique_child(batch, "Operation").expect("request has operation");
        assert_eq!(
            value_attribute(operation).expect("operation has a value"),
            "Get"
        );
        let response_batch =
            unique_child(response_node, "BatchItem").expect("response has one batch item");
        let response_operation =
            unique_child(response_batch, "Operation").expect("response has operation");
        assert_eq!(
            value_attribute(response_operation).expect("response operation has a value"),
            "Get"
        );

        selected.push(SelectedGetItem {
            step_identity: identity.to_owned(),
            request: parse_get_message_with_test_symbols(request_node, &catalog)
                .expect("selected Get request parses as generic TTLV"),
            response: parse_get_message_with_test_symbols(response_node, &catalog)
                .expect("selected Get response parses as generic TTLV"),
        });
    }
    selected
}

fn assert_get_item(
    selected: &SelectedGetItem,
    request_tags: &[u32],
    object_tag: u32,
    object_type: u32,
) {
    let request_payload = required_message_child(&selected.request, REQUEST_PAYLOAD_TAG);
    assert_eq!(direct_tags(&request_payload), request_tags);
    if request_tags.contains(&KEY_FORMAT_TYPE_TAG) {
        assert!(matches!(
            enumeration_value(&required_child(&request_payload, KEY_FORMAT_TYPE_TAG)),
            Some(PKCS_12_FORMAT)
        ));
    }

    let response_payload = required_message_child(&selected.response, RESPONSE_PAYLOAD_TAG);
    assert_eq!(
        direct_tags(&response_payload),
        [OBJECT_TYPE_TAG, UNIQUE_IDENTIFIER_TAG, object_tag]
    );
    assert_eq!(
        enumeration_value(&required_child(&response_payload, OBJECT_TYPE_TAG)),
        Some(object_type)
    );
    let request_identifier = required_child(&request_payload, UNIQUE_IDENTIFIER_TAG);
    let response_identifier = required_child(&response_payload, UNIQUE_IDENTIFIER_TAG);
    assert!(
        same_item_tree(&request_identifier, &response_identifier),
        "Get response preserves the selected request identifier"
    );
    let object = required_child(&response_payload, object_tag);
    let opaque_clone = object
        .try_clone()
        .expect("opaque Any Object clone succeeds");
    assert!(
        same_item_tree(&object, &opaque_clone),
        "opaque Any Object is preserved"
    );
}

fn same_item_tree(left: &Item, right: &Item) -> bool {
    if left.tag() != right.tag() || left.item_type() != right.item_type() {
        return false;
    }
    left.with_value(|left_value| {
        right.with_value(|right_value| match (left_value, right_value) {
            (ValueView::Structure(left), ValueView::Structure(right)) => {
                let left_children = left.children();
                let right_children = right.children();
                left_children.len() == right_children.len()
                    && left_children
                        .iter()
                        .zip(right_children)
                        .all(|(left, right)| same_item_tree(left, right))
            }
            (ValueView::Integer(left), ValueView::Integer(right)) => left == right,
            (ValueView::LongInteger(left), ValueView::LongInteger(right))
            | (ValueView::DateTime(left), ValueView::DateTime(right))
            | (ValueView::DateTimeExtended(left), ValueView::DateTimeExtended(right)) => {
                left == right
            }
            (ValueView::BigInteger(left), ValueView::BigInteger(right))
            | (ValueView::ByteString(left), ValueView::ByteString(right)) => left == right,
            (ValueView::Enumeration(left), ValueView::Enumeration(right))
            | (ValueView::Interval(left), ValueView::Interval(right)) => left == right,
            (ValueView::Boolean(left), ValueView::Boolean(right)) => left == right,
            (ValueView::TextString(left), ValueView::TextString(right)) => left == right,
            _ => false,
        })
    })
}

fn load_derived_vector(
    file: &str,
    catalog_id: &str,
    official_label: &str,
    operations: &[&str],
    payload_count: usize,
) -> JsonValue {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../kmipkit-protocol/tests/fixtures")
        .join(file);
    let source = fs::read_to_string(path).expect("derived vector file is checked in");
    let vector: JsonValue = serde_json::from_str(&source).expect("derived vector JSON parses");
    assert_eq!(string_field(&vector, "catalog_id"), Some(catalog_id));
    assert_eq!(
        string_field(&vector, "official_label"),
        Some(official_label)
    );
    assert_eq!(
        string_field(&vector, "status"),
        Some("derived-local-not-official")
    );
    assert_eq!(
        vector
            .get("sample_values_synthetic")
            .and_then(JsonValue::as_bool),
        Some(true)
    );
    assert_eq!(
        vector
            .get("request_operations")
            .and_then(JsonValue::as_array)
            .expect("derived vector lists request operations")
            .iter()
            .filter_map(JsonValue::as_str)
            .collect::<Vec<_>>(),
        operations
    );
    let payloads = vector
        .get("request_payload_ttlv_hex")
        .and_then(JsonValue::as_array)
        .expect("derived vector has request payload TTLV");
    assert_eq!(payloads.len(), payload_count);
    vector
}

fn decoded_payloads(vector: &JsonValue) -> Vec<Item> {
    vector
        .get("request_payload_ttlv_hex")
        .and_then(JsonValue::as_array)
        .expect("derived vector has request payload TTLV")
        .iter()
        .map(|payload| {
            let hex = payload.as_str().expect("TTLV payload is hexadecimal text");
            let bytes = decode_hex(hex);
            let item = codec::decode(&bytes).expect("derived request payload decodes");
            assert_eq!(item.tag().raw(), REQUEST_PAYLOAD_TAG);
            assert_eq!(item.item_type(), ItemType::Structure);
            item
        })
        .collect()
}

fn decode_hex(hex: &str) -> Vec<u8> {
    let digits: Vec<_> = hex
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    assert_eq!(digits.len() % 2, 0, "TTLV hexadecimal has an even length");
    digits
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("hex digits are ASCII");
            u8::from_str_radix(text, 16).expect("TTLV payload has hexadecimal digits")
        })
        .collect()
}

fn required_child(item: &Item, tag: u32) -> Item {
    item.with_value(|value| match value {
        ValueView::Structure(value) => value
            .children()
            .iter()
            .find(|child| child.tag().raw() == tag)
            .expect("expected TTLV child tag exists")
            .try_clone()
            .expect("selected TTLV child clone succeeds"),
        _ => panic!("expected TTLV item is a Structure"),
    })
}

fn required_message_child(message: &Structure, tag: u32) -> Item {
    find_descendant(message.view().children(), tag).expect("expected message item tag exists")
}

fn find_descendant(items: &[Item], tag: u32) -> Option<Item> {
    for item in items {
        if item.tag().raw() == tag {
            return item.try_clone().ok();
        }
        let nested = item.with_value(|value| match value {
            ValueView::Structure(value) => find_descendant(value.children(), tag),
            _ => None,
        });
        if nested.is_some() {
            return nested;
        }
    }
    None
}

fn direct_tags(item: &Item) -> Vec<u32> {
    item.with_value(|value| match value {
        ValueView::Structure(value) => value
            .children()
            .iter()
            .map(|child| child.tag().raw())
            .collect(),
        _ => Vec::new(),
    })
}

fn enumeration_value(item: &Item) -> Option<u32> {
    item.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    })
}

fn integer_value(item: &Item) -> Option<i32> {
    item.with_value(|value| match value {
        ValueView::Integer(value) => Some(*value),
        _ => None,
    })
}

fn assert_text_value(item: &Item, expected: &str) {
    assert!(item.with_value(|value| matches!(
        value,
        ValueView::TextString(value) if value == expected
    )));
}
