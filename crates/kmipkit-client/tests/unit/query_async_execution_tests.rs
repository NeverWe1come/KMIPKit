//! Client Query Asynchronous Requests behavior derived from OASIS KMIP v2.1
//! §6.1.41, Tables 285–287, §8.6, Table 399, and §9.12, Table 417; these are
//! not official vectors.
//!
//! Traceability: KMIPKIT-0009-FR-006–FR-010. The typed Table 286 response
//! mapping remains excluded while KMIPKIT-DISC-039 is open.

use std::cell::Cell;
use std::rc::Rc;

use kmipkit_protocol::QueryAsyncRequestsRequest;
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Value, ValueView};

use crate::asynchronous_execution_test_support::{
    client_for, client_for_with_request_observer, request_contains,
};
use crate::execute::Client;
use crate::execute::ZeroizationObserver;
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};

const QUERY: u32 = 0x0000_0039;
const ASYNCHRONOUS_CORRELATION_VALUES: u32 = 0x0042_0176;
const CORRELATION: &[u8] = b"QUERY_ASYNC_CORRELATION_SENTINEL";
const RESPONSE_SECRET: &[u8] = b"QUERY_RESPONSE_SECRET_SENTINEL";
const GENERIC_TAG: u32 = 0x0042_0012;

struct CapIgnoringTransport {
    response: Vec<u8>,
    exchange_count: Rc<Cell<usize>>,
}

impl Transport for CapIgnoringTransport {
    fn exchange(
        &mut self,
        _request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.exchange_count.set(self.exchange_count.get() + 1);
        Ok(TransportResponse::new(std::mem::take(&mut self.response)))
    }
}

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
        test_item(GENERIC_TAG, Value::byte_string(RESPONSE_SECRET.to_vec())),
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
        result.with_response_payload(|payload| {
            payload
                .children()
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>()
        }),
        Some(vec![
            ASYNCHRONOUS_CORRELATION_VALUES,
            GENERIC_TAG,
            GENERIC_TAG,
        ])
    );
    assert_eq!(
        result.with_response_payload(|payload| {
            payload.children()[1].with_value(|value| match value {
                ValueView::ByteString(bytes) => bytes == RESPONSE_SECRET,
                _ => false,
            })
        }),
        Some(true)
    );
    assert!(request_contains(&captured, CORRELATION));
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert!(!format!("{result:?}").contains("QUERY_RESPONSE_SECRET"));
}

#[test]
fn query_filter_encoded_request_copy_is_zeroized_after_success() {
    let response = asynchronous_response_bytes(QUERY, 0, None, None, Some(test_structure([])));
    let observer = ZeroizationObserver::new(None);
    let (mut client, fake, captured) = client_for_with_request_observer(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        observer.clone(),
    );
    // Keep one caller-owned allocation and transfer a separate Vec to KMIPKit.
    let caller_owned_copy = CORRELATION.to_vec();
    let transferred_filter = caller_owned_copy.clone();

    client
        .execute_query_async_requests(
            QueryAsyncRequestsRequest::new().with_correlation_values([transferred_filter]),
            None,
            &CodecLimits::defaults(),
        )
        .expect("a valid generic Query response succeeds");

    assert!(request_contains(&captured, CORRELATION));
    assert_eq!(observer.result(), Some(true));
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(caller_owned_copy, CORRELATION);
}

#[test]
fn query_filter_encoded_request_copy_is_zeroized_after_exchange_error() {
    let observer = ZeroizationObserver::new(None);
    let (mut client, fake, captured) = client_for_with_request_observer(
        ExchangeScript::FailAfterPartialWrite { written_bytes: 3 },
        observer.clone(),
    );

    let error = client
        .execute_query_async_requests(
            QueryAsyncRequestsRequest::new().with_correlation_values([CORRELATION.to_vec()]),
            None,
            &CodecLimits::defaults(),
        )
        .expect_err("the scripted exchange fails after sending part of the request");

    assert!(matches!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    ));
    assert!(request_contains(&captured, CORRELATION));
    assert_eq!(observer.result(), Some(true));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn query_preserves_delivery_evidence_without_retry() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = client
        .execute_query_async_requests(
            QueryAsyncRequestsRequest::new().with_correlation_values([CORRELATION.to_vec()]),
            None,
            &CodecLimits::defaults(),
        )
        .expect_err("a failed explicit Query is not retried");

    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert!(!error.to_string().contains("QUERY_ASYNC_CORRELATION"));
    assert!(!format!("{error:?}").contains("QUERY_ASYNC_CORRELATION"));
}

#[test]
fn query_filter_encoding_obeys_the_per_call_message_limit_before_transport() {
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response: Vec::new(),
        request_write_chunks: Vec::new(),
    });
    let limits = CodecLimits::new(8, 64, 100_000)
        .expect("the configured structure depth remains within the model limit");
    let error = client
        .execute_query_async_requests(
            QueryAsyncRequestsRequest::new().with_correlation_values([CORRELATION.to_vec()]),
            None,
            &limits,
        )
        .expect_err("an oversized Query filter must fail before transport");

    assert_eq!(error.category(), crate::ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::NotSent)
    );
    assert_eq!(fake.borrow().exchange_count(), 0);
    assert!(captured.borrow().is_none());
    assert!(!error.to_string().contains("QUERY_ASYNC_CORRELATION"));
    assert!(!format!("{error:?}").contains("QUERY_ASYNC_CORRELATION"));
}

#[test]
fn query_rejects_a_response_over_the_per_call_message_limit_after_one_exchange() {
    let response = asynchronous_response_bytes(
        QUERY,
        0,
        None,
        None,
        Some(test_structure([test_item(
            GENERIC_TAG,
            Value::byte_string(vec![0xa5; 512]),
        )])),
    );
    let exchange_count = Rc::new(Cell::new(0));
    let transport = CapIgnoringTransport {
        response,
        exchange_count: Rc::clone(&exchange_count),
    };
    let mut client = Client::for_test(transport);
    let limits = CodecLimits::new(256, 64, 100_000)
        .expect("the configured structure depth remains within the model limit");

    let error = client
        .execute_query_async_requests(QueryAsyncRequestsRequest::new(), None, &limits)
        .expect_err("the client rejects a response over the caller's message-size limit");

    assert_eq!(error.category(), crate::ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(exchange_count.get(), 1);
}
