//! Derived conversion-error tests for OASIS KMIP v2.1 §6.1.16, Tables 211–213,
//! and §9.16, Table 421's Discover Versions result rules. These are not
//! official OASIS test vectors.
//!
//! Traceability: `KMIPKIT-0007-FR-002`, `-FR-003`; `KMIPKIT-0007-SC-001`.

use std::error::Error;

use crate::{ResultStatus, ResultValidationError};

use super::{DiscoverVersionsError, operation_result};

#[test]
fn invalid_operation_result_error_retains_its_typed_validation_cause() {
    let invalid_results = [
        (
            ResultStatus::from_raw(1),
            None,
            ResultValidationError::FailureRequiresReason,
        ),
        (
            ResultStatus::from_raw(0),
            Some(crate::ResultReason::from_raw(1)),
            ResultValidationError::SuccessForbidsReason,
        ),
    ];

    for (status, reason, expected) in invalid_results {
        let error = operation_result(status, reason, None)
            .expect_err("invalid result combinations return a typed conversion error");
        assert_eq!(
            error,
            DiscoverVersionsError::InvalidOperationResult(expected)
        );
        let source = Error::source(&error)
            .expect("typed Discover Versions errors retain safe result-validation causes");
        assert_eq!(
            source.downcast_ref::<ResultValidationError>(),
            Some(&expected)
        );
    }
}

#[test]
fn discover_versions_errors_have_stable_safe_display_and_source_behavior() {
    let errors = [
        DiscoverVersionsError::UnexpectedOperation,
        DiscoverVersionsError::MissingResultStatus,
        DiscoverVersionsError::InvalidOperationResult(ResultValidationError::FailureRequiresReason),
        DiscoverVersionsError::MissingSuccessPayload,
        DiscoverVersionsError::MalformedProtocolVersion,
        DiscoverVersionsError::UnofferedProtocolVersion,
    ];

    for error in errors {
        assert_ne!(error.to_string(), "");
        if matches!(error, DiscoverVersionsError::InvalidOperationResult(_)) {
            assert!(Error::source(&error).is_some());
        } else {
            assert!(Error::source(&error).is_none());
        }
    }
}
