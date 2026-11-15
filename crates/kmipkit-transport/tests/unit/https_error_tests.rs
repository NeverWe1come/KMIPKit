use std::io;
use std::time::{Duration, Instant};
use std::{error::Error, fmt};

use crate::resolver::ResolveFailure;
use crate::worker::WorkerError;
use crate::{RequestDeliveryState, TransportCauseCategory};

use super::{
    canceled_error, deadline_for, earlier_deadline, error_chain_contains_tls_failure,
    io_error_cause, resolve_error, timeout_duration, timeout_error, wait_for_cancel, wait_until,
    worker_error,
};

#[test]
fn resolver_failures_map_to_safe_categories_and_preserve_delivery_state() {
    let cases = [
        (ResolveFailure::Deadline, TransportCauseCategory::Timeout),
        (ResolveFailure::Cancelled, TransportCauseCategory::Timeout),
        (ResolveFailure::Capacity, TransportCauseCategory::Other),
        (ResolveFailure::Lookup, TransportCauseCategory::Io),
    ];

    for (failure, cause) in cases {
        let error = resolve_error(failure, RequestDeliveryState::ResponseStarted);
        assert_eq!(error.cause_category(), cause);
        assert_eq!(
            error.delivery_state(),
            RequestDeliveryState::ResponseStarted
        );
    }
}

#[test]
fn worker_lifecycle_and_operation_errors_keep_their_safe_classification() {
    let cases = [
        (
            WorkerError::Deadline(RequestDeliveryState::NotSent),
            TransportCauseCategory::Timeout,
        ),
        (
            WorkerError::Closed(RequestDeliveryState::PossiblySent),
            TransportCauseCategory::Other,
        ),
        (
            WorkerError::Stopped(RequestDeliveryState::ResponseStarted),
            TransportCauseCategory::Other,
        ),
    ];

    for (failure, cause) in cases {
        let error = worker_error(failure);
        assert_eq!(error.cause_category(), cause);
        assert_eq!(error.delivery_state(), failure.delivery_state());
    }

    let source = super::safe_error(
        RequestDeliveryState::ResponseStarted,
        TransportCauseCategory::Tls,
        io::Error::other("private TLS source"),
    );
    let error = worker_error(WorkerError::Operation {
        delivery_state: RequestDeliveryState::PossiblySent,
        source,
    });
    assert_eq!(error.cause_category(), TransportCauseCategory::Tls);
    assert_eq!(error.delivery_state(), RequestDeliveryState::PossiblySent);
}

#[test]
fn tls_failure_chain_and_io_kinds_are_classified_without_exposing_sources() {
    let tls_error = rustls::Error::General("private TLS detail".to_owned());
    assert!(error_chain_contains_tls_failure(Some(&tls_error)));

    let wrapped_tls_error =
        ErrorWithSource(rustls::Error::General("private TLS detail".to_owned()));
    assert!(error_chain_contains_tls_failure(Some(&wrapped_tls_error)));
    assert!(!error_chain_contains_tls_failure(None));

    assert_eq!(
        io_error_cause(io::ErrorKind::TimedOut),
        TransportCauseCategory::Timeout
    );
    assert_eq!(
        io_error_cause(io::ErrorKind::InvalidInput),
        TransportCauseCategory::Other
    );
    assert_eq!(
        io_error_cause(io::ErrorKind::ConnectionReset),
        TransportCauseCategory::Io
    );
}

#[test]
fn deadline_helpers_preserve_unbounded_zero_and_earliest_deadline_semantics() {
    let start = Instant::now();
    assert_eq!(
        deadline_for(crate::TimeoutLimit::Unbounded, start)
            .expect("unbounded phase deadlines are representable"),
        None
    );
    assert_eq!(
        deadline_for(crate::TimeoutLimit::Bounded(Duration::ZERO), start)
            .expect("a zero duration deadline is representable"),
        Some(start)
    );
    assert_eq!(timeout_duration(crate::TimeoutLimit::Unbounded), None);
    assert_eq!(
        timeout_duration(crate::TimeoutLimit::Bounded(Duration::from_millis(7))),
        Some(Duration::from_millis(7))
    );

    let earlier = start + Duration::from_secs(1);
    let later = start + Duration::from_secs(2);
    assert_eq!(earlier_deadline(Some(earlier), Some(later)), Some(earlier));
    assert_eq!(earlier_deadline(Some(earlier), None), Some(earlier));
    assert_eq!(earlier_deadline(None, Some(later)), Some(later));
    assert_eq!(earlier_deadline(None, None), None);
}

#[test]
fn deadline_overflow_is_reported_without_panicking() {
    let error = deadline_for(crate::TimeoutLimit::Bounded(Duration::MAX), Instant::now())
        .expect_err("an unrepresentable deadline is rejected");

    assert_eq!(error.kind(), io::ErrorKind::Other);
}

#[tokio::test]
async fn unbounded_https_wait_remains_pending() {
    let result = tokio::time::timeout(Duration::from_millis(1), wait_until(None)).await;

    assert!(result.is_err(), "an unbounded deadline must not fire");
}

#[tokio::test]
async fn https_cancel_wait_ends_when_its_sender_is_dropped() {
    let (sender, mut receiver) = tokio::sync::watch::channel(false);
    drop(sender);

    wait_for_cancel(&mut receiver).await;
}

#[test]
fn explicit_timeout_and_cancellation_errors_retain_the_current_delivery_state() {
    for state in [
        RequestDeliveryState::NotSent,
        RequestDeliveryState::PossiblySent,
        RequestDeliveryState::ResponseStarted,
    ] {
        let timeout = timeout_error(state);
        assert_eq!(timeout.cause_category(), TransportCauseCategory::Timeout);
        assert_eq!(timeout.delivery_state(), state);

        let canceled = canceled_error(state);
        assert_eq!(canceled.cause_category(), TransportCauseCategory::Timeout);
        assert_eq!(canceled.delivery_state(), state);
    }
}

#[derive(Debug)]
struct ErrorWithSource(rustls::Error);

impl fmt::Display for ErrorWithSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("wrapped TLS test error")
    }
}

impl Error for ErrorWithSource {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}
