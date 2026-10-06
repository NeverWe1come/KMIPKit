//! Derived client-boundary tests for bounded and zeroizing response handling.
//! These are not official OASIS test vectors.
//!
//! OASIS KMIP Specification v2.1 §9.12 and Table 417; ADR-0014. These tests
//! observe only initialized bytes in the current KMIPKit-owned allocation
//! while it is still live. They make no claim about spare or uninitialized
//! capacity, prior allocations released during growth, caller copies, or
//! TLS/transport-library copies.
//!
//! Traceability: `KMIPKIT-0007-FR-005`, `-FR-009`, `-FR-013`, `-SC-003`,
//! `KMIPKIT-REQ-SPEC-9.12-001-002`, `-001-003`,
//! `KMIPKIT-ELEM-MESSAGE-FIELD-9-12-MAXIMUM-RESPONSE-SIZE`, and ADR-0014.

use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kmipkit_ttlv::codec::CodecLimits;
use zeroize::Zeroize;

const TEST_RESPONSE_CAP: usize = 8;
const RESPONSE_DEBUG_SENTINEL: &[u8] = b"KMIP_RESPONSE_DEBUG_SENTINEL";
const PARTIAL_RESPONSE_SENTINEL: &str = "KMIP_PARTIAL_RESPONSE_SENTINEL";
const PARTIAL_RESPONSE_PREFIX: &str = "KMIP_PARTIAL_";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateTransportFailure {
    ResponseTooLarge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateClientFailure {
    ResponseTooLarge,
}

/// Test-only observer retains only whether all initialized bytes were zero.
/// It never retains the slice, its length, pointer, capacity, or contents.
#[derive(Clone, Debug, Default)]
struct InitializedBytesZeroizationObserver(Arc<AtomicBool>);

impl InitializedBytesZeroizationObserver {
    fn record_initialized_range_is_zero(&self, initialized_bytes: &[u8]) {
        let all_zero = initialized_bytes.iter().all(|byte| *byte == 0);
        self.0.store(all_zero, Ordering::SeqCst);
    }

    fn initialized_range_was_zero(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// Private stand-in for the not-yet-implemented transport response wrapper.
/// The deliberately incomplete Drop and Debug behavior give T006 behavioral
/// Red assertions without introducing a production API.
#[derive(Debug)]
struct CandidateTransportResponse {
    initialized_bytes: Vec<u8>,
    drop_observer: Option<InitializedBytesZeroizationObserver>,
}

impl CandidateTransportResponse {
    fn new(initialized_bytes: Vec<u8>) -> Self {
        Self {
            initialized_bytes,
            drop_observer: None,
        }
    }

    fn with_drop_observer(
        initialized_bytes: Vec<u8>,
        drop_observer: InitializedBytesZeroizationObserver,
    ) -> Self {
        Self {
            initialized_bytes,
            drop_observer: Some(drop_observer),
        }
    }

    fn as_bytes(&self) -> &[u8] {
        self.initialized_bytes.as_slice()
    }
}

impl Drop for CandidateTransportResponse {
    fn drop(&mut self) {
        if let Some(observer) = &self.drop_observer {
            // This slice is exactly the initialized Vec range and is observed
            // before Rust drops the field. The Red candidate does not zeroize.
            observer.record_initialized_range_is_zero(self.initialized_bytes.as_slice());
        }
    }
}

#[derive(Default)]
struct CandidateCompliantFake {
    maximum_retained_bytes: usize,
}

impl CandidateCompliantFake {
    fn exchange(
        &mut self,
        response_stream: &[u8],
        max_response_bytes: usize,
    ) -> Result<CandidateTransportResponse, CandidateTransportFailure> {
        let mut initialized_bytes = Vec::new();
        initialized_bytes.extend(response_stream.iter().take(max_response_bytes).copied());
        self.maximum_retained_bytes = self.maximum_retained_bytes.max(initialized_bytes.len());

        if response_stream.len() > max_response_bytes {
            initialized_bytes.zeroize();
            return Err(CandidateTransportFailure::ResponseTooLarge);
        }

        Ok(CandidateTransportResponse::new(initialized_bytes))
    }
}

struct CandidateNonCompliantFake {
    oversized_response: Vec<u8>,
}

impl CandidateNonCompliantFake {
    fn exchange(self) -> CandidateTransportResponse {
        CandidateTransportResponse::new(self.oversized_response)
    }
}

/// Deliberately incomplete client-boundary candidate: it invokes the decoder
/// before checking the independently returned wrapper length.
fn candidate_decode_response<T>(
    response: &CandidateTransportResponse,
    limits: &CodecLimits,
    decoder: impl FnOnce(&[u8]) -> T,
) -> Result<T, CandidateClientFailure> {
    let decoded = decoder(response.as_bytes());
    if response.as_bytes().len() > limits.max_message_bytes() {
        return Err(CandidateClientFailure::ResponseTooLarge);
    }
    Ok(decoded)
}

#[derive(Debug)]
struct CandidatePartialReadSourceError {
    diagnostic: &'static str,
}

impl fmt::Display for CandidatePartialReadSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.diagnostic)
    }
}

impl Error for CandidatePartialReadSourceError {}

#[derive(Debug)]
struct CandidatePartialReadError {
    diagnostic: &'static str,
    source_error: CandidatePartialReadSourceError,
}

impl fmt::Display for CandidatePartialReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.diagnostic)
    }
}

impl Error for CandidatePartialReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source_error)
    }
}

/// Drops the current partial response allocation before returning a private
/// error candidate. Its diagnostic deliberately leaks a test sentinel.
fn candidate_partial_read_failure(
    initialized_bytes: Vec<u8>,
    observer: InitializedBytesZeroizationObserver,
) -> CandidatePartialReadError {
    let partial_response =
        CandidateTransportResponse::with_drop_observer(initialized_bytes, observer);
    drop(partial_response);

    CandidatePartialReadError {
        diagnostic: PARTIAL_RESPONSE_SENTINEL,
        source_error: CandidatePartialReadSourceError {
            diagnostic: PARTIAL_RESPONSE_PREFIX,
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateDiscoverVersionsHeaderField {
    BatchCount(usize),
    MaximumResponseSize(usize),
}

struct CandidateDiscoverVersionsRequest {
    header_fields: Vec<CandidateDiscoverVersionsHeaderField>,
}

/// Deliberately incomplete request candidate: Discover Versions is not a
/// likely-large operation, but this seam currently emits Maximum Response Size.
fn candidate_build_discover_versions_request() -> CandidateDiscoverVersionsRequest {
    CandidateDiscoverVersionsRequest {
        header_fields: vec![
            CandidateDiscoverVersionsHeaderField::BatchCount(1),
            CandidateDiscoverVersionsHeaderField::MaximumResponseSize(4_096),
        ],
    }
}

fn limits_with_response_cap(cap: usize) -> CodecLimits {
    CodecLimits::new(
        cap,
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
        CodecLimits::DEFAULT_MAX_ELEMENTS,
    )
    .expect("the fixture uses the supported default depth and element limits")
}

#[test]
fn compliant_fake_accepts_a_response_exactly_at_the_configured_cap() {
    let limits = limits_with_response_cap(TEST_RESPONSE_CAP);
    let response_stream = vec![0xA5; limits.max_message_bytes()];
    let mut fake = CandidateCompliantFake::default();

    let response = fake
        .exchange(&response_stream, limits.max_message_bytes())
        .expect("a response exactly at the configured cap is accepted");

    assert_eq!(response.as_bytes().len(), limits.max_message_bytes());
    assert_eq!(fake.maximum_retained_bytes, limits.max_message_bytes());
}

#[test]
fn compliant_fake_rejects_one_byte_over_without_retaining_past_the_cap() {
    let limits = limits_with_response_cap(TEST_RESPONSE_CAP);
    let response_stream = vec![0xA5; limits.max_message_bytes() + 1];
    let mut fake = CandidateCompliantFake::default();

    let result = fake.exchange(&response_stream, limits.max_message_bytes());

    assert!(matches!(
        result,
        Err(CandidateTransportFailure::ResponseTooLarge)
    ));
    assert_eq!(fake.maximum_retained_bytes, limits.max_message_bytes());
}

#[test]
fn client_rejects_oversized_non_compliant_wrapper_before_decoder_entry() {
    let limits = limits_with_response_cap(TEST_RESPONSE_CAP);
    let fake = CandidateNonCompliantFake {
        oversized_response: vec![0xA5; limits.max_message_bytes() + 1],
    };
    let response = fake.exchange();
    let decoder_calls = Cell::new(0);

    let result = candidate_decode_response(&response, &limits, |_bytes| {
        decoder_calls.set(decoder_calls.get() + 1);
    });

    assert_eq!(result, Err(CandidateClientFailure::ResponseTooLarge));
    assert_eq!(decoder_calls.get(), 0);
}

#[test]
fn successful_response_wrapper_zeroizes_initialized_bytes_before_drop() {
    let observer = InitializedBytesZeroizationObserver::default();
    let response = CandidateTransportResponse::with_drop_observer(
        RESPONSE_DEBUG_SENTINEL.to_vec(),
        observer.clone(),
    );

    drop(response);

    assert!(observer.initialized_range_was_zero());
}

#[test]
fn partial_read_error_zeroizes_initialized_bytes_before_allocation_release() {
    let observer = InitializedBytesZeroizationObserver::default();
    let _error = candidate_partial_read_failure(
        PARTIAL_RESPONSE_SENTINEL.as_bytes().to_vec(),
        observer.clone(),
    );

    assert!(observer.initialized_range_was_zero());
}

#[test]
fn response_wrapper_debug_redacts_initialized_response_bytes() {
    let response = CandidateTransportResponse::new(RESPONSE_DEBUG_SENTINEL.to_vec());
    let rendered = format!("{response:?}");
    let raw_bytes_debug = format!("{:?}", response.as_bytes());

    assert!(!rendered.contains(&raw_bytes_debug));
    assert!(
        !rendered.contains(
            std::str::from_utf8(RESPONSE_DEBUG_SENTINEL)
                .expect("the diagnostic sentinel is valid UTF-8")
        )
    );
}

#[test]
fn response_wrapper_debug_redacts_truncated_initialized_byte_prefix() {
    let response = CandidateTransportResponse::new(RESPONSE_DEBUG_SENTINEL.to_vec());
    let rendered = format!("{response:?}");
    let prefix = &RESPONSE_DEBUG_SENTINEL[.."KMIP_RESP".len()];
    let prefix_text = std::str::from_utf8(prefix).expect("the diagnostic prefix is valid UTF-8");
    let prefix_byte_values = prefix
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let prefix_bytes_debug = format!("[{prefix_byte_values}");

    assert!(!rendered.contains(prefix_text));
    assert!(!rendered.contains(&prefix_bytes_debug));
}

#[test]
fn partial_read_error_debug_and_display_redact_body_and_prefix() {
    let observer = InitializedBytesZeroizationObserver::default();
    let error =
        candidate_partial_read_failure(PARTIAL_RESPONSE_SENTINEL.as_bytes().to_vec(), observer);
    let rendered_debug = format!("{error:?}");
    let rendered_display = error.to_string();

    let debug_leaks_body = rendered_debug.contains(PARTIAL_RESPONSE_SENTINEL);
    let debug_leaks_prefix = rendered_debug.contains(PARTIAL_RESPONSE_PREFIX);
    let display_leaks_body = rendered_display.contains(PARTIAL_RESPONSE_SENTINEL);
    let display_leaks_prefix = rendered_display.contains(PARTIAL_RESPONSE_PREFIX);

    assert!(
        !debug_leaks_body && !debug_leaks_prefix && !display_leaks_body && !display_leaks_prefix,
        "leak checks: error Debug body={debug_leaks_body}, prefix={debug_leaks_prefix}; error Display body={display_leaks_body}, prefix={display_leaks_prefix}"
    );
}

#[test]
fn partial_read_error_source_chain_redacts_body_and_prefix() {
    let observer = InitializedBytesZeroizationObserver::default();
    let error =
        candidate_partial_read_failure(PARTIAL_RESPONSE_SENTINEL.as_bytes().to_vec(), observer);
    let mut source = error.source();
    let mut source_count = 0;
    let mut source_chain_leaks_body = false;
    let mut source_chain_leaks_prefix = false;

    while let Some(source_error) = source {
        let rendered_debug = format!("{source_error:?}");
        let rendered_display = source_error.to_string();

        source_chain_leaks_body |= rendered_debug.contains(PARTIAL_RESPONSE_SENTINEL)
            || rendered_display.contains(PARTIAL_RESPONSE_SENTINEL);
        source_chain_leaks_prefix |= rendered_debug.contains(PARTIAL_RESPONSE_PREFIX)
            || rendered_display.contains(PARTIAL_RESPONSE_PREFIX);

        source_count += 1;
        source = source_error.source();
    }

    assert!(source_count > 0);
    assert!(
        !source_chain_leaks_body && !source_chain_leaks_prefix,
        "leak checks: exposed error source chain body={source_chain_leaks_body}, prefix={source_chain_leaks_prefix}"
    );
}

#[test]
fn discover_versions_request_omits_peer_visible_maximum_response_size() {
    let request = candidate_build_discover_versions_request();

    assert!(!request.header_fields.iter().any(|field| matches!(
        field,
        CandidateDiscoverVersionsHeaderField::MaximumResponseSize(_)
    )));
    assert!(
        request
            .header_fields
            .iter()
            .any(|field| matches!(field, CandidateDiscoverVersionsHeaderField::BatchCount(1)))
    );
}
