//! Project-authored conformance vectors from OASIS KMIP Specification v2.1.
//!
//! Traceability: §7.13 Table 365; §8.3 Table 396; §9.13 Table 418;
//! §11.44 Table 476; §11.56 tag allocations; KMIPKIT-0012-FR-009 and
//! KMIPKIT-0012-FR-012.

#[path = "../../../tests/fixtures/extensions/oasis/mod.rs"]
mod oasis;

use kmipkit_protocol::extension::{self, ExtensionDefinition};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

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

#[test]
fn schema_fuzz_corpus_seed_decodes_and_reaches_complete_validation() {
    const SEED: &[u8] = include_bytes!("../../../fuzz/corpus/extension_schema/valid_nested.ttlv");
    let codec_limits = CodecLimits::new(4 * 1024, 64, 512)
        .expect("the fuzz target uses supported bounded decoder limits");
    let decoded = decode_with_limits(SEED, &codec_limits)
        .expect("the checked-in fuzz seed is one valid bounded TTLV Structure");
    assert_eq!(decoded.tag().raw(), 0x0042_0001);
    assert_eq!(decoded.item_type(), ItemType::Structure);

    let payload = decoded
        .with_value(|value| match value {
            ValueView::Structure(value) => clone_structure(&value),
            _ => None,
        })
        .expect("the valid seed contains a cloneable Structure");
    let definition = schema_fuzz_definition();
    let validated = extension::validate(&definition, payload, &codec_limits)
        .expect("the seed reaches and passes the fixed extension schema");
    let children = extension::generic_value(&validated).view();
    let nested = &children.children()[0];
    assert_eq!(nested.tag().raw(), 0x0054_0010);
    assert!(nested.with_value(|value| matches!(value, ValueView::Structure(_))));
}

fn schema_fuzz_definition() -> ExtensionDefinition {
    let discriminator = extension::required(
        tag(0x0054_0011),
        extension::scalar(ItemType::TextString).expect("TextString is supported"),
    )
    .expect("the discriminator rule is valid");
    let integer = extension::optional(
        tag(0x0054_0012),
        extension::with_signed_range(
            extension::scalar(ItemType::Integer).expect("Integer is supported"),
            0,
            i64::from(i32::MAX),
        )
        .expect("the Integer range is valid"),
    )
    .expect("the optional Integer rule is valid");
    let enumeration = extension::repeated(
        tag(0x0054_0013),
        extension::with_allowed_enumeration(
            extension::scalar(ItemType::Enumeration).expect("Enumeration is supported"),
            1,
        )
        .expect("the Enumeration allow-list is valid"),
    )
    .expect("the repeated Enumeration rule is valid");
    let nested = extension::structure(
        vec![discriminator, integer, enumeration],
        vec![
            extension::extension_order_constraint(tag(0x0054_0012), tag(0x0054_0013))
                .expect("order tags are distinct"),
        ],
        true,
    )
    .expect("the nested extension schema is valid");
    let root = extension::structure(
        vec![
            extension::required(tag(0x0054_0010), nested)
                .expect("the nested Structure rule is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("the extension root schema is valid");
    let identity = extension::extension_identity("fuzz.vendor", "bounded-schema", "1")
        .expect("the fuzz identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the version range includes this build");
    let path = extension::with_child_tag(
        extension::ttlv_path(tag(0x0054_0010)).expect("the discriminator path is valid"),
        tag(0x0054_0011),
    )
    .expect("the discriminator path is nested");
    let discriminator = extension::discriminator(path, Value::text_string("fuzz-v1".to_owned()))
        .expect("the discriminator scalar is valid");
    extension::extension_definition(identity, compatibility, discriminator, root)
        .expect("the fuzz extension definition is valid")
}

fn clone_structure(source: &kmipkit_ttlv::StructureView<'_>) -> Option<Structure> {
    let mut clone = Structure::new();
    for child in source.children() {
        let value = child.with_value(clone_value)?;
        clone.try_push(Item::new(child.tag(), value).ok()?).ok()?;
    }
    Some(clone)
}

fn clone_value(source: ValueView<'_>) -> Option<Value> {
    match source {
        ValueView::Structure(value) => Some(Value::structure(clone_structure(&value)?)),
        ValueView::Integer(value) => Some(Value::integer(*value)),
        ValueView::LongInteger(value) => Some(Value::long_integer(*value)),
        ValueView::BigInteger(value) => Some(Value::big_integer(value.to_vec())),
        ValueView::Enumeration(value) => Some(Value::enumeration(*value)),
        ValueView::Boolean(value) => Some(Value::boolean(*value)),
        ValueView::TextString(value) => Some(Value::text_string((*value).to_owned())),
        ValueView::ByteString(value) => Some(Value::byte_string(value.to_vec())),
        ValueView::DateTime(value) => Some(Value::date_time(*value)),
        ValueView::Interval(value) => Some(Value::interval(*value)),
        ValueView::DateTimeExtended(value) => Some(Value::date_time_extended(*value)),
        _ => None,
    }
}

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the fixture tag fits the KMIP Tag width")
        .try_checked()
        .expect("the fixture tag is allocation-valid")
}
