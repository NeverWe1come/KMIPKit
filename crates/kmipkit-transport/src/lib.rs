//! Public low-level synchronous byte-exchange contract for `KMIPKit`.
//!
//! This crate currently provides the [`Transport`] contract, delivery-aware
//! errors, and the zeroizing [`TransportResponse`] wrapper. It does not yet
//! provide a production TLS or HTTPS adapter. Direct callers supply and own
//! request bytes; this API does not encode or validate KMIP messages and does
//! not provide the typed client's request-owner guarantee. The top-level
//! `kmipkit` facade does not re-export this crate, and callers cannot inject a
//! custom implementation into `kmipkit-client::Client`.
//!
//! See the repository decision at
//! `docs/adr/0014-public-transport-exchange-contract.md` for the exact
//! ownership, response-limit, and cleanup contract.
#![forbid(unsafe_code)]

mod config;
mod error;
mod response;
// T019 exercises this private builder before T023 connects production adapters.
#[allow(dead_code)]
mod tls;
// T010 introduces the private timeout seam before later adapter tasks consume it.
#[allow(dead_code)]
mod timeout;
#[cfg(test)]
mod tls_policy;
#[cfg(test)]
#[path = "tls_policy_tests.rs"]
mod tls_policy_tests;
// The resolver precedes the production adapters that consume it in later tasks.
#[allow(dead_code)]
mod resolver;
// The worker precedes the production adapters that consume it in later tasks.
#[allow(dead_code)]
mod worker;

pub use config::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, RequestOptions,
    RevocationListInput, TimeoutLimit, TimeoutPolicy, TransportConfig, TransportConfigBuilder,
    TransportConfigError, TrustSource,
};
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
