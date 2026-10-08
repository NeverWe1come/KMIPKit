#![cfg(test)]

//! Attribute model requirements from OASIS KMIP v2.1 §§4.60 and 5.1–5.4,
//! Tables 150 and 157–160. Generic direct attribute items preserve their
//! original tags, typed values, repetitions, and order under KMIPKIT-0014-FR-014.

use crate::AttributeSet;
use kmipkit_ttlv::codec::decode;
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, StructureView, Tag, Value, ValueView};
use quickcheck::{Arbitrary, Gen, QuickCheck};

const PROPERTY_SEED: u64 = 0x4B4D_4950_4B49_5434;
const STANDARD_ATTRIBUTE_TAG: u32 = 0x0042_0001;
const SECOND_STANDARD_ATTRIBUTE_TAG: u32 = 0x0042_0028;
const VENDOR_EXTENSION_TAG: u32 = 0x0054_1234;
const TTLV_ENVELOPE_TAG: u32 = 0x0054_1235;
const VENDOR_ATTRIBUTE_TAG: u32 = 0x0042_0008;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE_TAG: u32 = 0x0042_000B;
const VALUE_SENTINEL: &[u8] = b"attribute-value-secret-sentinel";

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits the TTLV tag width")
        .try_checked()
        .expect("fixture tag uses an assigned or extension allocation")
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("checked tag and value form a generic TTLV item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut result = Structure::new();
    for child in items {
        result
            .try_push(child)
            .expect("fixture structure stays within the nesting limit");
    }
    result
}

fn vendor_attribute(children: impl IntoIterator<Item = Item>) -> Item {
    item(VENDOR_ATTRIBUTE_TAG, Value::structure(structure(children)))
}

fn valid_vendor_attribute() -> Item {
    vendor_attribute([
        item(
            VENDOR_IDENTIFICATION_TAG,
            Value::text_string("KMIPKit.TestVendor_1".to_owned()),
        ),
        item(
            ATTRIBUTE_NAME_TAG,
            Value::text_string("Opaque.Attribute".to_owned()),
        ),
        item(
            ATTRIBUTE_VALUE_TAG,
            Value::byte_string(VALUE_SENTINEL.to_vec()),
        ),
    ])
}

#[test]
fn attribute_set_preserves_unknown_standard_and_vendor_item_tags() {
    let attributes = AttributeSet::try_new([
        item(STANDARD_ATTRIBUTE_TAG, Value::date_time(1_760_000_000)),
        item(SECOND_STANDARD_ATTRIBUTE_TAG, Value::integer(256)),
        item(STANDARD_ATTRIBUTE_TAG, Value::date_time(1_760_000_001)),
        item(
            VENDOR_EXTENSION_TAG,
            Value::byte_string(vec![0, 0x80, 0xff]),
        ),
    ])
    .expect("generic direct attribute items remain lossless");

    assert_eq!(attributes.len(), 4);
    assert_eq!(
        attributes
            .as_items()
            .iter()
            .map(|item| item.tag().raw())
            .collect::<Vec<_>>(),
        [
            STANDARD_ATTRIBUTE_TAG,
            SECOND_STANDARD_ATTRIBUTE_TAG,
            STANDARD_ATTRIBUTE_TAG,
            VENDOR_EXTENSION_TAG,
        ]
    );
    assert_eq!(attributes.as_items()[0].item_type(), ItemType::DateTime);
    assert_eq!(attributes.as_items()[1].item_type(), ItemType::Integer);
    assert_eq!(attributes.as_items()[3].item_type(), ItemType::ByteString);
}

#[test]
fn attribute_set_preserves_the_distinct_vendor_attribute_structure() {
    let attributes = AttributeSet::try_new([valid_vendor_attribute()])
        .expect("Table 150 vendor structure has its required members");
    let vendor = &attributes.as_items()[0];

    assert_eq!(vendor.tag().raw(), VENDOR_ATTRIBUTE_TAG);
    assert_eq!(vendor.item_type(), ItemType::Structure);
    let fields = vendor.with_value(|value| match value {
        ValueView::Structure(structure) => Some(
            structure
                .children()
                .iter()
                .map(|field| (field.tag().raw(), field.item_type()))
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert_eq!(
        fields,
        Some(vec![
            (VENDOR_IDENTIFICATION_TAG, ItemType::TextString),
            (ATTRIBUTE_NAME_TAG, ItemType::TextString),
            (ATTRIBUTE_VALUE_TAG, ItemType::ByteString),
        ])
    );
}

#[test]
fn attribute_set_rejects_malformed_vendor_attribute_fields() {
    let missing_vendor = vendor_attribute([
        item(ATTRIBUTE_NAME_TAG, Value::text_string("name".to_owned())),
        item(ATTRIBUTE_VALUE_TAG, Value::integer(7)),
    ]);
    let wrong_vendor_type = vendor_attribute([
        item(VENDOR_IDENTIFICATION_TAG, Value::integer(7)),
        item(ATTRIBUTE_NAME_TAG, Value::text_string("name".to_owned())),
        item(ATTRIBUTE_VALUE_TAG, Value::integer(7)),
    ]);
    let missing_attribute_name = vendor_attribute([
        item(
            VENDOR_IDENTIFICATION_TAG,
            Value::text_string("KMIPKit.TestVendor".to_owned()),
        ),
        item(ATTRIBUTE_VALUE_TAG, Value::integer(7)),
    ]);
    let missing_attribute_value = vendor_attribute([
        item(
            VENDOR_IDENTIFICATION_TAG,
            Value::text_string("KMIPKit.TestVendor".to_owned()),
        ),
        item(ATTRIBUTE_NAME_TAG, Value::text_string("name".to_owned())),
    ]);
    let invalid_vendor_identifier = vendor_attribute([
        item(
            VENDOR_IDENTIFICATION_TAG,
            Value::text_string("vendor-with-hyphens".to_owned()),
        ),
        item(ATTRIBUTE_NAME_TAG, Value::text_string("name".to_owned())),
        item(ATTRIBUTE_VALUE_TAG, Value::integer(7)),
    ]);

    for malformed in [
        missing_vendor,
        wrong_vendor_type,
        missing_attribute_name,
        missing_attribute_value,
        invalid_vendor_identifier,
    ] {
        assert!(AttributeSet::try_new([malformed]).is_err());
    }
}

#[test]
fn optional_attribute_group_distinguishes_absent_from_present_empty() {
    let absent: Option<AttributeSet> = None;
    let present = Some(AttributeSet::default());

    assert!(absent.is_none());
    assert!(present.as_ref().is_some_and(AttributeSet::is_empty));
}

#[test]
fn attribute_set_debug_redacts_item_value_bytes() {
    let attributes = AttributeSet::try_new([item(
        STANDARD_ATTRIBUTE_TAG,
        Value::byte_string(VALUE_SENTINEL.to_vec()),
    )])
    .expect("ordinary attribute item is preserved");

    let diagnostic = format!("{attributes:?}");
    assert!(!diagnostic.contains("attribute-value-secret-sentinel"));
    assert!(!diagnostic.contains("97, 116, 116, 114"));
    assert!(diagnostic.contains("item_count"));
}

#[test]
fn vendor_attribute_error_debug_redacts_invalid_vendor_text() {
    let invalid = vendor_attribute([
        item(
            VENDOR_IDENTIFICATION_TAG,
            Value::text_string("secret-vendor-sentinel".to_owned()),
        ),
        item(ATTRIBUTE_NAME_TAG, Value::text_string("name".to_owned())),
        item(ATTRIBUTE_VALUE_TAG, Value::integer(7)),
    ]);
    let error =
        AttributeSet::try_new([invalid]).expect_err("invalid Vendor Identification is rejected");
    let diagnostic = format!("{error:?} {error}");

    assert!(!diagnostic.contains("secret-vendor-sentinel"));
}

#[derive(Clone, Debug)]
struct AttributeCase(Vec<AttributeEntry>);

#[derive(Clone, Debug)]
struct AttributeEntry {
    tag: u32,
    value: AttributeValue,
}

#[derive(Clone, Debug)]
enum AttributeValue {
    Structure,
    Integer(i32),
    LongInteger(i64),
    BigInteger([u8; 8]),
    Enumeration(u32),
    Boolean(bool),
    TextString(String),
    ByteString(Vec<u8>),
    DateTime(i64),
    Interval(u32),
    DateTimeExtended(i64),
}

impl Arbitrary for AttributeCase {
    fn arbitrary(generator: &mut Gen) -> Self {
        let repeated_tag = if bool::arbitrary(generator) {
            STANDARD_ATTRIBUTE_TAG
        } else {
            VENDOR_EXTENSION_TAG
        };
        let mut entries = vec![
            AttributeEntry {
                tag: repeated_tag,
                value: arbitrary_value(generator),
            },
            AttributeEntry {
                tag: repeated_tag,
                value: arbitrary_value(generator),
            },
        ];
        let additional = usize::arbitrary(generator) % 8;
        for _ in 0..additional {
            let tag = match u8::arbitrary(generator) % 3 {
                0 => STANDARD_ATTRIBUTE_TAG,
                1 => SECOND_STANDARD_ATTRIBUTE_TAG,
                _ => VENDOR_EXTENSION_TAG,
            };
            entries.push(AttributeEntry {
                tag,
                value: arbitrary_value(generator),
            });
        }
        Self(entries)
    }
}

fn arbitrary_value(generator: &mut Gen) -> AttributeValue {
    match u8::arbitrary(generator) % 11 {
        0 => AttributeValue::Structure,
        1 => AttributeValue::Integer(i32::arbitrary(generator)),
        2 => AttributeValue::LongInteger(i64::arbitrary(generator)),
        3 => {
            let mut octets = [0; 8];
            for octet in &mut octets {
                *octet = u8::arbitrary(generator);
            }
            AttributeValue::BigInteger(octets)
        }
        4 => AttributeValue::Enumeration(u32::arbitrary(generator)),
        5 => AttributeValue::Boolean(bool::arbitrary(generator)),
        6 => AttributeValue::TextString(String::arbitrary(generator)),
        7 => AttributeValue::ByteString(Vec::<u8>::arbitrary(generator)),
        8 => AttributeValue::DateTime(i64::arbitrary(generator)),
        9 => AttributeValue::Interval(u32::arbitrary(generator)),
        _ => AttributeValue::DateTimeExtended(i64::arbitrary(generator)),
    }
}

impl AttributeValue {
    fn into_ttlv(self) -> Value {
        match self {
            Self::Structure => Value::structure(Structure::new()),
            Self::Integer(value) => Value::integer(value),
            Self::LongInteger(value) => Value::long_integer(value),
            Self::BigInteger(value) => Value::big_integer(value.to_vec()),
            Self::Enumeration(value) => Value::enumeration(value),
            Self::Boolean(value) => Value::boolean(value),
            Self::TextString(value) => Value::text_string(value),
            Self::ByteString(value) => Value::byte_string(value),
            Self::DateTime(value) => Value::date_time(value),
            Self::Interval(value) => Value::interval(value),
            Self::DateTimeExtended(value) => Value::date_time_extended(value),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CapturedItem {
    tag: u32,
    value: CapturedValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CapturedValue {
    Structure(Vec<CapturedItem>),
    Integer(i32),
    LongInteger(i64),
    BigInteger(Vec<u8>),
    Enumeration(u32),
    Boolean(bool),
    TextString(String),
    ByteString(Vec<u8>),
    DateTime(i64),
    Interval(u32),
    DateTimeExtended(i64),
    Unrecognized,
}

fn capture_items(items: &[Item]) -> Vec<CapturedItem> {
    items
        .iter()
        .map(|item| CapturedItem {
            tag: item.tag().raw(),
            value: item.with_value(capture_value),
        })
        .collect()
}

fn capture_structure(structure: &StructureView<'_>) -> Vec<CapturedItem> {
    capture_items(structure.children())
}

fn capture_value(value: ValueView<'_>) -> CapturedValue {
    match value {
        ValueView::Structure(structure) => CapturedValue::Structure(capture_structure(&structure)),
        ValueView::Integer(value) => CapturedValue::Integer(*value),
        ValueView::LongInteger(value) => CapturedValue::LongInteger(*value),
        ValueView::BigInteger(value) => CapturedValue::BigInteger(value.to_vec()),
        ValueView::Enumeration(value) => CapturedValue::Enumeration(*value),
        ValueView::Boolean(value) => CapturedValue::Boolean(*value),
        ValueView::TextString(value) => CapturedValue::TextString(value.to_owned()),
        ValueView::ByteString(value) => CapturedValue::ByteString(value.to_vec()),
        ValueView::DateTime(value) => CapturedValue::DateTime(*value),
        ValueView::Interval(value) => CapturedValue::Interval(*value),
        ValueView::DateTimeExtended(value) => CapturedValue::DateTimeExtended(*value),
        _ => CapturedValue::Unrecognized,
    }
}

fn encode_item(item: &Item, output: &mut Vec<u8>) -> Option<()> {
    let (item_type, payload) = item.with_value(encode_value)?;
    append_ttlv(item.tag().raw(), item_type, &payload, output)
}

fn encode_value(value: ValueView<'_>) -> Option<(u8, Vec<u8>)> {
    match value {
        ValueView::Structure(structure) => {
            let mut payload = Vec::new();
            for child in structure.children() {
                encode_item(child, &mut payload)?;
            }
            Some((0x01, payload))
        }
        ValueView::Integer(value) => Some((0x02, value.to_be_bytes().to_vec())),
        ValueView::LongInteger(value) => Some((0x03, value.to_be_bytes().to_vec())),
        ValueView::BigInteger(value) => Some((0x04, value.to_vec())),
        ValueView::Enumeration(value) => Some((0x05, value.to_be_bytes().to_vec())),
        ValueView::Boolean(value) => {
            let mut payload = vec![0; 8];
            payload[7] = u8::from(*value);
            Some((0x06, payload))
        }
        ValueView::TextString(value) => Some((0x07, value.as_bytes().to_vec())),
        ValueView::ByteString(value) => Some((0x08, value.to_vec())),
        ValueView::DateTime(value) => Some((0x09, value.to_be_bytes().to_vec())),
        ValueView::Interval(value) => Some((0x0A, value.to_be_bytes().to_vec())),
        ValueView::DateTimeExtended(value) => Some((0x0B, value.to_be_bytes().to_vec())),
        _ => None,
    }
}

fn append_ttlv(raw_tag: u32, item_type: u8, payload: &[u8], output: &mut Vec<u8>) -> Option<()> {
    let length = u32::try_from(payload.len()).ok()?;
    let tag_bytes = raw_tag.to_be_bytes();
    output.extend_from_slice(&tag_bytes[1..]);
    output.push(item_type);
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(payload);
    let padding = (8 - payload.len() % 8) % 8;
    output.resize(output.len() + padding, 0);
    Some(())
}

fn encode_attribute_envelope(items: &[Item]) -> Option<Vec<u8>> {
    let mut payload = Vec::new();
    for item in items {
        encode_item(item, &mut payload)?;
    }
    let mut encoded = Vec::new();
    append_ttlv(TTLV_ENVELOPE_TAG, 0x01, &payload, &mut encoded)?;
    Some(encoded)
}

#[allow(clippy::needless_pass_by_value)]
fn roundtrips_ordered_repeated_direct_attribute_items(case: AttributeCase) -> bool {
    let items = case
        .0
        .into_iter()
        .map(|entry| item(entry.tag, entry.value.into_ttlv()))
        .collect::<Vec<_>>();
    let Ok(attributes) = AttributeSet::try_new(items) else {
        return false;
    };
    let expected = capture_items(attributes.as_items());
    let Some(wire) = encode_attribute_envelope(attributes.as_items()) else {
        return false;
    };
    let Ok(decoded) = decode(&wire) else {
        return false;
    };
    let actual = decoded.with_value(|value| match value {
        ValueView::Structure(structure) => Some(capture_structure(&structure)),
        _ => None,
    });

    actual.as_ref() == Some(&expected)
}

#[test]
fn property_roundtrips_ordered_repeated_direct_attribute_items() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(32, PROPERTY_SEED))
        .tests(256)
        .quickcheck(
            roundtrips_ordered_repeated_direct_attribute_items as fn(AttributeCase) -> bool,
        );
}
