//! Source-derived KMIP 2.1 asynchronous operation fixtures.
//!
//! The field order and result cases follow OASIS KMIP v2.1 §§6.1.5, 6.1.38,
//! 6.1.39, and 6.1.41, Tables 176–178, 276–280, and 285–287. Query response
//! fixtures remain generic while `KMIPKIT-DISC-039` is open. These are derived
//! fixtures, not official conformance vectors.

use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

use crate::ResponseMessage;

pub(crate) const CANCEL: u32 = 0x0000_0019;
pub(crate) const POLL: u32 = 0x0000_001A;
pub(crate) const PROCESS: u32 = 0x0000_003A;
pub(crate) const QUERY_ASYNCHRONOUS_REQUESTS: u32 = 0x0000_0039;
pub(crate) const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
pub(crate) const CANCELLATION_RESULT: u32 = 0x0042_0012;
pub(crate) const OPERATIONS: u32 = 0x0042_014F;
pub(crate) const ASYNCHRONOUS_CORRELATION_VALUES: u32 = 0x0042_0176;
pub(crate) const OPERATION: u32 = 0x0042_005C;

const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;

pub(crate) fn response_message(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    correlation_value: Option<&[u8]>,
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
    let mut batch = vec![item(OPERATION, Value::enumeration(operation))];
    batch.push(item(RESULT_STATUS, Value::enumeration(status)));
    if let Some(reason) = reason {
        batch.push(item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(value) = correlation_value {
        batch.push(item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(value.to_vec()),
        ));
    }
    if let Some(payload) = payload {
        batch.push(item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    let tree = structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(structure(batch))),
    ]);
    ResponseMessage::try_from_ttlv(tree).expect("fixture is a valid KMIP response message")
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
