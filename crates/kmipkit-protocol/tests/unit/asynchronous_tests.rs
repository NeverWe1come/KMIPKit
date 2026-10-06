//! Safe-error and local model-boundary tests for KMIP 2.1 asynchronous payloads.
//!
//! The operation shapes exercised by these helpers are specified in OASIS KMIP
//! v2.1 §§6.1.5, 6.1.38, and 6.1.39, Tables 176, 276, and 278.

use std::error::Error;

use kmipkit_ttlv::{Item, Structure, Value};

use crate::{
    AsynchronousOperationError, ProtocolCauseCategory, ProtocolErrorKind, ResultValidationError,
};

#[test]
fn asynchronous_response_errors_display_safe_messages_and_preserve_only_typed_causes() {
    let cases = [
        (
            AsynchronousOperationError::UnexpectedOperation,
            "response item is for a different operation",
            false,
        ),
        (
            AsynchronousOperationError::MissingResultStatus,
            "asynchronous operation result status is missing",
            false,
        ),
        (
            AsynchronousOperationError::InvalidOperationResult(
                ResultValidationError::FailureRequiresReason,
            ),
            "asynchronous operation result is invalid: Failure requires a Result Reason",
            true,
        ),
        (
            AsynchronousOperationError::MissingAsynchronousCorrelationValue,
            "required asynchronous correlation value is missing or malformed",
            false,
        ),
        (
            AsynchronousOperationError::MissingResponsePayload,
            "required asynchronous response payload is missing",
            false,
        ),
        (
            AsynchronousOperationError::UnexpectedResponsePayload,
            "asynchronous response payload is forbidden for this result",
            false,
        ),
        (
            AsynchronousOperationError::MalformedResponsePayload,
            "asynchronous response payload is malformed",
            false,
        ),
        (
            AsynchronousOperationError::ForbiddenResultStatus,
            "result status is forbidden for this operation",
            false,
        ),
    ];

    for (error, expected_message, has_source) in cases {
        assert_eq!(error.to_string(), expected_message);
        assert_eq!(Error::source(&error).is_some(), has_source);
    }

    let source = Error::source(&AsynchronousOperationError::InvalidOperationResult(
        ResultValidationError::FailureRequiresReason,
    ))
    .expect("result conversion errors retain their typed validation source");
    assert_eq!(
        source.downcast_ref::<ResultValidationError>(),
        Some(&ResultValidationError::FailureRequiresReason)
    );
}

#[test]
fn asynchronous_payload_helpers_sanitize_invalid_tags_and_depth() {
    let invalid_tag = crate::asynchronous::item(0x0100_0000, Value::integer(7))
        .expect_err("raw tags wider than 24 bits are invalid TTLV model values");
    assert_eq!(invalid_tag.kind(), ProtocolErrorKind::InvalidValue);
    assert_eq!(
        invalid_tag.cause_category(),
        ProtocolCauseCategory::InvalidValue
    );
    assert!(!invalid_tag.to_string().contains("16777216"));

    let tag = crate::asynchronous::tag(0x0042_0006)
        .expect("the Asynchronous Correlation Value tag is allocated by KMIP 2.1");
    let mut nested = Structure::new();
    let mut depth_error = None;
    for _ in 0..=64 {
        let child = Item::new(tag, Value::structure(nested))
            .expect("a checked tag and owned Structure form an Item");
        match crate::asynchronous::structure([child]) {
            Ok(next) => nested = next,
            Err(error) => {
                depth_error = Some(error);
                break;
            }
        }
    }
    let depth_error =
        depth_error.expect("asynchronous payload construction enforces the local depth limit");

    assert_eq!(depth_error.kind(), ProtocolErrorKind::InvalidValue);
    assert_eq!(
        depth_error.cause_category(),
        ProtocolCauseCategory::InvalidValue
    );
    assert!(!depth_error.to_string().contains("StructureDepthExceeded"));
}
