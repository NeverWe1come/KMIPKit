//! Source-derived structural fixtures for OASIS KMIP v2.1 §6.1.36, Tables
//! 271–272, and §6.1.40, Tables 281–284. These are not official test vectors.

use crate::{ResponseBatchItemView, ResponseMessage};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

pub(crate) const PING_OPERATION: u32 = 0x0000_003b;
pub(crate) const QUERY_OPERATION: u32 = 0x0000_0018;
pub(crate) const QUERY_FUNCTION: u32 = 0x0042_0074;
pub(crate) const OBJECT_GROUPS: u32 = 0x0042_0166;
pub(crate) const OBJECT_GROUP: u32 = 0x0042_0056;
pub(crate) const RESPONSE_HEADER: u32 = 0x0042_007a;
pub(crate) const PROTOCOL_VERSION: u32 = 0x0042_0069;
pub(crate) const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006a;
pub(crate) const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006b;
pub(crate) const TIME_STAMP: u32 = 0x0042_0092;
pub(crate) const BATCH_COUNT: u32 = 0x0042_000d;
pub(crate) const BATCH_ITEM: u32 = 0x0042_000f;
pub(crate) const OPERATION: u32 = 0x0042_005c;
pub(crate) const RESULT_STATUS: u32 = 0x0042_007f;
pub(crate) const RESULT_REASON: u32 = 0x0042_007e;
pub(crate) const RESULT_MESSAGE: u32 = 0x0042_007d;
pub(crate) const RESPONSE_PAYLOAD: u32 = 0x0042_007c;

pub(crate) const QUERY_RESPONSE_TAGS: [u32; 14] = [
    0x0042_005c, // Operation
    0x0042_0057, // Object Type
    0x0042_009d, // Vendor Identification
    0x0042_0088, // Server Information
    0x0042_0003, // Application Namespace
    0x0042_00a4, // Extension Information
    0x0042_00c7, // Attestation Type
    0x0042_00d9, // RNG Parameters
    0x0042_00eb, // Profile Information
    0x0042_00df, // Validation Information
    0x0042_00f7, // Capability Information
    0x0042_00f6, // Client Registration Method
    0x0042_0152, // Defaults Information
    0x0042_015f, // Protection Storage Masks
];

pub(crate) fn tag(raw_tag: u32) -> Tag {
    RawTag::new(raw_tag)
        .expect("fixture tag fits the 24-bit KMIP field")
        .try_checked()
        .expect("fixture tag is assigned by the KMIP 2.1 catalog")
}

pub(crate) fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("fixture item uses an allocated tag")
}

pub(crate) fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("fixture structure stays within the model depth limit");
    }
    structure
}

pub(crate) fn response_message(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    result_message: Option<&str>,
    payload: Option<Structure>,
) -> ResponseMessage {
    let version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(version)),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut batch = vec![
        item(OPERATION, Value::enumeration(operation)),
        item(RESULT_STATUS, Value::enumeration(status)),
    ];
    if let Some(reason) = reason {
        batch.push(item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(message) = result_message {
        batch.push(item(RESULT_MESSAGE, Value::text_string(message.to_owned())));
    }
    if let Some(payload) = payload {
        batch.push(item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    ResponseMessage::try_from_ttlv(structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(structure(batch))),
    ]))
    .expect("fixture is a valid KMIP 2.1 response message")
}

pub(crate) fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one response batch item")
}
