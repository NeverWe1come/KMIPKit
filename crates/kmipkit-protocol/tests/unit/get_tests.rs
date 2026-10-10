//! Get request tests derived from OASIS KMIP Specification v2.1 §6.1.19,
//! Table 220. Valid Key Compression Type, Key Format Type, and Key Wrap Type
//! values come from §§11.24, 11.25 (Table 461), and 11.29. These are local
//! source-derived vectors, not official OASIS Test Cases.
//!
//! Traceability: `KMIPKIT-ELEM-OP-C2S-GET`; KMIPKIT-0017 FR-002, FR-003;
//! SC-001 and SC-002.

use crate::{GetRequest, KeyCompressionType, KeyFormatType, KeyWrapType, UniqueIdentifier};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Value, ValueView};

const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const KEY_FORMAT_TYPE_TAG: u32 = 0x0042_0042;
const KEY_WRAP_TYPE_TAG: u32 = 0x0042_00F8;
const KEY_COMPRESSION_TYPE_TAG: u32 = 0x0042_0041;
const KEY_WRAPPING_SPECIFICATION_TAG: u32 = 0x0042_0047;
const TEST_EXTENSION_TAG: u32 = 0x0054_1234;

fn field_signature(payload: &Structure) -> Vec<(u32, ItemType)> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| (field.tag().raw(), field.item_type()))
        .collect()
}

fn assert_enumeration_field(request: GetRequest, tag: u32, value: u32) {
    let payload = request
        .to_ttlv_payload()
        .expect("a Table 220 Enumeration field is representable");
    let fields = payload.view().children();

    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), tag);
    assert_eq!(fields[0].item_type(), ItemType::Enumeration);
    assert!(
        fields[0].with_value(
            |actual| matches!(actual, ValueView::Enumeration(actual) if *actual == value)
        )
    );
}

#[test]
fn request_omits_every_optional_table_220_field_when_unspecified() {
    let payload = GetRequest::new()
        .to_ttlv_payload()
        .expect("an empty Table 220 request payload is valid");

    assert!(payload.view().children().is_empty());
}

#[test]
fn request_preserves_the_optional_unique_identifier() {
    let payload = GetRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("object-123".to_owned()))
        .to_ttlv_payload()
        .expect("Table 220 permits the optional Unique Identifier");
    let fields = payload.view().children();

    assert_eq!(
        field_signature(&payload),
        [(UNIQUE_IDENTIFIER_TAG, ItemType::TextString)]
    );
    assert!(fields[0].with_value(
        |value| matches!(value, ValueView::TextString(actual) if actual == "object-123")
    ));
}

#[test]
fn request_preserves_the_optional_key_format_type_enumeration() {
    // §11.25, Table 461 assigns Raw the Enumeration value 1.
    assert_enumeration_field(
        GetRequest::new().with_key_format_type(KeyFormatType::from_raw(1)),
        KEY_FORMAT_TYPE_TAG,
        1,
    );
}

#[test]
fn request_preserves_the_optional_key_wrap_type_enumeration() {
    // §11.29 assigns Not Wrapped the Enumeration value 1.
    assert_enumeration_field(
        GetRequest::new().with_key_wrap_type(KeyWrapType::from_raw(1)),
        KEY_WRAP_TYPE_TAG,
        1,
    );
}

#[test]
fn request_preserves_the_optional_key_compression_type_enumeration() {
    // §11.24, Table 459 assigns EC Public Key Type Uncompressed the value 1.
    assert_enumeration_field(
        GetRequest::new().with_key_compression_type(KeyCompressionType::from_raw(1)),
        KEY_COMPRESSION_TYPE_TAG,
        1,
    );
}

#[test]
fn request_preserves_the_optional_key_wrapping_specification_structure() {
    let mut specification = Structure::new();
    specification
        .try_push(
            Item::new(
                RawTag::new(TEST_EXTENSION_TAG)
                    .expect("fixture tag fits TTLV")
                    .try_checked()
                    .expect("fixture tag uses the allocated extension range"),
                Value::integer(7),
            )
            .expect("fixture extension is a valid TTLV item"),
        )
        .expect("fixture specification stays within the nesting limit");
    let payload = GetRequest::new()
        .with_key_wrapping_specification(specification)
        .to_ttlv_payload()
        .expect("Table 220 permits the optional Key Wrapping Specification");
    let fields = payload.view().children();

    assert_eq!(
        field_signature(&payload),
        [(KEY_WRAPPING_SPECIFICATION_TAG, ItemType::Structure)]
    );
    assert!(fields[0].with_value(|value| matches!(
        value,
        ValueView::Structure(structure)
            if structure.children().len() == 1
                && structure.children()[0].tag().raw() == TEST_EXTENSION_TAG
                && structure.children()[0].with_value(|nested| matches!(nested, ValueView::Integer(7)))
    )));
}

#[test]
fn request_serializes_present_table_220_fields_in_source_order() {
    let mut specification = Structure::new();
    specification
        .try_push(
            Item::new(
                RawTag::new(TEST_EXTENSION_TAG)
                    .expect("fixture tag fits TTLV")
                    .try_checked()
                    .expect("fixture tag uses the allocated extension range"),
                Value::integer(7),
            )
            .expect("fixture extension is a valid TTLV item"),
        )
        .expect("fixture specification stays within the nesting limit");
    let payload = GetRequest::new()
        .with_key_wrapping_specification(specification)
        .with_key_compression_type(KeyCompressionType::from_raw(1))
        .with_key_wrap_type(KeyWrapType::from_raw(1))
        .with_key_format_type(KeyFormatType::from_raw(1))
        .with_unique_identifier(UniqueIdentifier::TextString("object-123".to_owned()))
        .to_ttlv_payload()
        .expect("all supplied Table 220 fields are representable");

    assert_eq!(
        field_signature(&payload),
        [
            (UNIQUE_IDENTIFIER_TAG, ItemType::TextString),
            (KEY_FORMAT_TYPE_TAG, ItemType::Enumeration),
            (KEY_WRAP_TYPE_TAG, ItemType::Enumeration),
            (KEY_COMPRESSION_TYPE_TAG, ItemType::Enumeration),
            (KEY_WRAPPING_SPECIFICATION_TAG, ItemType::Structure),
        ]
    );
}
