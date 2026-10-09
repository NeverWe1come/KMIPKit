//! Fake-transport Query behavior derived from OASIS KMIP v2.1 §6.1.40.
//! These structural vectors do not claim an official OASIS fixture pass.

use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::ValueView;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{ClientBatchOutcome, ClientOperation};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use kmipkit_protocol::{QueryFunction, QueryRequest};

const QUERY: u32 = 0x0000_0018;
const BATCH_ITEM: u32 = 0x0042_000f;
const OPERATION: u32 = 0x0042_005c;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const QUERY_FUNCTION: u32 = 0x0042_0074;
const OBJECT_GROUPS: u32 = 0x0042_0166;
const OBJECT_GROUP: u32 = 0x0042_0056;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015f;

fn query_request_shape(request: &[u8]) -> Option<(Vec<u32>, Vec<String>)> {
    let decoded = decode_with_limits(request, &CodecLimits::defaults()).ok()?;
    decoded.with_value(|value| {
        let ValueView::Structure(root) = value else {
            return None;
        };
        let batch = root
            .children()
            .iter()
            .find(|field| field.tag().raw() == BATCH_ITEM)?;
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                return None;
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)?
                .with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                })?;
            if operation != QUERY {
                return None;
            }
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)?;
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    return None;
                };
                let functions = payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == QUERY_FUNCTION)
                    .map(|field| {
                        field.with_value(|value| match value {
                            ValueView::Enumeration(value) => Some(*value),
                            _ => None,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                let groups = match payload
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == OBJECT_GROUPS)
                {
                    Some(field) => field.with_value(|value| {
                        let ValueView::Structure(groups) = value else {
                            return None;
                        };
                        groups
                            .children()
                            .iter()
                            .filter(|group| group.tag().raw() == OBJECT_GROUP)
                            .map(|group| {
                                group.with_value(|value| match value {
                                    ValueView::TextString(text) => Some((*text).to_owned()),
                                    _ => None,
                                })
                            })
                            .collect::<Option<Vec<_>>>()
                    })?,
                    None => Vec::new(),
                };
                Some((functions, groups))
            })
        })
    })
}

#[test]
fn query_sends_one_request_with_repeated_functions_and_ordered_object_groups() {
    let response = asynchronous_response_bytes(
        QUERY,
        0,
        None,
        None,
        Some(test_structure([test_item(
            PROTECTION_STORAGE_MASKS,
            kmipkit_ttlv::Value::structure(test_structure([])),
        )])),
    );
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });
    let outcome = client
        .query(
            QueryRequest::new([
                QueryFunction::OPERATIONS,
                QueryFunction::OBJECTS,
                QueryFunction::OPERATIONS,
            ])
            .with_object_groups(["group-b", "group-a", "group-b"]),
            &CodecLimits::defaults(),
        )
        .expect("the fake server returns a valid structured Query response");
    let ClientBatchOutcome::Query(query) = outcome.outcome() else {
        panic!("the typed outcome is Query");
    };

    assert_eq!(query.response_fields().len(), 1);
    assert_eq!(outcome.outcome().operation(), ClientOperation::Query);
    assert_eq!(
        query_request_shape(captured.borrow().as_deref().expect("request captured")),
        Some((
            vec![1, 2, 1],
            vec!["group-b".into(), "group-a".into(), "group-b".into()]
        ))
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn empty_query_is_not_sent() {
    let (mut client, transport, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 1 });
    let error = client
        .query(QueryRequest::new([]), &CodecLimits::defaults())
        .expect_err("an empty Query Function list is rejected locally");

    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn query_failure_preserves_kmip_result_and_transport_delivery_without_retry() {
    let response = asynchronous_response_bytes(QUERY, 1, Some(1), None, None);
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let outcome = client
        .query(
            QueryRequest::new([QueryFunction::OPERATIONS]),
            &CodecLimits::defaults(),
        )
        .expect("a valid KMIP operation failure remains a typed response");
    let ClientBatchOutcome::Query(query) = outcome.outcome() else {
        panic!("the typed outcome is Query");
    };

    assert_eq!(query.result().status().raw(), 1);
    assert_eq!(query.result().reason().map(|reason| reason.raw()), Some(1));
    assert!(query.result().message().is_none());
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn query_rejects_malformed_response_after_one_exchange() {
    let response = asynchronous_response_bytes(
        QUERY,
        0,
        None,
        None,
        Some(test_structure([test_item(
            0x0042_005c,
            kmipkit_ttlv::Value::text_string("wrong type".to_owned()),
        )])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let error = client
        .query(
            QueryRequest::new([QueryFunction::OPERATIONS]),
            &CodecLimits::defaults(),
        )
        .expect_err("a known Query response member has its specified type");
    assert!(error.delivery_state().is_some());
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn query_operation_is_the_defined_kmip_operation() {
    assert_eq!(QUERY, 0x0000_0018);
}
