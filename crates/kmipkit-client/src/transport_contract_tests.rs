//! Derived low-level transport contract tests; these do not exercise
//! `Client::execute` or establish behavior for a production network adapter.
//!
//! Traceability: KMIPKIT-0007-FR-005/-FR-009/-FR-013 and SC-003; ADR-0014;
//! OASIS KMIP Specification v2.1 §9.12 and Table 417 for the response limit.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kmipkit_test_support::fixtures::{PARTIAL_RESPONSE, REQUEST_SENTINEL, SUCCESS_RESPONSE};
use kmipkit_test_support::{ExchangeScript, ScriptedTransport};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use zeroize::Zeroize;

const RESPONSE_LIMIT: usize = 8;

#[derive(Clone)]
struct RequestCopyObserver {
    zeroized: Arc<AtomicBool>,
    expected_len: usize,
}

impl RequestCopyObserver {
    fn new(expected_len: usize) -> Self {
        Self {
            zeroized: Arc::default(),
            expected_len,
        }
    }

    fn record_initialized_range_is_zero(&self, bytes: &[u8]) {
        let all_zero = bytes.len() == self.expected_len && bytes.iter().all(|byte| *byte == 0);
        self.zeroized.store(all_zero, Ordering::SeqCst);
    }

    fn initialized_range_was_zero(&self) -> bool {
        self.zeroized.load(Ordering::SeqCst)
    }
}

struct ObservedRequestCopy {
    bytes: Vec<u8>,
    observer: RequestCopyObserver,
}

impl ObservedRequestCopy {
    fn new(bytes: &[u8], observer: RequestCopyObserver) -> Self {
        Self {
            bytes: bytes.to_vec(),
            observer,
        }
    }
}

impl Drop for ObservedRequestCopy {
    fn drop(&mut self) {
        self.bytes.as_mut_slice().zeroize();
        self.observer
            .record_initialized_range_is_zero(self.bytes.as_slice());
    }
}

struct CopyingFakeTransport {
    fake: ScriptedTransport,
    request_copy_observer: RequestCopyObserver,
}

impl CopyingFakeTransport {
    fn new(fake: ScriptedTransport, request_copy_observer: RequestCopyObserver) -> Self {
        Self {
            fake,
            request_copy_observer,
        }
    }
}

impl Transport for CopyingFakeTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        let request_copy = ObservedRequestCopy::new(request, self.request_copy_observer.clone());
        self.fake
            .exchange(request_copy.bytes.as_slice(), max_response_bytes)
    }
}

fn assert_request_not_logged_or_retained(fake: &ScriptedTransport) {
    let sentinel =
        std::str::from_utf8(REQUEST_SENTINEL).expect("the request sentinel fixture is valid UTF-8");
    assert!(fake.retained_request_bytes().is_none());
    assert!(
        fake.captured_logs()
            .iter()
            .all(|message| !message.contains(sentinel))
    );
    assert!(!format!("{fake:?}").contains(sentinel));
}

#[test]
fn scripted_fake_accepts_response_exactly_at_limit() {
    let response_bytes = vec![0xA5; RESPONSE_LIMIT];
    let mut fake = ScriptedTransport::new(ExchangeScript::Success {
        response: response_bytes.clone(),
        request_write_chunks: vec![2, 3],
    });

    let response = fake
        .exchange(REQUEST_SENTINEL, RESPONSE_LIMIT)
        .expect("a response exactly at the cap is accepted");

    assert_eq!(response.as_bytes(), response_bytes);
    assert_eq!(fake.last_response_limit(), Some(RESPONSE_LIMIT));
    assert_eq!(fake.maximum_response_bytes_retained(), RESPONSE_LIMIT);
    assert_eq!(fake.write_call_count(), 3);
    assert_eq!(fake.written_byte_count(), REQUEST_SENTINEL.len());
}

#[test]
fn scripted_fake_rejects_one_byte_over_without_retaining_past_limit() {
    let mut fake = ScriptedTransport::new(ExchangeScript::Success {
        response: vec![0xA5; RESPONSE_LIMIT + 1],
        request_write_chunks: vec![REQUEST_SENTINEL.len()],
    });

    let error = fake
        .exchange(REQUEST_SENTINEL, RESPONSE_LIMIT)
        .expect_err("a response one byte over the cap is rejected");

    assert_eq!(fake.maximum_response_bytes_retained(), RESPONSE_LIMIT);
    assert_eq!(
        error.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
}

#[test]
fn request_sentinel_is_not_logged_or_retained_and_temporary_copy_is_zeroized_on_success() {
    let fake = ScriptedTransport::new(ExchangeScript::Success {
        response: SUCCESS_RESPONSE.to_vec(),
        request_write_chunks: vec![2, REQUEST_SENTINEL.len()],
    });
    let observer = RequestCopyObserver::new(REQUEST_SENTINEL.len());
    let mut transport = CopyingFakeTransport::new(fake, observer.clone());

    let response = transport
        .exchange(REQUEST_SENTINEL, SUCCESS_RESPONSE.len())
        .expect("the scripted exchange succeeds");

    assert_eq!(response.as_bytes(), SUCCESS_RESPONSE);
    assert!(
        observer.initialized_range_was_zero(),
        "observer must reject a zero-length slice after the initialized request copy was stored"
    );
    assert_eq!(transport.fake.exchange_count(), 1);
    assert_request_not_logged_or_retained(&transport.fake);
}

#[test]
fn request_sentinel_is_not_logged_or_retained_and_temporary_copy_is_zeroized_on_error() {
    let fake = ScriptedTransport::new(ExchangeScript::FailAfterPartialRead {
        written_bytes: 2,
        response_bytes: PARTIAL_RESPONSE.to_vec(),
    });
    let observer = RequestCopyObserver::new(REQUEST_SENTINEL.len());
    let mut transport = CopyingFakeTransport::new(fake, observer.clone());

    let error = transport
        .exchange(REQUEST_SENTINEL, RESPONSE_LIMIT)
        .expect_err("the scripted partial read fails");

    assert_eq!(
        error.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
    assert!(
        observer.initialized_range_was_zero(),
        "observer must reject a zero-length slice on the exchange error path"
    );
    assert_eq!(transport.fake.exchange_count(), 1);
    assert_request_not_logged_or_retained(&transport.fake);
    assert!(!format!("{error} {error:?}").contains("KMIPKIT_LOW_LEVEL_REQUEST_SENTINEL_73"));
}

#[test]
fn scripted_failure_before_write_reports_not_sent() {
    let mut fake = ScriptedTransport::new(ExchangeScript::FailBeforeWrite);

    let error = fake
        .exchange(REQUEST_SENTINEL, RESPONSE_LIMIT)
        .expect_err("the scripted pre-write failure is returned");

    assert_eq!(error.delivery_state(), RequestDeliveryState::NotSent);
    assert_eq!(fake.exchange_count(), 1);
    assert_eq!(fake.write_call_count(), 0);
}

#[test]
fn scripted_failure_after_partial_write_reports_possibly_sent() {
    let mut fake =
        ScriptedTransport::new(ExchangeScript::FailAfterPartialWrite { written_bytes: 2 });

    let error = fake
        .exchange(REQUEST_SENTINEL, RESPONSE_LIMIT)
        .expect_err("the scripted partial write fails");

    assert_eq!(error.delivery_state(), RequestDeliveryState::PossiblySent);
    assert_eq!(fake.exchange_count(), 1);
    assert_eq!(fake.write_call_count(), 1);
    assert_eq!(fake.written_byte_count(), 2);
}
