//! One-shot fake transport support for deterministic client execution tests.

use std::cell::Cell;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use zeroize::{Zeroize, Zeroizing};

/// A fake transport that captures its first request and transfers one response.
///
/// The fake does not split, rewrite, or retry request or response bytes. The
/// configured response is moved into [`TransportResponse`], which keeps the
/// transport crate's zeroizing ownership behavior.
pub(crate) struct OneShotFakeTransport {
    observation: FakeTransportObservation,
    response: Option<PendingResponse>,
}

impl OneShotFakeTransport {
    /// Creates a fake transport and a handle for inspecting its exchange.
    pub(crate) fn new(response: Vec<u8>) -> (Self, FakeTransportObservation) {
        let observation = FakeTransportObservation::default();
        (
            Self {
                observation: observation.clone(),
                response: Some(PendingResponse::new(response, None)),
            },
            observation,
        )
    }

    fn with_response_drop_observer(response: Vec<u8>, drop_observer: ResponseDropObserver) -> Self {
        Self {
            observation: FakeTransportObservation::default(),
            response: Some(PendingResponse::new(response, Some(drop_observer))),
        }
    }
}

impl Transport for OneShotFakeTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        {
            let mut observation = self.observation.0.borrow_mut();
            observation.exchange_count = observation.exchange_count.saturating_add(1);
            if observation.captured_request.is_none() {
                observation.captured_request = Some(Zeroizing::new(request.to_vec()));
            }
        }

        let Some(response) = self.response.take() else {
            return Err(TransportError::new(
                RequestDeliveryState::not_sent(),
                TransportCauseCategory::Other,
                std::io::Error::other("the one-shot fake response was already consumed"),
            ));
        };

        if response.bytes.len() > max_response_bytes {
            let received_bytes = response.bytes.len();
            return Err(TransportError::new(
                RequestDeliveryState::not_sent()
                    .write_started()
                    .response_bytes_received(received_bytes),
                TransportCauseCategory::Other,
                std::io::Error::other("the configured fake response exceeds the response limit"),
            ));
        }

        Ok(TransportResponse::new(response.into_bytes()))
    }
}

struct PendingResponse {
    bytes: Zeroizing<Vec<u8>>,
    drop_observer: Option<ResponseDropObserver>,
}

impl PendingResponse {
    fn new(bytes: Vec<u8>, drop_observer: Option<ResponseDropObserver>) -> Self {
        Self {
            bytes: Zeroizing::new(bytes),
            drop_observer,
        }
    }

    fn into_bytes(mut self) -> Vec<u8> {
        self.drop_observer.take();
        std::mem::take(&mut *self.bytes)
    }
}

impl Drop for PendingResponse {
    fn drop(&mut self) {
        self.bytes.as_mut_slice().zeroize();
        if let Some(observer) = &self.drop_observer {
            observer.record(self.bytes.as_slice());
        }
    }
}

#[derive(Clone)]
struct ResponseDropObserver {
    observed_zeroized: Rc<Cell<Option<bool>>>,
    expected_len: usize,
}

impl ResponseDropObserver {
    fn new(expected_len: usize) -> Self {
        Self {
            observed_zeroized: Rc::new(Cell::new(None)),
            expected_len,
        }
    }

    fn result(&self) -> Option<bool> {
        self.observed_zeroized.get()
    }

    fn record(&self, bytes: &[u8]) {
        let zeroized = bytes.len() == self.expected_len && bytes.iter().all(|byte| *byte == 0);
        self.observed_zeroized.set(Some(zeroized));
    }
}

/// Read-only observation of the fake transport's exchange count and request.
#[derive(Clone, Default)]
pub(crate) struct FakeTransportObservation(Rc<RefCell<ObservationState>>);

#[derive(Default)]
struct ObservationState {
    exchange_count: usize,
    captured_request: Option<Zeroizing<Vec<u8>>>,
}

impl FakeTransportObservation {
    /// Returns the number of times the transport's exchange method was called.
    pub(crate) fn exchange_count(&self) -> usize {
        self.0.borrow().exchange_count
    }

    /// Borrows the captured request for inspection without retaining a copy.
    pub(crate) fn with_captured_request<R>(&self, inspect: impl FnOnce(Option<&[u8]>) -> R) -> R {
        let observation = self.0.borrow();
        let request = observation
            .captured_request
            .as_ref()
            .map(|bytes| bytes.as_slice());
        inspect(request)
    }
}

impl fmt::Debug for FakeTransportObservation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("FakeTransportObservation([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::{FakeTransportObservation, OneShotFakeTransport, ResponseDropObserver};
    use kmipkit_transport::{RequestDeliveryState, Transport};

    #[test]
    fn captures_exact_request_and_moves_response_into_transport_wrapper() {
        let request = [0x42, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00];
        let response = [0x42, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00];
        let (mut transport, observation) = OneShotFakeTransport::new(response.to_vec());

        let actual_response = transport
            .exchange(&request, response.len())
            .expect("the configured response is returned");

        assert_eq!(actual_response.as_bytes(), response);
        assert_eq!(observation.exchange_count(), 1);
        observation.with_captured_request(|captured| {
            assert_eq!(captured, Some(request.as_slice()));
        });
        assert_eq!(
            format!("{observation:?}"),
            "FakeTransportObservation([REDACTED])"
        );
    }

    #[test]
    fn subsequent_exchange_is_counted_and_does_not_receive_another_response() {
        let first_request = [1, 2, 3];
        let second_request = [4, 5, 6];
        let (mut transport, observation) = OneShotFakeTransport::new(vec![7, 8, 9]);

        let _ = transport
            .exchange(&first_request, usize::MAX)
            .expect("the first exchange consumes the configured response");
        let error = transport
            .exchange(&second_request, usize::MAX)
            .expect_err("a second exchange has no configured response");

        assert_eq!(observation.exchange_count(), 2);
        assert_eq!(error.delivery_state(), RequestDeliveryState::not_sent());
        observation.with_captured_request(|captured| {
            assert_eq!(captured, Some(first_request.as_slice()));
        });
    }

    #[test]
    fn dropping_transport_zeroizes_an_unconsumed_configured_response() {
        let response = vec![0xA5; 8];
        let observer = ResponseDropObserver::new(response.len());
        let transport =
            OneShotFakeTransport::with_response_drop_observer(response, observer.clone());

        drop(transport);

        assert_eq!(observer.result(), Some(true));
    }

    #[test]
    fn oversized_configured_response_is_rejected_after_response_bytes_arrive() {
        let response = b"OVER_CAP_RESPONSE_SENTINEL";
        let (mut transport, observation) = OneShotFakeTransport::new(response.to_vec());

        let error = transport
            .exchange(&[0x42], response.len() - 1)
            .expect_err("the fake must enforce the configured response cap");

        assert_eq!(
            error.delivery_state(),
            RequestDeliveryState::ResponseStarted
        );
        assert_eq!(observation.exchange_count(), 1);
        assert!(!error.to_string().contains("OVER_CAP_RESPONSE_SENTINEL"));
        assert!(!format!("{error:?}").contains("OVER_CAP_RESPONSE_SENTINEL"));
    }

    #[test]
    fn observation_handle_can_be_cloned_for_shared_test_assertions() {
        let observation = FakeTransportObservation::default();
        let clone = observation.clone();

        assert_eq!(clone.exchange_count(), 0);
    }
}
