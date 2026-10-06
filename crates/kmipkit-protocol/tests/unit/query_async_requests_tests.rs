//! Derived KMIP 2.1 Query Asynchronous Requests tests; not official vectors.
//!
//! Traceability: `KMIPKIT-0009-FR-001`, `FR-006`, `FR-009`; OASIS KMIP v2.1
//! §6.1.41, Table 285, and §7.1, Table 352. Response interpretation remains
//! generic while `KMIPKIT-DISC-039` is open.

use kmipkit_ttlv::ValueView;

use crate::QueryAsyncRequestsRequest;
use crate::async_operation_fixtures::{
    ASYNCHRONOUS_CORRELATION_VALUE, ASYNCHRONOUS_CORRELATION_VALUES, OPERATION, OPERATIONS,
};

#[test]
fn query_filters_preserve_absence_order_repetition_and_arbitrary_bytes() {
    let absent = QueryAsyncRequestsRequest::new()
        .to_ttlv_payload()
        .expect("Query permits absent optional filters");
    assert!(absent.view().children().is_empty());

    let first = vec![0x00, 0xff, 0x80];
    let second = vec![0x81, 0x00];
    let query = QueryAsyncRequestsRequest::new()
        .with_correlation_values([first.clone(), second.clone(), first.clone()])
        .with_operations([0x1a, 0xdead_beef, 0x1a]);
    let payload = query
        .to_ttlv_payload()
        .expect("Query request filter structures follow Table 285");
    let fields = payload.view().children();

    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].tag().raw(), ASYNCHRONOUS_CORRELATION_VALUES);
    assert_eq!(fields[1].tag().raw(), OPERATIONS);
    let correlations = fields[0].with_value(|value| match value {
        ValueView::Structure(value) => Some(
            value
                .children()
                .iter()
                .map(|item| {
                    assert_eq!(item.tag().raw(), ASYNCHRONOUS_CORRELATION_VALUE);
                    item.with_value(|field| match field {
                        ValueView::ByteString(bytes) => bytes.to_vec(),
                        _ => Vec::new(),
                    })
                })
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert_eq!(correlations, Some(vec![first.clone(), second, first]));
    let operations = fields[1].with_value(|value| match value {
        ValueView::Structure(value) => Some(
            value
                .children()
                .iter()
                .map(|item| {
                    assert_eq!(item.tag().raw(), OPERATION);
                    item.with_value(|field| match field {
                        ValueView::Enumeration(raw) => *raw,
                        _ => 0,
                    })
                })
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert_eq!(operations, Some(vec![0x1a, 0xdead_beef, 0x1a]));
    assert!(!format!("{query:?}").contains("255"));
}

#[test]
fn query_response_remains_an_opaque_generic_ttlv_structure() {
    use crate::QueryAsyncRequestsResponse;
    use crate::async_operation_fixtures::{
        QUERY_ASYNCHRONOUS_REQUESTS, item, response_message, structure,
    };
    use kmipkit_ttlv::Value;

    let nested = structure([
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(vec![0x00, 0xff]),
        ),
        item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::enumeration(0xdead_beef),
        ),
    ]);
    let payload = structure([
        item(ASYNCHRONOUS_CORRELATION_VALUES, Value::structure(nested)),
        item(OPERATIONS, Value::structure(structure([]))),
    ]);
    let message = response_message(QUERY_ASYNCHRONOUS_REQUESTS, 0, None, None, Some(payload));
    let item = message
        .batch_items()
        .next()
        .expect("one response item exists");
    let query = QueryAsyncRequestsResponse::try_from_response_item(item)
        .expect("the unresolved Table 286 response remains generic");

    assert_eq!(query.result().status().raw(), 0);
    assert_eq!(
        query.with_response_payload(|payload| payload.children().len()),
        Some(2)
    );
}
