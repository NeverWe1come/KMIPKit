#![cfg(test)]

//! Source-derived Vendor Attribute vector from OASIS KMIP v2.1 §4.60,
//! Table 150. The linked official case `KMIPKIT-TEST-CN01-2-42`
//! (`TC-I18N-3-21`) is unavailable in the pinned test-case source; this
//! independent vector does not claim that official case passed.

use crate::AttributeSet;
use kmipkit_ttlv::{ItemType, ValueView};

#[path = "../support/attribute_fixtures.rs"]
mod attribute_fixtures;

use attribute_fixtures::vendor_attribute_table_150;

const TABLE_150_REQUIRED_FIELDS: [(u32, ItemType); 3] = [
    (0x0042_009D, ItemType::TextString), // Vendor Identification
    (0x0042_000A, ItemType::TextString), // Attribute Name
    (0x0042_000B, ItemType::ByteString), // Attribute Value Item
];

#[test]
fn vendor_attribute_table_150_preserves_required_members_in_order() {
    let attributes = AttributeSet::try_new([vendor_attribute_table_150()])
        .expect("Table 150 Vendor Attribute contains its required members");
    let attribute = &attributes.as_items()[0];

    assert_eq!(attribute.tag().raw(), 0x0042_0008); // Attribute item used for §4.60 Vendor Attribute
    let fields = attribute.with_value(|value| match value {
        ValueView::Structure(structure) => Some(
            structure
                .children()
                .iter()
                .map(|field| (field.tag().raw(), field.item_type()))
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    let fields = fields.expect("Vendor Attribute is a Structure");

    assert_eq!(fields, TABLE_150_REQUIRED_FIELDS);
}
