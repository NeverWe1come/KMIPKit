//! Derived KMIP 2.1 Query Asynchronous Requests tests; not official vectors.
//!
//! Traceability: `KMIPKIT-0009-FR-001`, `FR-006`, `FR-009`; OASIS KMIP v2.1
//! §6.1.41, Table 285, and §7.1, Table 352. Response interpretation remains
//! generic while `KMIPKIT-DISC-039` is open.

use kmipkit_ttlv::ValueView;

use crate::async_operation_fixtures::{
    ASYNCHRONOUS_CORRELATION_VALUE, ASYNCHRONOUS_CORRELATION_VALUES, OPERATION, OPERATIONS,
};
use crate::{AsynchronousOperationError, QueryAsyncRequestsRequest};

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
    let view = payload.view();
    let fields = view.children();

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
fn query_preserves_present_but_empty_filter_structures() {
    let query = QueryAsyncRequestsRequest::new()
        .with_correlation_values(std::iter::empty::<Vec<u8>>())
        .with_operations(std::iter::empty());
    let payload = query
        .to_ttlv_payload()
        .expect("empty filter structures remain distinguishable from absent filters");
    let view = payload.view();
    let fields = view.children();

    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].tag().raw(), ASYNCHRONOUS_CORRELATION_VALUES);
    assert_eq!(fields[1].tag().raw(), OPERATIONS);
    for field in fields {
        assert!(field.with_value(|value| match value {
            ValueView::Structure(value) => value.children().is_empty(),
            _ => false,
        }));
    }
}

#[test]
fn query_encodes_each_optional_filter_independently() {
    let correlations = QueryAsyncRequestsRequest::new().with_correlation_values([b"corr".to_vec()]);
    let correlation_payload = correlations
        .to_ttlv_payload()
        .expect("Table 285 permits only the correlation filter");
    let correlation_view = correlation_payload.view();
    let fields = correlation_view.children();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), ASYNCHRONOUS_CORRELATION_VALUES);

    let operations = QueryAsyncRequestsRequest::new().with_operations([0x1a]);
    let operation_payload = operations
        .to_ttlv_payload()
        .expect("Table 285 permits only the Operation filter");
    let operation_view = operation_payload.view();
    let fields = operation_view.children();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), OPERATIONS);
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

#[test]
fn query_response_keeps_failure_shape_generic_and_rejects_other_operations() {
    use crate::QueryAsyncRequestsResponse;
    use crate::async_operation_fixtures::{POLL, QUERY_ASYNCHRONOUS_REQUESTS, response_message};

    let failure = response_message(QUERY_ASYNCHRONOUS_REQUESTS, 1, Some(1), None, None);
    let item = failure
        .batch_items()
        .next()
        .expect("one response item exists");
    let query = QueryAsyncRequestsResponse::try_from_response_item(item)
        .expect("Query Failure follows the general response shape");
    assert_eq!(
        query.with_response_payload(|payload| payload.children().len()),
        None
    );
    assert!(format!("{query:?}").contains("has_response_payload: false"));

    let other_operation = response_message(POLL, 1, Some(1), None, None);
    let other_response_item = other_operation
        .batch_items()
        .next()
        .expect("one response item exists");
    assert!(matches!(
        QueryAsyncRequestsResponse::try_from_response_item(other_response_item),
        Err(AsynchronousOperationError::UnexpectedOperation)
    ));
}
