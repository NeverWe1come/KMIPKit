//! Client Query Asynchronous Requests behavior derived from OASIS KMIP v2.1
//! §6.1.41, Tables 285–287, and §8.6, Table 399; these are not official vectors.
//!
//! Traceability: KMIPKIT-0009-FR-006–FR-010. The typed Table 286 response
//! mapping remains excluded while KMIPKIT-DISC-039 is open.

use kmipkit_protocol::QueryAsyncRequestsRequest;
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;

use crate::asynchronous_execution_test_support::{client_for, request_contains};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};

const QUERY: u32 = 0x0000_0039;
const ASYNCHRONOUS_CORRELATION_VALUES: u32 = 0x0042_0176;
const CORRELATION: &[u8] = b"QUERY_ASYNC_CORRELATION_SENTINEL";
const GENERIC_TAG: u32 = 0x0042_0199;

#[test]
fn query_executes_once_and_exposes_generic_response_structure() {
    let opaque_payload = test_structure([
        test_item(
            ASYNCHRONOUS_CORRELATION_VALUES,
            Value::structure(test_structure([test_item(
                0x0042_0006,
                Value::byte_string(vec![0x00, 0xff, 0x80]),
            )])),
        ),
        test_item(GENERIC_TAG, Value::enumeration(0xdead_beef)),
    ]);
    let response = asynchronous_response_bytes(QUERY, 0, None, None, Some(opaque_payload));
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let request = QueryAsyncRequestsRequest::new()
        .with_correlation_values([CORRELATION.to_vec(), CORRELATION.to_vec()])
        .with_operations([0x1a, 0xdead_beef]);
    assert!(!format!("{request:?}").contains("QUERY_ASYNC_CORRELATION"));

    let result = client
        .execute_query_async_requests(request, None, &CodecLimits::defaults())
        .expect("Query response remains generic while DISC-039 is unresolved");

    assert_eq!(result.result().status().raw(), 0);
    assert_eq!(
        result.with_response_payload(|payload| payload.children().len()),
        Some(2)
    );
    assert!(request_contains(&captured, CORRELATION));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn query_preserves_delivery_evidence_without_retry() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = client
        .execute_query_async_requests(
            QueryAsyncRequestsRequest::new(),
            None,
            &CodecLimits::defaults(),
        )
        .expect_err("a failed explicit Query is not retried");

    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
