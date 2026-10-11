//! Source-derived OASIS KMIP v2.1 §6.1.26, Tables 241–243; §7.15–7.16.
//! These fixtures are not official OASIS test cases.

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{InteropError, InteropRequest, InteropResponse, SecretText};
use kmipkit_ttlv::{Structure, Value, ValueView};

const OPERATION: u32 = 0x34;
const FUNCTION: u32 = 0x420160;
const IDENTIFIER: u32 = 0x420161;
const SENTINEL: &str = "interop-identifier-sentinel";

fn response(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<InteropResponse, InteropError> {
    let message = response_message(
        operation,
        status,
        reason,
        (status == 2).then_some(b"correlation".as_slice()),
        payload,
    );
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
    view.children()[0]
        .with_value(|value| assert!(matches!(value, ValueView::Enumeration(0xf1234567))));
    assert_eq!(view.children()[1].tag().raw(), IDENTIFIER);
    view.children()[1]
        .with_value(|value| assert!(matches!(value, ValueView::TextString(v) if v == SENTINEL)));
}

#[test]
fn table_242_success_requires_empty_payload() {
    assert_eq!(
        response(OPERATION, 0, None, Some(Structure::new()))
            .unwrap()
            .result()
            .status()
            .raw(),
        0
    );
    for value in [
        Value::enumeration(1),
        Value::text_string(SENTINEL.to_owned()),
        Value::structure(Structure::new()),
    ] {
        let error =
            response(OPERATION, 0, None, Some(structure([item(FUNCTION, value)]))).unwrap_err();
        assert_eq!(error, InteropError::MalformedResponsePayload);
        assert!(!format!("{error:?} {error}").contains(SENTINEL));
    }
}

#[test]
fn table_243_failure_pending_and_unknown_results_are_preserved() {
    for name in [
        "Invalid Field",
        "Attestation Failed",
        "Attestation Required",
        "Feature Not Supported",
        "Invalid Message",
        "Operation Not Supported",
        "Permission Denied",
        "Response Too Large",
    ] {
        let reason = crate::ResultReason::known_values()
            .iter()
            .find(|(_, n)| *n == name)
            .unwrap()
            .0
            .raw();
        assert_eq!(
            response(OPERATION, 1, Some(reason), None)
                .unwrap()
                .result()
                .reason()
                .unwrap()
                .raw(),
            reason
        );
    }
    for (status, reason) in [(1, Some(0xf1234567)), (2, None), (0xf1234567, None)] {
        let payload = (status != 1).then(Structure::new);
        let result = response(OPERATION, status, reason, payload).unwrap();
        assert_eq!(result.result().status().raw(), status);
        assert_eq!(
            result.result().reason().map(crate::ResultReason::raw),
            reason
        );
        result.with_ttlv(|tree| assert!(!tree.children().is_empty()));
    }
    assert_eq!(
        response(0x3b, 0, None, Some(Structure::new())).unwrap_err(),
        InteropError::UnexpectedOperation
    );
}

#[test]
fn response_errors_are_payload_free_and_preserve_sources() {
    for error in [
        InteropError::UnexpectedOperation,
        InteropError::MissingResultStatus,
        InteropError::MissingResponsePayload,
        InteropError::MalformedResponsePayload,
        InteropError::MalformedResponseItem,
    ] {
        assert!(!format!("{error:?} {error}").contains(SENTINEL));
        assert!(std::error::Error::source(&error).is_none());
    }
    let error =
        InteropError::InvalidOperationResult(crate::ResultValidationError::SuccessForbidsReason);
    assert!(std::error::Error::source(&error).is_some());
    assert!(error.to_string().contains("Interop"));
}

#[test]
fn generic_unknown_response_data_remains_exact_and_redacted() {
    let payload = structure([item(
        0x541234,
        Value::byte_string(b"generic-body-private-sentinel".to_vec()),
    )]);
    let message = response_message(OPERATION, 0xf1234567, Some(0xf7654321), None, Some(payload));
    let batch = message.batch_items().next().unwrap();
    let expected = batch
        .with_ttlv(|tree| {
            tree.children()
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>()
        })
        .unwrap();
    let typed = InteropResponse::try_from_response_item(batch).unwrap();
    typed.with_ttlv(|tree| {
        assert_eq!(tree.children().iter().map(|field| field.tag().raw()).collect::<Vec<_>>(), expected);
        let payload = tree.children().iter().find(|field| field.tag().raw() == 0x42007c).unwrap();
        payload.with_value(|v| {
            let ValueView::Structure(payload) = v else { panic!("Response Payload Structure") };
            assert_eq!(payload.children()[0].tag().raw(), 0x541234);
            payload.children()[0].with_value(|v| assert!(matches!(v, ValueView::ByteString(bytes) if bytes == b"generic-body-private-sentinel")));
        });
    });
    assert_eq!(typed.result().reason().unwrap().raw(), 0xf7654321);
    assert!(!format!("{typed:?}").contains("generic-body-private-sentinel"));
}

#[test]
fn interop_function_standard_vendor_and_future_values_are_open() {
    for function in [1, 2, 3, 0x80000001, 0xf1234567] {
        let payload = InteropRequest::new(function, SecretText::new("id".to_owned()))
            .into_ttlv_payload()
            .unwrap();
        payload.view().children()[0].with_value(|value| {
            assert!(matches!(value, ValueView::Enumeration(raw) if *raw == function))
        });
    }
}

#[test]
fn unknown_request_extensions_preserve_order_without_overriding_table_fields() {
    let request = InteropRequest::new(1, SecretText::new("id".to_owned()))
        .with_extension(item(
            0x541234,
            Value::byte_string(b"extension-private-sentinel".to_vec()),
        ))
        .with_extension(item(0x541235, Value::enumeration(0xf1234567)));
    assert!(!format!("{request:?}").contains("extension-private-sentinel"));
    let payload = request.into_ttlv_payload().unwrap();
    let view = payload.view();
    let fields = view.children();
    assert_eq!(fields[fields.len() - 2].tag().raw(), 0x541234);
    assert_eq!(fields[fields.len() - 1].tag().raw(), 0x541235);
    fields[fields.len() - 2].with_value(|v| {
        assert!(matches!(v, ValueView::ByteString(bytes) if bytes == b"extension-private-sentinel"))
    });
    fields[fields.len() - 1]
        .with_value(|v| assert!(matches!(v, ValueView::Enumeration(0xf1234567))));
    for known in [4325728, 4325729] {
        let error = InteropRequest::new(1, SecretText::new("id".to_owned()))
            .with_extension(item(
                known,
                Value::text_string("extension-private-sentinel".to_owned()),
            ))
            .into_ttlv_payload()
            .unwrap_err();
        assert_eq!(error.kind(), crate::ProtocolErrorKind::InvalidValue);
        assert!(!format!("{error:?} {error}").contains("extension-private-sentinel"));
    }
}

#[test]
fn request_extension_depth_limit_is_checked_before_encoding() {
    let mut nested = Structure::new();
    for _ in 0..63 {
        nested = structure([item(0x541234, Value::structure(nested))]);
    }
    let error = InteropRequest::new(1, SecretText::new("id".to_owned()))
        .with_extension(item(0x541234, Value::structure(nested)))
        .into_ttlv_payload()
        .unwrap_err();
    assert_eq!(error.kind(), crate::ProtocolErrorKind::InvalidValue);
}
