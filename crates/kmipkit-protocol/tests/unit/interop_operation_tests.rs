//! Source-derived OASIS KMIP v2.1 §6.1.26, Tables 241–243; §7.15–7.16.
//! These fixtures are not official OASIS test cases.

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{InteropError, InteropRequest, InteropResponse, SecretText};
use kmipkit_ttlv::{Structure, Value, ValueView};

const OPERATION: u32 = 0x34;
const FUNCTION: u32 = 0x420160;
const IDENTIFIER: u32 = 0x420161;
const SENTINEL: &str = "interop-identifier-sentinel";

fn response(operation: u32, status: u32, reason: Option<u32>, payload: Option<Structure>) -> Result<InteropResponse, InteropError> {
    let message = response_message(operation, status, reason, (status == 2).then_some(b"correlation".as_slice()), payload);
    InteropResponse::try_from_response_item(message.batch_items().next().unwrap())
}

#[test]
fn table_241_required_fields_preserve_order_and_unknown_function() {
    let request = InteropRequest::new(0xf1234567, SecretText::new(SENTINEL.to_owned()));
    assert_eq!(request.interop_function(), 0xf1234567);
    request.with_interop_identifier(|value| assert_eq!(value, SENTINEL));
    assert!(!format!("{request:?}").contains(SENTINEL));
    let payload = request.into_ttlv_payload().unwrap();
    let view = payload.view();
    assert_eq!(view.children().len(), 2);
    assert_eq!(view.children()[0].tag().raw(), FUNCTION);
    view.children()[0].with_value(|value| assert!(matches!(value, ValueView::Enumeration(0xf1234567))));
    assert_eq!(view.children()[1].tag().raw(), IDENTIFIER);
    view.children()[1].with_value(|value| assert!(matches!(value, ValueView::TextString(v) if v == SENTINEL)));
}

#[test]
fn table_242_success_requires_empty_payload() {
    assert_eq!(response(OPERATION, 0, None, Some(Structure::new())).unwrap().result().status().raw(), 0);
    assert_eq!(response(OPERATION, 0, None, None).unwrap_err(), InteropError::MissingResponsePayload);
    for value in [Value::enumeration(1), Value::text_string(SENTINEL.to_owned()), Value::structure(Structure::new())] {
        let error = response(OPERATION, 0, None, Some(structure([item(FUNCTION, value)]))).unwrap_err();
        assert_eq!(error, InteropError::MalformedResponsePayload);
        assert!(!format!("{error:?} {error}").contains(SENTINEL));
    }
}

#[test]
fn table_243_failure_pending_and_unknown_results_are_preserved() {
    for name in ["Invalid Field", "Attestation Failed", "Attestation Required", "Feature Not Supported", "Invalid Message", "Operation Not Supported", "Permission Denied", "Response Too Large"] {
        let reason = crate::ResultReason::known_values().iter().find(|(_, n)| *n == name).unwrap().0.raw();
        assert_eq!(response(OPERATION, 1, Some(reason), None).unwrap().result().reason().unwrap().raw(), reason);
    }
    for (status, reason) in [(1, Some(0xf1234567)), (2, None), (0xf1234567, None)] {
        let result = response(OPERATION, status, reason, None).unwrap();
        assert_eq!(result.result().status().raw(), status);
        assert_eq!(result.result().reason().map(crate::ResultReason::raw), reason);
        result.with_ttlv(|tree| assert!(!tree.children().is_empty()));
    }
    assert_eq!(response(0x3b, 0, None, Some(Structure::new())).unwrap_err(), InteropError::UnexpectedOperation);
}

#[test]
fn response_errors_are_payload_free_and_preserve_sources() {
    for error in [InteropError::UnexpectedOperation, InteropError::MissingResultStatus, InteropError::MissingResponsePayload, InteropError::MalformedResponsePayload, InteropError::MalformedResponseItem] {
        assert!(!format!("{error:?} {error}").contains(SENTINEL));
        assert!(std::error::Error::source(&error).is_none());
    }
    let error = InteropError::InvalidOperationResult(crate::ResultValidationError::SuccessForbidsReason);
    assert!(std::error::Error::source(&error).is_some());
    assert!(error.to_string().contains("Interop"));
}
