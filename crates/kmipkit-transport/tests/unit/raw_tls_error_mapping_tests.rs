use std::io;

use crate::resolver::ResolveFailure;
use crate::worker::WorkerError;
use crate::{RequestDeliveryState, TransportCauseCategory};

use super::{
    canceled_error, deadline_for, earlier_deadline, io_error, resolve_error, timeout_duration,
    timeout_error, wait_for_cancel, wait_until, worker_error,
};

#[test]
fn resolver_failures_keep_their_safe_cause_and_delivery_state() {
    let cases = [
        (ResolveFailure::Deadline, TransportCauseCategory::Timeout),
        (ResolveFailure::Cancelled, TransportCauseCategory::Timeout),
        (ResolveFailure::Capacity, TransportCauseCategory::Other),
        (ResolveFailure::Lookup, TransportCauseCategory::Io),
    ];

    for (failure, cause) in cases {
        let error = resolve_error(failure, RequestDeliveryState::PossiblySent);
        assert_eq!(error.cause_category(), cause);
        assert_eq!(error.delivery_state(), RequestDeliveryState::PossiblySent);
    }
}

#[test]
fn worker_lifecycle_and_operation_failures_keep_classification() {
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
fn io_failure_categories_distinguish_tls_timeout_input_and_socket_errors() {
    let cases = [
        (io::ErrorKind::TimedOut, TransportCauseCategory::Timeout),
        (io::ErrorKind::InvalidInput, TransportCauseCategory::Other),
        (io::ErrorKind::ConnectionReset, TransportCauseCategory::Io),
    ];
    for (kind, cause) in cases {
        let error = io_error(
            io::Error::new(kind, "private IO detail"),
            RequestDeliveryState::NotSent,
        );
        assert_eq!(error.cause_category(), cause);
        assert_eq!(error.delivery_state(), RequestDeliveryState::NotSent);
    }
}

#[test]
fn explicit_deadline_and_cancellation_errors_preserve_delivery_evidence() {
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

#[tokio::test]
async fn unbounded_raw_tls_wait_remains_pending() {
    let result = tokio::time::timeout(std::time::Duration::from_millis(1), wait_until(None)).await;

    assert!(result.is_err(), "an unbounded deadline must not fire");
}

#[tokio::test]
async fn raw_tls_cancel_wait_ends_when_its_sender_is_dropped() {
    let (sender, mut receiver) = tokio::sync::watch::channel(false);
    drop(sender);

    wait_for_cancel(&mut receiver).await;
}

#[test]
fn raw_tls_unbounded_and_combined_deadlines_keep_their_contract() {
    let now = std::time::Instant::now();
    let later = now + std::time::Duration::from_secs(1);

    assert_eq!(timeout_duration(crate::TimeoutLimit::Unbounded), None);
    assert_eq!(
        deadline_for(crate::TimeoutLimit::Unbounded, now)
            .expect("unbounded deadlines do not overflow"),
        None
    );
    assert_eq!(earlier_deadline(None, Some(later)), Some(later));
    assert_eq!(earlier_deadline(None, None), None);
}
