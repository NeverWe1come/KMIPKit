//! Raw TLS and HTTPS transport implementations for `KMIPKit`.
#![forbid(unsafe_code)]

mod error;
mod response;

pub use error::{RequestDeliveryState, TransportCauseCategory, TransportError};
pub use response::TransportResponse;

/// A synchronous, bounded exchange of caller-supplied request bytes.
///
/// Implementations borrow the request only for the duration of `exchange`.
/// They must not log, format, or retain its contents after the call returns.
/// Response reads must stop at `max_response_bytes`; implementations must
/// clean up any KMIPKit-owned temporary request or response allocations before
/// releasing them. Failures preserve the strongest available request-delivery
/// evidence and must not expose request or response contents.
pub trait Transport {
    /// Exchanges one request and returns its bounded response.
    ///
    /// `request` remains owned by the caller. This method performs no KMIP
    /// encoding or validation. Implementations must not retry the exchange
    /// automatically, retain the request beyond this synchronous call, or
    /// return a response larger than `max_response_bytes`.
    ///
    /// # Errors
    ///
    /// Returns a redacted error that preserves the strongest available
    /// [`RequestDeliveryState`]. Any partial response allocation owned by
    /// `KMIPKit` must be zeroized before this method returns an error.
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError>;
}
