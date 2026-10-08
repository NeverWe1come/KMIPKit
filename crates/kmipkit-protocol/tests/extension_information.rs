//! OASIS KMIP 2.1 §7.13, Table 365 Extension Information model tests.
//!
//! Traceability: KMIPKIT-0012-FR-009.

use kmipkit_protocol::{ProtocolErrorKind, extension};
use kmipkit_ttlv::{ItemType, ValueView};

const TABLE_365_TAGS_IN_ORDER: [u32; 7] = [
    0x0042_00A5, // Extension Name
    0x0042_00A6, // Extension Tag
    0x0042_00A7, // Extension Type
    0x0042_0129, // Extension Enumeration (Tag allocation in §11.56)
    0x0042_012A, // Extension Attribute
    0x0042_012B, // Extension Parent Structure Tag
    0x0042_012C, // Extension Description
];

fn item_types(structure: &kmipkit_ttlv::Structure) -> Vec<ItemType> {
    structure
        .view()
        .children()
        .iter()
        .map(kmipkit_ttlv::Item::item_type)
        .collect()
}

#[test]
fn extension_information_requires_name_and_omits_each_optional_table_field_by_default() {
    let information = extension::extension_information("vendor-extension")
        .expect("Extension Name is the required Table 365 field");

    assert_eq!(information.name(), "vendor-extension");
    assert_eq!(information.tag(), None);
    assert_eq!(information.item_type(), None);
    assert_eq!(information.enumeration(), None);
    assert_eq!(information.attribute(), None);
    assert_eq!(information.parent_structure_tag(), None);
    assert_eq!(information.description(), None);

    let encoded = extension::to_ttlv(information).expect("required metadata encodes as TTLV");
    assert_eq!(
        encoded.view().children()[0].tag().raw(),
        TABLE_365_TAGS_IN_ORDER[0]
    );
    assert_eq!(item_types(&encoded), [ItemType::TextString]);
}

#[test]
fn extension_information_exposes_every_optional_field_in_table_order_and_type() {
    let information = extension::extension_information("vendor-extension")
        .expect("the required Extension Name is valid");
    let information = extension::with_tag(information, 0x0054_0001)
        .expect("Extension Tag is representable as a KMIP Integer");
    let information = extension::with_type(information, ItemType::TextString)
        .expect("Extension Type names a represented TTLV Item Type");
    let information = extension::with_enumeration(information, 17)
        .expect("Extension Enumeration is a non-negative KMIP Integer");
    let information = extension::with_attribute(information, false)
        .expect("false is a represented Extension Attribute value");
    let information = extension::with_parent_structure_tag(information, 0x0054_0002)
        .expect("Parent Structure Tag is representable as a KMIP Integer");
    let information = extension::with_description(information, "local metadata")
        .expect("Extension Description is valid text");

    assert_eq!(information.tag(), Some(0x0054_0001));
    assert_eq!(information.item_type(), Some(ItemType::TextString));
    assert_eq!(information.enumeration(), Some(17));
    assert_eq!(information.attribute(), Some(false));
    assert_eq!(information.parent_structure_tag(), Some(0x0054_0002));
    assert_eq!(information.description(), Some("local metadata"));

    let encoded = extension::to_ttlv(information).expect("all Table 365 fields encode");
    let encoded_view = encoded.view();
    let items = encoded_view.children();
    assert_eq!(
        items
            .iter()
            .map(|item| item.tag().raw())
            .collect::<Vec<_>>(),
        TABLE_365_TAGS_IN_ORDER
    );
    assert_eq!(
        item_types(&encoded),
        [
            ItemType::TextString,
            ItemType::Integer,
            ItemType::Enumeration,
            ItemType::Integer,
            ItemType::Boolean,
            ItemType::Integer,
            ItemType::TextString,
        ]
    );
    assert!(items[4].with_value(|value| matches!(value, ValueView::Boolean(false))));
}

#[test]
fn extension_type_maps_every_supported_ttlv_type_to_its_table_365_enumeration() {
    for (item_type, expected_enumeration) in [
        (ItemType::Structure, 0x01),
        (ItemType::Integer, 0x02),
        (ItemType::LongInteger, 0x03),
        (ItemType::BigInteger, 0x04),
        (ItemType::Enumeration, 0x05),
        (ItemType::Boolean, 0x06),
        (ItemType::TextString, 0x07),
        (ItemType::ByteString, 0x08),
        (ItemType::DateTime, 0x09),
        (ItemType::Interval, 0x0A),
        (ItemType::DateTimeExtended, 0x0B),
    ] {
        let information = extension::extension_information("vendor-extension")
            .expect("Extension Name is required");
        let information = extension::with_type(information, item_type)
            .expect("each represented TTLV type has a Table 365 code");
        let encoded =
            extension::to_ttlv(information).expect("Extension Type encodes as an Enumeration");
        let enumeration = encoded.view().children()[1].with_value(|value| match value {
            ValueView::Enumeration(value) => Some(*value),
            _ => None,
        });

        assert_eq!(enumeration, Some(expected_enumeration));
    }
}

#[test]
fn extension_information_rejects_invalid_identity_and_out_of_range_integer_fields() {
    assert_eq!(
        extension::extension_information("")
            .expect_err("empty Extension Name is invalid")
            .kind(),
        ProtocolErrorKind::InvalidIdentity
    );

    let information =
        extension::extension_information("vendor-extension").expect("Extension Name is required");
    assert_eq!(
        extension::with_tag(information.clone(), 0x0100_0000)
            .expect_err("Extension Tag must fit 24 bits")
            .kind(),
        ProtocolErrorKind::InvalidSchema
    );
    assert_eq!(
        extension::with_enumeration(information.clone(), u32::MAX)
            .expect_err("Extension Enumeration must fit a signed KMIP Integer")
            .kind(),
        ProtocolErrorKind::InvalidSchema
    );
    assert_eq!(
        extension::with_parent_structure_tag(information, 0x0100_0000)
            .expect_err("Parent Structure Tag must fit 24 bits")
            .kind(),
        ProtocolErrorKind::InvalidSchema
    );
}
