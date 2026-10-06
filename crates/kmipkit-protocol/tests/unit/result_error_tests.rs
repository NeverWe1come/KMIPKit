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
