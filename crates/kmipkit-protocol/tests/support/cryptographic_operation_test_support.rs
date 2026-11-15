//! Shared fixtures for source-derived cryptographic operation tests.

use crate::{ResponseBatchItemView, ResponseMessage, SecretBytes};
use kmipkit_ttlv::{Item, Structure, StructureView, Value, ValueView};

/// Builds a TTLV Structure from operation payload fields.
pub(crate) fn payload(items: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    let mut payload = Structure::new();
    for (raw_tag, value) in items {
        payload
            .try_push(crate::operation_test_support::item(raw_tag, value))
            .expect("source-derived operation fixture fits the TTLV Structure limits");
    }
    payload
}

/// Returns raw tags in the order supplied to a TTLV Structure.
pub(crate) fn field_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

/// Returns the single response batch item in an operation fixture.
pub(crate) fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("the fixture contains one validated operation response item")
}

/// Checks a response-owned generic TTLV tree for a tag at any nesting level.
pub(crate) fn structure_contains_tag(structure: &StructureView<'_>, sought: u32) -> bool {
    structure.children().iter().any(|item| {
        item.tag().raw() == sought
            || item.with_value(|value| match value {
                ValueView::Structure(nested) => structure_contains_tag(&nested, sought),
                _ => false,
            })
    })
}

/// Asserts the opaque, zeroizing byte wrapper retains exact server bytes.
pub(crate) fn output_bytes(value: Option<&SecretBytes>, expected: &[u8]) {
    value
        .expect("the source-derived operation response includes its byte string")
        .with_bytes(|actual| assert_eq!(actual, expected));
}

/// Copies one test-only TTLV Byte String for wire-value assertions.
pub(crate) fn snapshot(item: &Item) -> (u32, Vec<u8>) {
    (
        item.tag().raw(),
        item.with_value(|value| match value {
            ValueView::ByteString(value) => Some(value.to_vec()),
            _ => None,
        })
        .expect("the selected field is a Byte String"),
    )
}
