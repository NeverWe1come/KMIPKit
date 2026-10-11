//! KMIPKIT-0017-FR-004, FR-005, and FR-012 secret-handling contracts.
//!
//! Get field shapes follow OASIS KMIP v2.1 §6.1.19, Tables 220–221; the Any
//! Object remains opaque as described in §2. Payload redaction and zeroization
//! are KMIPKit ownership requirements, not OASIS conformance claims.

use std::error::Error;

use kmipkit_protocol::{GetRequest, GetResponse, ResponseBatchItemView, ResponseMessage};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

const GET_OPERATION: u32 = 0x0000_000A;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const KEY_WRAPPING_SPECIFICATION: u32 = 0x0042_0047;
const TEST_STRUCTURE_TAG: u32 = 0x0054_1234;
const TEST_TEXT_TAG: u32 = 0x0054_1235;
const TEST_BYTES_TAG: u32 = 0x0054_1236;
const REQUEST_TEXT_SENTINEL: &str = "KMIPKIT_GET_WRAPPING_TEXT_SECRET_83fa21";
const REQUEST_BYTES_SENTINEL: &[u8] = b"KMIPKIT_GET_WRAPPING_BYTES_SECRET_d9426c";
const RESPONSE_TEXT_SENTINEL: &str = "KMIPKIT_GET_OBJECT_TEXT_SECRET_5b4f11";
const RESPONSE_BYTES_SENTINEL: &[u8] = b"KMIPKIT_GET_OBJECT_BYTES_SECRET_771e09";
const ERROR_SENTINEL: &str = "KMIPKIT_GET_OBJECT_ERROR_SECRET_24d8c0";

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits the KMIP Tag width")
        .try_checked()
        .expect("fixture tag is assigned or in the allocated vendor range")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("checked fixture tag and value form a TTLV Item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("fixture structure stays within the configured nesting limit");
    }
    structure
}

fn wrapping_specification(text: &str, bytes: &[u8]) -> Structure {
    structure([item(
        TEST_STRUCTURE_TAG,
        Value::structure(structure([
            item(TEST_TEXT_TAG, Value::text_string(text.to_owned())),
            item(TEST_BYTES_TAG, Value::byte_string(bytes.to_vec())),
        ])),
    )])
}

fn contains_text(value: ValueView<'_>, sentinel: &str) -> bool {
    match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .any(|child| child.with_value(|value| contains_text(value, sentinel))),
        ValueView::TextString(text) => text == sentinel,
        _ => false,
    }
}

fn contains_bytes(value: ValueView<'_>, sentinel: &[u8]) -> bool {
    match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .any(|child| child.with_value(|value| contains_bytes(value, sentinel))),
        ValueView::ByteString(bytes) => bytes == sentinel,
        _ => false,
    }
}

fn any_object(text: &str, bytes: &[u8]) -> Item {
    item(
        TEST_STRUCTURE_TAG,
        Value::structure(structure([item(
            TEST_STRUCTURE_TAG,
            Value::structure(structure([
                item(TEST_TEXT_TAG, Value::text_string(text.to_owned())),
                item(TEST_BYTES_TAG, Value::byte_string(bytes.to_vec())),
            ])),
        )])),
    )
}

fn response_payload(object: Item, duplicate_object: Option<Item>) -> Structure {
    let mut fields = vec![
        item(OBJECT_TYPE, Value::enumeration(0xF123_4567)),
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string("get-secret-test-object".to_owned()),
        ),
        object,
    ];
    if let Some(duplicate_object) = duplicate_object {
        fields.push(duplicate_object);
    }
    structure(fields)
}

fn response_message(payload: Structure) -> ResponseMessage {
    let version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(version)),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch_item = structure([
        item(OPERATION, Value::enumeration(GET_OPERATION)),
        item(RESULT_STATUS, Value::enumeration(0)),
        item(RESPONSE_PAYLOAD, Value::structure(payload)),
    ]);

    ResponseMessage::try_from_ttlv(structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(batch_item)),
    ]))
    .expect("fixture is a valid KMIP 2.1 response message")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one response batch item")
}

fn diagnostics(error: &dyn Error) -> String {
    let mut output = format!("debug={error:?}; display={error}");
    let mut source = error.source();
    while let Some(cause) = source {
        output.push_str(&format!("; source-debug={cause:?}; source-display={cause}"));
        source = cause.source();
    }
    output
}

#[test]
fn explicit_wrapping_fields_are_redacted_and_move_into_the_ttlv_owner() {
    // FR-005 keeps explicit Table 220 wrapping input caller-selected. Formatting
    // protection and zeroization on drop come from the KMIPKit TTLV contract.
    let request = GetRequest::new().with_key_wrapping_specification(wrapping_specification(
        REQUEST_TEXT_SENTINEL,
        REQUEST_BYTES_SENTINEL,
    ));
    let request_debug = format!("{request:?}");

    assert!(!request_debug.contains(REQUEST_TEXT_SENTINEL));
    assert!(!request_debug.contains("KMIPKIT_GET_WRAPPING_BYTES_SECRET"));

    let payload = request
        .to_ttlv_payload()
        .expect("explicit Table 220 wrapping input is representable");
    let payload_view = payload.view();
    let wrapping = payload_view
        .children()
        .iter()
        .find(|field| field.tag().raw() == KEY_WRAPPING_SPECIFICATION)
        .expect("the explicit Key Wrapping Specification remains present");
    assert!(wrapping.with_value(|value| contains_text(value, REQUEST_TEXT_SENTINEL)));
    assert!(wrapping.with_value(|value| contains_bytes(value, REQUEST_BYTES_SENTINEL)));

    let payload_debug = format!("{payload:?}; {wrapping:?}");
    assert!(!payload_debug.contains(REQUEST_TEXT_SENTINEL));
    assert!(!payload_debug.contains("KMIPKIT_GET_WRAPPING_BYTES_SECRET"));
    drop(payload);
}

#[test]
fn wrapping_depth_error_redacts_the_consumed_nested_secret() {
    // A Table 220 field that would exceed the existing TTLV depth limit fails
    // without exposing its nested payload through the sanitized error chain.
    let mut deepest = structure([item(
        TEST_TEXT_TAG,
        Value::text_string(REQUEST_TEXT_SENTINEL.to_owned()),
    )]);
    for _ in 1..64 {
        deepest = structure([item(TEST_STRUCTURE_TAG, Value::structure(deepest))]);
    }

    let error = GetRequest::new()
        .with_key_wrapping_specification(deepest)
        .to_ttlv_payload()
        .expect_err("wrapping input cannot exceed the configured TTLV depth limit");
    let diagnostic = diagnostics(&error);

    assert!(!diagnostic.contains(REQUEST_TEXT_SENTINEL));
}

#[test]
fn nested_get_object_is_owned_after_source_drop_and_redacted_in_debug() {
    // §2 and Table 221 leave Any Object opaque. FR-004 requires the retained
    // object to use KMIPKit's existing zeroizing, redacting Item ownership.
    let response = {
        let message = response_message(response_payload(
            any_object(RESPONSE_TEXT_SENTINEL, RESPONSE_BYTES_SENTINEL),
            None,
        ));
        GetResponse::try_from_response_item(response_item(&message))
            .expect("successful Get response retains its opaque Any Object")
    };
    let object = response
        .object()
        .expect("successful Get exposes its object");

    assert!(object.with_value(|value| contains_text(value, RESPONSE_TEXT_SENTINEL)));
    assert!(object.with_value(|value| contains_bytes(value, RESPONSE_BYTES_SENTINEL)));

    let diagnostic = format!("response={response:?}; object={object:?}");
    assert!(!diagnostic.contains(RESPONSE_TEXT_SENTINEL));
    assert!(!diagnostic.contains("KMIPKIT_GET_OBJECT_BYTES_SECRET"));

    // Dropping this response drops the independently owned Item; TTLV's
    // existing zeroization tests cover clearing its payload allocation.
    drop(response);
}

#[test]
fn malformed_get_object_error_redacts_nested_payload_sentinels() {
    // FR-012 requires sanitized failures even when an invalid response carries
    // a secret-bearing nested Any Object.
    let message = response_message(response_payload(
        any_object(ERROR_SENTINEL, RESPONSE_BYTES_SENTINEL),
        Some(any_object(RESPONSE_TEXT_SENTINEL, RESPONSE_BYTES_SENTINEL)),
    ));
    let error = GetResponse::try_from_response_item(response_item(&message))
        .expect_err("successful Get payload rejects duplicate Any Object fields");
    let diagnostic = diagnostics(&error);

    assert!(!diagnostic.contains(ERROR_SENTINEL));
    assert!(!diagnostic.contains(RESPONSE_TEXT_SENTINEL));
    assert!(!diagnostic.contains("KMIPKIT_GET_OBJECT_BYTES_SECRET"));
}
