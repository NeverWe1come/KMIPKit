use std::io;
use std::{error::Error, fmt};

use crate::resolver::ResolveFailure;
use crate::worker::WorkerError;
use crate::{RequestDeliveryState, TransportCauseCategory};

use super::{error_chain_contains_tls_failure, io_error_cause, resolve_error, worker_error};

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
