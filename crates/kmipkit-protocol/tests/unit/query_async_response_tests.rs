//! Derived generic Query Asynchronous Requests response tests; not official
//! conformance vectors. OASIS KMIP v2.1 §6.1.41, Table 286 has the open
//! `KMIPKIT-DISC-039` caption discrepancy; this test preserves the payload
//! without assigning it a typed schema.
//!
//! Traceability: `KMIPKIT-0009-FR-007`, `FR-012`.

use kmipkit_ttlv::{StructureView, Value, ValueView};

use crate::QueryAsyncRequestsResponse;
use crate::async_operation_fixtures::{
    ASYNCHRONOUS_CORRELATION_VALUE, ASYNCHRONOUS_CORRELATION_VALUES, QUERY_ASYNCHRONOUS_REQUESTS,
    item, response_message, structure,
};

const GENERIC_TAG: u32 = 0x0042_0012;

#[derive(Debug, Eq, PartialEq)]
struct SnapshotItem {
    tag: u32,
    value: SnapshotValue,
}

#[derive(Debug, Eq, PartialEq)]
enum SnapshotValue {
    Structure(Vec<SnapshotItem>),
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
    Other,
}

fn snapshot_structure(view: &StructureView<'_>) -> Vec<SnapshotItem> {
    view.children()
        .iter()
        .map(|item| SnapshotItem {
            tag: item.tag().raw(),
            value: item.with_value(snapshot_value),
        })
        .collect()
}

fn snapshot_value(view: ValueView<'_>) -> SnapshotValue {
    match view {
        ValueView::Structure(view) => SnapshotValue::Structure(snapshot_structure(&view)),
        ValueView::Integer(value) => SnapshotValue::Integer(*value),
        ValueView::LongInteger(value) => SnapshotValue::LongInteger(*value),
        ValueView::BigInteger(value) => SnapshotValue::BigInteger(value.to_vec()),
        ValueView::Enumeration(value) => SnapshotValue::Enumeration(*value),
        ValueView::Boolean(value) => SnapshotValue::Boolean(*value),
        ValueView::TextString(value) => SnapshotValue::TextString(value.to_owned()),
        ValueView::ByteString(value) => SnapshotValue::ByteString(value.to_vec()),
        ValueView::DateTime(value) => SnapshotValue::DateTime(*value),
        ValueView::Interval(value) => SnapshotValue::Interval(*value),
        ValueView::DateTimeExtended(value) => SnapshotValue::DateTimeExtended(*value),
        _ => SnapshotValue::Other,
    }
}

#[test]
fn query_payload_survives_typed_view_and_ttlv_ownership_round_trip() {
    let async_values = structure([
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(vec![0x00, 0xff, 0x80]),
        ),
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::enumeration(0xdead_beef),
        ),
    ]);
    let payload = structure([
        item(
            ASYNCHRONOUS_CORRELATION_VALUES,
            Value::structure(async_values),
        ),
        item(GENERIC_TAG, Value::enumeration(0xdead_beef)),
        item(GENERIC_TAG, Value::byte_string(b"opaque-repeat".to_vec())),
    ]);
    let response = response_message(QUERY_ASYNCHRONOUS_REQUESTS, 0, None, None, Some(payload));
    let before = response.with_ttlv(|view| snapshot_structure(&view));
    let response_item = response
        .batch_items()
        .next()
        .expect("one validated Query response item exists");
    let typed = QueryAsyncRequestsResponse::try_from_response_item(response_item)
        .expect("the open Table 286 discrepancy permits a generic operation result");

    let observed_payload = typed
        .with_response_payload(|view| snapshot_structure(&view))
        .expect("the generic Response Payload remains available");
    assert_eq!(observed_payload[0].tag, ASYNCHRONOUS_CORRELATION_VALUES);
    assert_eq!(
        observed_payload[1..]
            .iter()
            .map(|item| item.tag)
            .collect::<Vec<_>>(),
        [GENERIC_TAG, GENERIC_TAG]
    );
    let generic_tree = response.into_ttlv();
    assert_eq!(snapshot_structure(&generic_tree.view()), before);
}
