//! One-shot fake transport support for deterministic client execution tests.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use zeroize::Zeroizing;

/// A fake transport that captures its first request and transfers one response.
///
/// The fake does not split, rewrite, or retry request or response bytes. The
/// configured response is moved into [`TransportResponse`], which keeps the
/// transport crate's zeroizing ownership behavior.
pub(crate) struct OneShotFakeTransport {
    observation: FakeTransportObservation,
    response: Option<Vec<u8>>,
}

impl OneShotFakeTransport {
    /// Creates a fake transport and a handle for inspecting its exchange.
    pub(crate) fn new(response: Vec<u8>) -> (Self, FakeTransportObservation) {
        let observation = FakeTransportObservation::default();
        (
            Self {
                observation: observation.clone(),
                response: Some(response),
            },
            observation,
        )
    }
}

impl Transport for OneShotFakeTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        _max_response_bytes: usize,
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

        Ok(TransportResponse::new(response))
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
    use super::{FakeTransportObservation, OneShotFakeTransport};
    use kmipkit_transport::{RequestDeliveryState, Transport};

    #[test]
    fn captures_exact_request_and_moves_response_into_transport_wrapper() {
        let request = [0x42, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00];
        let response = [0x42, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00];
        let (mut transport, observation) = OneShotFakeTransport::new(response.to_vec());

        let actual_response = transport
            .exchange(&request, 1)
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
    fn observation_handle_can_be_cloned_for_shared_test_assertions() {
        let observation = FakeTransportObservation::default();
        let clone = observation.clone();

        assert_eq!(clone.exchange_count(), 0);
    }
}
