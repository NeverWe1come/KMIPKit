//! Derived client request Time Stamp tests; these are not official OASIS vectors.
//!
//! OASIS KMIP Specification v2.1 §9.20, Table 425: a client request may
//! include the optional Time Stamp as a Date-Time. These tests use a caller
//! supplied raw Date-Time and do not test countdown-derived values.
//!
//! Traceability: `KMIPKIT-0007-FR-015`, `KMIPKIT-0007-SC-007`,
//! `KMIPKIT-REQ-SPEC-9.20-001-001`, and
//! `KMIPKIT-ELEM-MESSAGE-FIELD-9-20-TIME-STAMP`.

use std::error::Error;
use std::fmt;

const REQUEST_TIME_STAMP_SENTINEL: i64 = 1_234_567_890;
const REQUEST_TIME_STAMP_PREFIX: &str = "123456";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CandidateOutgoingRequestHeader {
    time_stamp: Option<i64>,
}

impl fmt::Display for CandidateOutgoingRequestHeader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "RequestHeader(time_stamp={:?})", self.time_stamp)
    }
}

/// Deliberately incomplete candidate: the future client currently omits a
/// caller-supplied Time Stamp instead of preserving its Date-Time unchanged.
fn candidate_build_request_header(
    _caller_time_stamp: Option<i64>,
) -> CandidateOutgoingRequestHeader {
    CandidateOutgoingRequestHeader { time_stamp: None }
}

#[derive(Debug)]
struct CandidateRequestErrorSource {
    time_stamp: i64,
}

impl fmt::Display for CandidateRequestErrorSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "request failed at {}", self.time_stamp)
    }
}

impl Error for CandidateRequestErrorSource {}

#[derive(Debug)]
struct CandidateRequestError {
    time_stamp: i64,
    source_error: CandidateRequestErrorSource,
}

impl fmt::Display for CandidateRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "request failed at {}", self.time_stamp)
    }
}

impl Error for CandidateRequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source_error)
    }
}

fn timestamp_leak_flags(rendered: &str) -> (bool, bool) {
    (
        rendered.contains(REQUEST_TIME_STAMP_SENTINEL.to_string().as_str()),
        rendered.contains(REQUEST_TIME_STAMP_PREFIX),
    )
}

#[test]
fn outgoing_request_preserves_the_caller_time_stamp_date_time_exactly() {
    let request = candidate_build_request_header(Some(REQUEST_TIME_STAMP_SENTINEL));

    assert_eq!(request.time_stamp, Some(REQUEST_TIME_STAMP_SENTINEL));
}

#[test]
fn outgoing_request_omits_time_stamp_when_the_caller_did_not_supply_one() {
    let request = candidate_build_request_header(None);

    assert_eq!(request.time_stamp, None);
}

#[test]
fn outgoing_request_debug_and_display_redact_the_caller_time_stamp() {
    let request = CandidateOutgoingRequestHeader {
        time_stamp: Some(REQUEST_TIME_STAMP_SENTINEL),
    };
    let rendered_debug = format!("{request:?}");
    let rendered_display = request.to_string();
    let (debug_leaks_value, debug_leaks_prefix) = timestamp_leak_flags(&rendered_debug);
    let (display_leaks_value, display_leaks_prefix) = timestamp_leak_flags(&rendered_display);

    assert!(
        !debug_leaks_value && !debug_leaks_prefix && !display_leaks_value && !display_leaks_prefix,
        "request leak checks: Debug value={debug_leaks_value}, prefix={debug_leaks_prefix}; Display value={display_leaks_value}, prefix={display_leaks_prefix}"
    );
}

#[test]
fn request_error_debug_display_and_every_source_redact_the_time_stamp() {
    let error = CandidateRequestError {
        time_stamp: REQUEST_TIME_STAMP_SENTINEL,
        source_error: CandidateRequestErrorSource {
            time_stamp: REQUEST_TIME_STAMP_SENTINEL,
        },
    };
    let rendered_debug = format!("{error:?}");
    let rendered_display = error.to_string();
    let (debug_leaks_value, debug_leaks_prefix) = timestamp_leak_flags(&rendered_debug);
    let (display_leaks_value, display_leaks_prefix) = timestamp_leak_flags(&rendered_display);
    let mut source = error.source();
    let mut source_count = 0;
    let mut source_chain_leaks_value = false;
    let mut source_chain_leaks_prefix = false;

    while let Some(source_error) = source {
        let source_debug = format!("{source_error:?}");
        let source_display = source_error.to_string();
        let (debug_value, debug_prefix) = timestamp_leak_flags(&source_debug);
        let (display_value, display_prefix) = timestamp_leak_flags(&source_display);

        source_chain_leaks_value |= debug_value || display_value;
        source_chain_leaks_prefix |= debug_prefix || display_prefix;
        source_count += 1;
        source = source_error.source();
    }

    assert!(source_count > 0);
    assert!(
        !debug_leaks_value
            && !debug_leaks_prefix
            && !display_leaks_value
            && !display_leaks_prefix
            && !source_chain_leaks_value
            && !source_chain_leaks_prefix,
        "error leak checks: Debug value={debug_leaks_value}, prefix={debug_leaks_prefix}; Display value={display_leaks_value}, prefix={display_leaks_prefix}; sources value={source_chain_leaks_value}, prefix={source_chain_leaks_prefix}"
    );
}

// T008 owns the static production-logger AST audit. These test-only source
// fixtures and its deliberately incomplete seam let T008 replace this test
// without introducing a second scanner or early CI enforcement.
mod logger_audit_fixture {
    pub(super) const FORMATTED_TIME_STAMP_CALLSITE: &str = r#"
tracing::debug!("sending request with timestamp {}", request.header().time_stamp());
"#;

    pub(super) const TIME_STAMP_LOG_FIELD_CALLSITE: &str = r#"
tracing::debug!(request_time_stamp = ?request.header().time_stamp(), "sending request");
"#;

    pub(super) const SAFE_CALLSITE: &str = r#"
tracing::debug!(batch_count = request.header().batch_count(), "sending request");
"#;
}

/// Incomplete T008 candidate seam. Static analysis is intentionally absent.
fn candidate_logger_audit_detects_time_stamp(_callsite: &str) -> bool {
    false
}

#[test]
fn static_logger_audit_fixture_flags_formatted_and_field_time_stamps_only() {
    let formatted_callsite_flagged = candidate_logger_audit_detects_time_stamp(
        logger_audit_fixture::FORMATTED_TIME_STAMP_CALLSITE,
    );
    let field_callsite_flagged = candidate_logger_audit_detects_time_stamp(
        logger_audit_fixture::TIME_STAMP_LOG_FIELD_CALLSITE,
    );
    let safe_callsite_flagged =
        candidate_logger_audit_detects_time_stamp(logger_audit_fixture::SAFE_CALLSITE);

    assert!(
        formatted_callsite_flagged && field_callsite_flagged && !safe_callsite_flagged,
        "logger audit fixture results: formatted={formatted_callsite_flagged}, field={field_callsite_flagged}, safe={safe_callsite_flagged}"
    );
}
