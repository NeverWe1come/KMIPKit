//! Zeroizing owner for successful low-level transport response bytes.
//!
//! Derived project-policy tests cite KMIPKIT-0007-FR-005/-FR-009/-FR-013 and
//! SC-003, ADR-0014, and OASIS KMIP Specification v2.1 §9.12, Table 417 for
//! the response-size bound. They are not official OASIS conformance tests.

use std::fmt;

use zeroize::{Zeroize, Zeroizing};

/// An owned transport response whose initialized bytes are zeroized on drop.
///
/// The guarantee covers the initialized bytes in this wrapper's current
/// allocation. It does not cover spare capacity, allocations released earlier
/// during growth, caller-created copies, or copies owned by TLS, the operating
/// system, or third-party transport libraries.
pub struct TransportResponse {
    bytes: Zeroizing<Vec<u8>>,
    #[cfg(test)]
    drop_observer: Option<DropObserver>,
}

impl TransportResponse {
    /// Moves `bytes` into zeroizing-owned response storage.
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes: Zeroizing::new(bytes),
            #[cfg(test)]
            drop_observer: None,
        }
    }

    /// Borrows the initialized response bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    #[cfg(test)]
    fn with_drop_observer(bytes: Vec<u8>, observer: DropObserver) -> Self {
        Self {
            bytes: Zeroizing::new(bytes),
            drop_observer: Some(observer),
        }
    }
}

impl fmt::Debug for TransportResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TransportResponse([REDACTED])")
    }
}

impl Drop for TransportResponse {
    fn drop(&mut self) {
        (*self.bytes).zeroize();
        #[cfg(test)]
        if let Some(observer) = &self.drop_observer {
            observer.record_initialized_range_is_zero(self.bytes.as_slice());
        }
    }
}

#[cfg(test)]
#[derive(Clone)]
struct DropObserver {
    zeroized: std::sync::Arc<std::sync::atomic::AtomicBool>,
    expected_len: usize,
}

#[cfg(test)]
impl DropObserver {
    fn new(expected_len: usize) -> Self {
        Self {
            zeroized: std::sync::Arc::default(),
            expected_len,
        }
    }

    fn record_initialized_range_is_zero(&self, initialized_bytes: &[u8]) {
        let all_zero = initialized_bytes.len() == self.expected_len
            && initialized_bytes.iter().all(|byte| *byte == 0);
        self.zeroized
            .store(all_zero, std::sync::atomic::Ordering::SeqCst);
    }

    fn initialized_range_was_zero(&self) -> bool {
        self.zeroized.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fmt;

    use crate::{RequestDeliveryState, TransportCauseCategory, TransportError};

    use super::{DropObserver, TransportResponse};

    const RESPONSE_SENTINEL: &[u8] = b"KMIP_RESPONSE_BODY_SENTINEL";
    const PARTIAL_RESPONSE_SENTINEL: &str = "KMIP_PARTIAL_RESPONSE_BODY_SENTINEL";

    #[derive(Debug)]
    struct PartialReadSource(&'static str);

    impl fmt::Display for PartialReadSource {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.0)
        }
    }

    impl Error for PartialReadSource {}

    fn partial_read_failure(observer: DropObserver) -> Result<(), TransportError> {
        let _partial_response = TransportResponse::with_drop_observer(
            PARTIAL_RESPONSE_SENTINEL.as_bytes().to_vec(),
            observer,
        );

        Err(TransportError::new(
            RequestDeliveryState::ResponseStarted,
            TransportCauseCategory::Io,
            PartialReadSource(PARTIAL_RESPONSE_SENTINEL),
        ))
    }

    #[test]
    fn successful_response_drop_zeroizes_initialized_bytes_before_release() {
        let observer = DropObserver::new(RESPONSE_SENTINEL.len());
        let response =
            TransportResponse::with_drop_observer(RESPONSE_SENTINEL.to_vec(), observer.clone());

        assert_eq!(response.as_bytes(), RESPONSE_SENTINEL);
        drop(response);

        assert!(
            observer.initialized_range_was_zero(),
            "observer must reject a zero-length slice after initialized response bytes were stored"
        );
    }

    #[test]
    fn partial_read_error_drops_zeroized_response_and_redacts_source() {
        let observer = DropObserver::new(PARTIAL_RESPONSE_SENTINEL.len());
        let error = partial_read_failure(observer.clone())
            .expect_err("partial read returns a redacted transport error");
        let rendered = format!("{error} {error:?}");
        assert!(!rendered.contains(PARTIAL_RESPONSE_SENTINEL));
        let mut source = error.source();
        while let Some(current) = source {
            assert!(!current.to_string().contains(PARTIAL_RESPONSE_SENTINEL));
            source = current.source();
        }
        assert!(
            observer.initialized_range_was_zero(),
            "observer must reject a zero-length slice on the partial-read error path"
        );
    }

    #[test]
    fn response_debug_redacts_initialized_response_bytes() {
        let response = TransportResponse::new(RESPONSE_SENTINEL.to_vec());

        assert!(!format!("{response:?}").contains("KMIP_RESPONSE_BODY_SENTINEL"));
    }
}
