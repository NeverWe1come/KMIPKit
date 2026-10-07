//! Project-authored conformance vectors from OASIS KMIP Specification v2.1.
//!
//! Traceability: §7.13 Table 365; §8.3 Table 396; §9.13 Table 418;
//! §11.44 Table 476; §11.56 tag allocations; KMIPKIT-0012-FR-009 and
//! KMIPKIT-0012-FR-012.

#[path = "../../../tests/fixtures/extensions/oasis/mod.rs"]
mod oasis;

use kmipkit_protocol::extension;
use kmipkit_ttlv::{ItemType, ValueView};

#[test]
fn table_365_extension_information_uses_exact_tags_types_order_and_requiredness() {
    let required_only = extension::to_ttlv(
        extension::extension_information("fixture-extension")
            .expect("Table 365 requires Extension Name"),
    )
    .expect("the required Table 365 field is encodable");
    let required = required_only.view();
    assert_eq!(required.children().len(), 1);
    assert_eq!(required.children()[0].tag().raw(), 0x0042_00A5);
    assert_eq!(required.children()[0].item_type(), ItemType::TextString);

    let all_fields = extension::to_ttlv(oasis::extension_information_with_all_fields())
        .expect("every optional Table 365 field is encodable");
    let view = all_fields.view();
    assert_eq!(
        view.children()
            .iter()
            .map(|child| child.tag().raw())
            .collect::<Vec<_>>(),
        [
            0x0042_00A5, // Extension Name
            0x0042_00A6, // Extension Tag
            0x0042_00A7, // Extension Type
            0x0042_0129, // Extension Enumeration (tag allocation in §11.56)
            0x0042_012A, // Extension Attribute
            0x0042_012B, // Extension Parent Structure Tag
            0x0042_012C, // Extension Description
        ]
    );
    assert_eq!(
        view.children()
            .iter()
            .map(kmipkit_ttlv::Item::item_type)
            .collect::<Vec<_>>(),
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
}

#[test]
fn tables_396_and_418_allow_repeated_extensions_with_each_required_field_in_order() {
    let request = oasis::request_with_repeated_message_extensions();
    let batch = request
        .batch_items()
        .next()
        .expect("Table 396 request contains one batch item");
    assert_eq!(batch.message_extension_count(), 2);

    let first = batch
        .message_extension(0)
        .expect("the first repeated Message Extension is present");
    let second = batch
        .message_extension(1)
        .expect("the second repeated Message Extension is present");
    assert_eq!(
        first.with_vendor_identification(str::to_owned),
        Some("example.vendor.alpha".to_owned())
    );
    assert_eq!(first.criticality_indicator(), Some(false));
    assert_eq!(
        second.with_vendor_identification(str::to_owned),
        Some("example.vendor.beta".to_owned())
    );
    assert_eq!(second.criticality_indicator(), Some(true));

    for extension in [first, second] {
        let fields = extension
            .with_ttlv(|value| {
                value
                    .children()
                    .iter()
                    .map(|child| (child.tag().raw(), child.item_type()))
                    .collect::<Vec<_>>()
            })
            .expect("Table 418 exposes the complete Message Extension structure");
        assert_eq!(
            fields,
            [
                (0x0042_009D, ItemType::TextString), // Vendor Identification
                (0x0042_0026, ItemType::Boolean),    // Criticality Indicator
                (0x0042_009C, ItemType::Structure),  // Vendor Extension
            ]
        );
    }
}

#[test]
fn table_476_query_extension_functions_are_enumeration_values_five_and_six() {
    for (name, value) in [("Query Extension List", 5), ("Query Extension Map", 6)] {
        let query_function = oasis::query_extension_function(value);
        assert_eq!(query_function.tag().raw(), 0x0042_0074);
        assert_eq!(query_function.item_type(), ItemType::Enumeration);
        assert!(
            query_function.with_value(|actual| {
                matches!(actual, ValueView::Enumeration(actual) if *actual == value)
            }),
            "Table 476 value for {name} is preserved exactly"
        );
    }
}
