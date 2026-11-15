//! Source-derived fixtures for OASIS KMIP v2.1 attribute structures.
//!
//! The Vendor Attribute fixture follows §4.60, Table 150. Its linked official
//! case `KMIPKIT-TEST-CN01-2-42` (`TC-I18N-3-21`) is unavailable in the pinned
//! test-case source; this fixture is independent and does not claim that case
//! passed.

use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

pub(crate) const VENDOR_ATTRIBUTE_TAG: u32 = 0x0042_0008;
pub(crate) const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
pub(crate) const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;
pub(crate) const ATTRIBUTE_VALUE_TAG: u32 = 0x0042_000B;

/// Creates a Table 150 Vendor Attribute in the required member order.
pub(crate) fn vendor_attribute_table_150() -> Item {
    let fields = structure([
        item(
            VENDOR_IDENTIFICATION_TAG,
            Value::text_string("KMIPKit.TestVendor_1".to_owned()),
        ),
        item(
            ATTRIBUTE_NAME_TAG,
            Value::text_string("Opaque.Attribute".to_owned()),
        ),
        item(ATTRIBUTE_VALUE_TAG, Value::byte_string(vec![0, 0x80, 0xff])),
    ]);
    item(VENDOR_ATTRIBUTE_TAG, Value::structure(fields))
}

pub(crate) fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("fixture tag and value form a valid Item")
}

pub(crate) fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for item in items {
        structure
            .try_push(item)
            .expect("fixture structure fits model depth limits");
    }
    structure
}

fn tag(raw_tag: u32) -> Tag {
    RawTag::new(raw_tag)
        .expect("fixture tag fits TTLV width")
        .try_checked()
        .expect("fixture tag is allocated")
}
