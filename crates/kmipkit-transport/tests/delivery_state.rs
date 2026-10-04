use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kmipkit_transport::{RequestDeliveryState, TransportCauseCategory, TransportError};

#[test]
fn request_is_not_sent_until_write_begins() {
    let state = RequestDeliveryState::not_sent();

    assert_eq!(state, RequestDeliveryState::NotSent);
    assert_eq!(state.write_started(), RequestDeliveryState::PossiblySent);
}

#[test]
fn zero_byte_response_read_remains_possibly_sent() {
    let state = RequestDeliveryState::PossiblySent;

    assert_eq!(
        state.response_bytes_received(0),
        RequestDeliveryState::PossiblySent
    );
}

#[test]
fn first_response_byte_advances_delivery_to_response_started() {
    let state = RequestDeliveryState::PossiblySent;

    assert_eq!(
        state.response_bytes_received(1),
        RequestDeliveryState::ResponseStarted
    );
}

#[test]
fn response_started_never_regresses_after_a_zero_byte_read() {
    let state = RequestDeliveryState::ResponseStarted;

    assert_eq!(
        state.response_bytes_received(0),
        RequestDeliveryState::ResponseStarted
    );
    assert_eq!(
        state.response_bytes_received(8),
        RequestDeliveryState::ResponseStarted
    );
}

#[test]
fn transport_error_drops_untrusted_source_and_redacts_its_chain() {
    let dropped = Arc::new(AtomicBool::new(false));
    let sentinel = "TLS_PRIVATE_KEY_SENTINEL";
    let error = TransportError::new(
        RequestDeliveryState::PossiblySent,
        TransportCauseCategory::Tls,
        DropProbe {
            dropped: Arc::clone(&dropped),
            text: sentinel,
        },
    );

    assert!(dropped.load(Ordering::Acquire));
    assert_eq!(error.delivery_state(), RequestDeliveryState::PossiblySent);
    assert_eq!(error.cause_category(), TransportCauseCategory::Tls);
    assert!(!format!("{error}").contains(sentinel));
    assert!(!format!("{error:?}").contains(sentinel));

    let mut source = error.source();
    while let Some(current) = source {
        assert!(!format!("{current}").contains(sentinel));
        assert!(!format!("{current:?}").contains(sentinel));
        source = current.source();
    }
}

struct DropProbe<'a> {
    dropped: Arc<AtomicBool>,
    text: &'a str,
}

impl fmt::Display for DropProbe<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.text)
    }
}

impl fmt::Debug for DropProbe<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.text)
    }
}

impl Error for DropProbe<'_> {}

impl Drop for DropProbe<'_> {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::Release);
    }
}
