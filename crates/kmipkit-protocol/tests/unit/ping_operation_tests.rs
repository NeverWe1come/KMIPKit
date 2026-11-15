//! Ping behavior derived from OASIS KMIP v2.1 §6.1.36, Tables 271–272.
//! These structural vectors do not claim an official OASIS fixture pass.

use crate::query_ping_fixtures::{
    PING_OPERATION, item, response_item, response_message, structure,
};
use crate::{PingError, PingRequest, PingResponse, ResultValidationError};
use kmipkit_ttlv::Value;
use std::error::Error;

#[test]
fn ping_request_has_an_empty_operation_payload() {
    let payload = PingRequest::new()
        .to_ttlv_payload()
        .expect("the empty Ping payload is representable");

    assert!(payload.view().children().is_empty());
}

#[test]
fn successful_ping_requires_an_empty_response_payload() {
    let message = response_message(PING_OPERATION, 0, None, None, Some(structure([])));
    let response = PingResponse::try_from_response_item(response_item(&message))
        .expect("an empty successful Ping response is valid");

    assert_eq!(response.result().status().raw(), 0);
}

#[test]
fn ping_failure_preserves_the_common_result_without_a_success_payload() {
    let message = response_message(
        PING_OPERATION,
        1,
        Some(1),
        Some("server refused Ping"),
        None,
    );
    let response = PingResponse::try_from_response_item(response_item(&message))
        .expect("a valid KMIP failure has no Ping response payload");

    assert_eq!(response.result().status().raw(), 1);
    assert_eq!(
        response.result().reason().map(crate::ResultReason::raw),
        Some(1)
    );
    assert_eq!(
        response
            .result()
            .message()
            .map(crate::ResultMessage::as_str),
        Some("server refused Ping")
    );
}

#[test]
fn successful_ping_rejects_a_nonempty_operation_payload() {
    let message = response_message(
        PING_OPERATION,
        0,
        None,
        None,
        Some(structure([item(
            0x0042_0012,
            Value::text_string("unexpected".to_owned()),
        )])),
    );

    assert!(PingResponse::try_from_response_item(response_item(&message)).is_err());
}

#[test]
fn ping_response_rejects_a_different_operation() {
    let message = response_message(0x0000_001e, 0, None, None, Some(structure([])));

    assert!(PingResponse::try_from_response_item(response_item(&message)).is_err());
}

#[test]
fn ping_errors_format_safely_and_retain_only_typed_sources() {
    let errors = [
        PingError::UnexpectedOperation,
        PingError::MissingResultStatus,
        PingError::InvalidOperationResult(ResultValidationError::FailureRequiresReason),
        PingError::MissingResponsePayload,
        PingError::MalformedResponsePayload,
    ];
    for error in errors {
        assert_ne!(error.to_string(), "");
        assert_eq!(
            error.source().is_some(),
            matches!(error, PingError::InvalidOperationResult(_))
        );
    }
}
