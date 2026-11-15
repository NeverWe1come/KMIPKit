//! Derived client-boundary tests for bounded response handling.
//! These are not official OASIS test vectors.
//!
//! OASIS KMIP Specification v2.1 §9.12 and Table 417; ADR-0014. The transport
//! crate tests the response wrapper's current initialized allocation cleanup.
//! This module verifies that the client checks a deliberately non-compliant
//! response wrapper before entering its decoder.
//!
//! Traceability: `KMIPKIT-0007-FR-005`, `-FR-009`, `-FR-013`, `-SC-003`,
//! `KMIPKIT-REQ-SPEC-9.12-001-002`, `-001-003`, and ADR-0014.

use std::cell::Cell;
use std::rc::Rc;

use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::CodecLimits;

use crate::execute::{
    Client, ClientBatch, ClientBatchItem, ClientRequest, LimitsIdentityObserver,
    request_message_for_test,
};
use crate::execute_test_support::{ResponseItemFixture, response_bytes};
const RESPONSE_DEBUG_SENTINEL: &[u8] = b"KMIP_RESPONSE_DEBUG_SENTINEL";

struct OversizedTransport {
    exchange_count: Rc<Cell<usize>>,
    last_response_cap: Rc<Cell<Option<usize>>>,
}

impl Default for OversizedTransport {
    fn default() -> Self {
        Self {
            exchange_count: Rc::new(Cell::new(0)),
            last_response_cap: Rc::new(Cell::new(None)),
        }
    }
}

impl Transport for OversizedTransport {
    fn exchange(
        &mut self,
        _request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.exchange_count.set(self.exchange_count.get() + 1);
        self.last_response_cap.set(Some(max_response_bytes));
        Ok(TransportResponse::new(vec![0xA5; max_response_bytes + 1]))
    }
}

struct ExactLimitTransport {
    response: Vec<u8>,
    exchange_count: Rc<Cell<usize>>,
    last_response_cap: Rc<Cell<Option<usize>>>,
}

impl Transport for ExactLimitTransport {
    fn exchange(
        &mut self,
        _request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.exchange_count.set(self.exchange_count.get() + 1);
        self.last_response_cap.set(Some(max_response_bytes));
        Ok(TransportResponse::new(self.response.clone()))
    }
}

fn limits_with_response_cap(cap: usize) -> CodecLimits {
    CodecLimits::new(
        cap,
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
        CodecLimits::DEFAULT_MAX_ELEMENTS,
    )
    .expect("the fixture uses supported decoder limits")
}

#[test]
fn client_rejects_oversized_transport_response_before_decoder_entry() {
    let limits = limits_with_response_cap(256);
    let observer = LimitsIdentityObserver::new(&limits);
    let transport = OversizedTransport::default();
    let exchange_count = Rc::clone(&transport.exchange_count);
    let last_response_cap = Rc::clone(&transport.last_response_cap);
    let mut client = Client::for_test_with_limits_observer(transport, observer.clone());
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));

    let error = client
        .execute(batch, &limits)
        .expect_err("client rejects a non-compliant oversized response wrapper");

    assert_eq!(exchange_count.get(), 1);
    assert_eq!(last_response_cap.get(), Some(limits.max_message_bytes()));
    assert_eq!(observer.decode_calls(), 0);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
}

#[test]
fn client_accepts_a_valid_transport_response_exactly_at_the_limit() {
    let response = response_bytes((2, 1), &[ResponseItemFixture::success(None)]);
    let cap = response.len();
    let limits = limits_with_response_cap(cap);
    let exchange_count = Rc::new(Cell::new(0));
    let last_response_cap = Rc::new(Cell::new(None));
    let transport = ExactLimitTransport {
        response,
        exchange_count: Rc::clone(&exchange_count),
        last_response_cap: Rc::clone(&last_response_cap),
    };
    let mut client = Client::for_test(transport);
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));

    let result = client
        .execute(batch, &limits)
        .expect("a valid response at the exact configured cap is accepted");

    assert_eq!(exchange_count.get(), 1);
    assert_eq!(last_response_cap.get(), Some(cap));
    assert_eq!(result.len(), 1);
}

#[test]
fn empty_successful_transport_response_preserves_possibly_sent_state() {
    let limits = limits_with_response_cap(256);
    let exchange_count = Rc::new(Cell::new(0));
    let transport = ExactLimitTransport {
        response: Vec::new(),
        exchange_count: Rc::clone(&exchange_count),
        last_response_cap: Rc::new(Cell::new(None)),
    };
    let mut client = Client::for_test(transport);
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));

    let error = client
        .execute(batch, &limits)
        .expect_err("an empty response cannot decode as a KMIP message");

    assert_eq!(exchange_count.get(), 1);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
}

#[test]
fn response_wrapper_debug_redacts_initialized_response_bytes() {
    let response = TransportResponse::new(RESPONSE_DEBUG_SENTINEL.to_vec());

    assert_eq!(response.as_bytes(), RESPONSE_DEBUG_SENTINEL);
    assert!(!format!("{response:?}").contains("KMIP_RESPONSE_DEBUG_SENTINEL"));
}

#[test]
fn discover_versions_omits_peer_response_cap_and_server_correlation_fields() {
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
    let request = request_message_for_test(batch, &CodecLimits::defaults())
        .expect("typed Discover Versions request is valid");

    assert_eq!(request.header().maximum_response_size(), None);
    let has_server_correlation_value = request.with_ttlv(|root| {
        root.children().iter().any(|item| {
            item.tag().raw() == 0x0042_0077
                && item.with_value(|value| match value {
                    kmipkit_ttlv::ValueView::Structure(header) => header
                        .children()
                        .iter()
                        .any(|field| field.tag().raw() == 0x0042_0106),
                    _ => false,
                })
        })
    });
    assert!(!has_server_correlation_value);
}
