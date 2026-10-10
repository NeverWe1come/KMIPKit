//! Test-only fixtures and helpers shared by `KMIPKit` crates.
#![forbid(unsafe_code)]

mod dns;
mod local_transport;
pub mod oasis_crypto_fixtures;
#[cfg(feature = "fixtures")]
mod pki;
#[cfg(feature = "scripted-transport")]
mod transport;

/// Stable byte fixtures for transport boundary tests.
pub mod fixtures {
    /// Unique request bytes used to detect accidental request logging or retention.
    pub const REQUEST_SENTINEL: &[u8] = b"KMIPKIT_LOW_LEVEL_REQUEST_SENTINEL_73";

    /// Short success response used by scripted transport tests.
    pub const SUCCESS_RESPONSE: &[u8] = b"KMIPKIT_RESPONSE_FIXTURE";

    /// Partial response bytes used by scripted error-path tests.
    pub const PARTIAL_RESPONSE: &[u8] = b"KMIPKIT_PARTIAL_RESPONSE_FIXTURE";
}

pub use dns::{DnsQueryType, LocalDnsFixture};
pub use local_transport::LoopbackTcpListener;
#[cfg(feature = "fixtures")]
pub use pki::{EphemeralIdentity, EphemeralPki, PkiFixtureError};
#[cfg(feature = "scripted-transport")]
pub use transport::{ExchangeScript, ResponseDropObserver, ScriptedTransport};
