use std::error::Error;
use std::fmt::Display;

use crate::{
    CertifyError, CheckError, ObtainLeaseError, ReProvisionError, RecertifyError, RegisterError,
    RekeyError, RekeyKeyPairError, ResultValidationError, RevokeError,
};

fn assert_error_display<E: Display + Error>(error: E, expected: &str) {
    assert_eq!(error.to_string(), expected);
    assert!(Error::source(&error).is_none());
}

fn assert_result_source<E: Display + Error>(error: E, operation: &str) {
    assert_eq!(
        error.to_string(),
        format!("{operation} operation result is invalid: Failure requires a Result Reason")
    );
    assert_eq!(
        Error::source(&error).map(ToString::to_string).as_deref(),
        Some("Failure requires a Result Reason")
    );
}

fn assert_operation_error_contract<E: Display + Error>(
    operation: &str,
    error: impl IntoIterator<Item = (E, String)>,
    invalid_result: E,
) {
    for (error, expected) in error {
        assert_error_display(error, &expected);
    }
    assert_result_source(invalid_result, operation);
}

macro_rules! lifecycle_error_contract_test {
    ($test_name:ident, $error:ident, $operation:literal, $malformed:ident, $malformed_message:literal) => {
        #[test]
        fn $test_name() {
            let operation = $operation;
            assert_operation_error_contract(
                operation,
                [
                    (
                        $error::UnexpectedOperation,
                        format!("response item is not {operation}"),
                    ),
                    (
                        $error::MissingResultStatus,
                        format!("{operation} result status is missing"),
                    ),
                    (
                        $error::MalformedResponseItem,
                        format!("{operation} response item is malformed"),
                    ),
                    ($error::$malformed, $malformed_message.to_owned()),
                ],
                $error::InvalidOperationResult(ResultValidationError::FailureRequiresReason),
            );
        }
    };
}

lifecycle_error_contract_test!(
    certify_error_display_and_source_cover_every_variant,
    CertifyError,
    "Certify",
    MalformedSuccessPayload,
    "successful Certify response payload is malformed"
);
lifecycle_error_contract_test!(
    check_error_display_and_source_cover_every_variant,
    CheckError,
    "Check",
    MalformedResponsePayload,
    "Check response payload is malformed"
);
lifecycle_error_contract_test!(
    obtain_lease_error_display_and_source_cover_every_variant,
    ObtainLeaseError,
    "Obtain Lease",
    MalformedSuccessPayload,
    "successful Obtain Lease response payload is malformed"
);
lifecycle_error_contract_test!(
    recertify_error_display_and_source_cover_every_variant,
    RecertifyError,
    "Re-certify",
    MalformedSuccessPayload,
    "successful Re-certify response payload is malformed"
);
lifecycle_error_contract_test!(
    rekey_error_display_and_source_cover_every_variant,
    RekeyError,
    "Re-key",
    MalformedSuccessPayload,
    "successful Re-key response payload is malformed"
);
lifecycle_error_contract_test!(
    rekey_key_pair_error_display_and_source_cover_every_variant,
    RekeyKeyPairError,
    "Re-key Key Pair",
    MalformedSuccessPayload,
    "successful Re-key Key Pair payload is malformed"
);
lifecycle_error_contract_test!(
    reprovision_error_display_and_source_cover_every_variant,
    ReProvisionError,
    "Re-Provision",
    MalformedSuccessPayload,
    "successful Re-Provision payload is malformed"
);
lifecycle_error_contract_test!(
    register_error_display_and_source_cover_every_variant,
    RegisterError,
    "Register",
    MalformedSuccessPayload,
    "successful Register response payload is malformed"
);
lifecycle_error_contract_test!(
    revoke_error_display_and_source_cover_every_variant,
    RevokeError,
    "Revoke",
    MalformedSuccessPayload,
    "successful Revoke response payload is malformed"
);

#[test]
fn default_requests_match_their_empty_constructors() {
    assert_eq!(
        format!("{:?}", crate::CertifyRequest::default()),
        format!("{:?}", crate::CertifyRequest::new())
    );
    assert_eq!(
        format!("{:?}", crate::CheckRequest::default()),
        format!("{:?}", crate::CheckRequest::new())
    );
    assert_eq!(
        format!("{:?}", crate::ObtainLeaseRequest::default()),
        format!("{:?}", crate::ObtainLeaseRequest::new())
    );
    assert_eq!(
        format!("{:?}", crate::RecertifyRequest::default()),
        format!("{:?}", crate::RecertifyRequest::new())
    );
    assert_eq!(
        format!("{:?}", crate::RekeyRequest::default()),
        format!("{:?}", crate::RekeyRequest::new())
    );
    assert_eq!(
        format!("{:?}", crate::RekeyKeyPairRequest::default()),
        format!("{:?}", crate::RekeyKeyPairRequest::new())
    );
    assert_eq!(
        format!("{:?}", crate::ReProvisionRequest::default()),
        format!("{:?}", crate::ReProvisionRequest::new())
    );
}
