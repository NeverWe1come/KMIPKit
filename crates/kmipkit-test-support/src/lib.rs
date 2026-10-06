//! Test-only fixtures and helpers shared by `KMIPKit` crates.
#![forbid(unsafe_code)]

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

pub use transport::{ExchangeScript, ResponseDropObserver, ScriptedTransport};
