//! Executable values derived from OASIS KMIP Specification v2.1 clauses.
//!
//! These are project-authored vectors, not official OASIS test cases. They
//! preserve the source table's declared tags, Item Types, and field order.

use kmipkit_protocol::{RequestMessage, extension};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value};

const REQUEST_HEADER: u32 = 0x0042_0077;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const MESSAGE_EXTENSION: u32 = 0x0042_0051;
const CRITICALITY_INDICATOR: u32 = 0x0042_0026;
const VENDOR_EXTENSION: u32 = 0x0042_009C;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const QUERY_FUNCTION: u32 = 0x0042_0074;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the OASIS vector tag fits in 24 bits")
        .try_checked()
        .expect("the OASIS vector tag is assigned by KMIP 2.1")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("the OASIS tag and Item Type form a generic Item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut value = Structure::new();
    for child in items {
        value
            .try_push(child)
            .expect("the bounded OASIS fixture fits the TTLV model depth");
    }
    value
}

/// OASIS §9.13, Table 418: Message Extension's three required children.
pub fn message_extension(vendor: &str, critical: bool, payload_tag: u32) -> Item {
    let vendor_extension = structure([item(
        payload_tag,
        Value::text_string("fixture-payload".to_owned()),
    )]);
    item(
        MESSAGE_EXTENSION,
        Value::structure(structure([
            item(
                VENDOR_IDENTIFICATION,
                Value::text_string(vendor.to_owned()),
            ),
            item(CRITICALITY_INDICATOR, Value::boolean(critical)),
            item(VENDOR_EXTENSION, Value::structure(vendor_extension)),
        ])),
    )
}

/// OASIS §8.3, Table 396: Message Extension is optional and may be repeated.
pub fn request_with_repeated_message_extensions() -> RequestMessage {
    let version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(version)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch = structure([
        item(OPERATION, Value::enumeration(0x1E)),
        item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
        message_extension("example.vendor.alpha", false, 0x0054_0001),
        message_extension("example.vendor.beta", true, 0x0054_0002),
    ]);
    RequestMessage::try_from_ttlv(structure([
        item(REQUEST_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(batch)),
    ]))
    .expect("the Table 396 request fixture is structurally valid")
}

/// OASIS §7.13, Table 365: required Extension Name plus every optional field.
pub fn extension_information_with_all_fields() -> extension::ExtensionInformation {
    let value = extension::extension_information("fixture-extension")
        .expect("Table 365 requires a Text String Extension Name");
    let value = extension::with_tag(value, 0x0054_0001)
        .expect("Extension Tag is represented by a non-negative Integer");
    let value = extension::with_type(value, ItemType::TextString)
        .expect("Extension Type is an Item Type Enumeration");
    let value = extension::with_enumeration(value, 17)
        .expect("Extension Enumeration is an Integer");
    let value = extension::with_attribute(value, false)
        .expect("Extension Attribute is Boolean");
    let value = extension::with_parent_structure_tag(value, 0x0054_0002)
        .expect("Extension Parent Structure Tag is an Integer");
    extension::with_description(value, "fixture description")
        .expect("Extension Description is a Text String")
}

/// OASIS §11.44, Table 476: Query Extension List and Query Extension Map.
pub fn query_extension_function(value: u32) -> Item {
    item(QUERY_FUNCTION, Value::enumeration(value))
}
