//! Source-derived OASIS KMIP v2.1 §6.1.29, Tables 250–252; §7.19.
//! These fixtures are not official OASIS test cases.

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{LogError, LogRequest, LogResponse, SecretText};
use kmipkit_ttlv::{Structure, Value, ValueView};

const OPERATION: u32 = 0x2c;
const MESSAGE: u32 = 0x420141;
const SENTINEL: &str = "log-identifier-sentinel";

fn response(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<LogResponse, LogError> {
    let message = response_message(
        operation,
        status,
        reason,
        (status == 2).then_some(b"correlation".as_slice()),
        payload,
    );
    LogResponse::try_from_response_item(message.batch_items().next().unwrap())
}

#[test]
fn table_250_required_message_is_preserved_and_redacted() {
    let request = LogRequest::new(SecretText::new(SENTINEL.to_owned()));
    request.with_log_message(|value| assert_eq!(value, SENTINEL));
    assert!(!format!("{request:?}").contains(SENTINEL));
    let payload = request.into_ttlv_payload().unwrap();
    assert!(!format!("{payload:?}").contains(SENTINEL));
    let view = payload.view();
    assert_eq!(view.children().len(), 1);
    assert_eq!(view.children()[0].tag().raw(), MESSAGE);
    view.children()[0]
        .with_value(|value| assert!(matches!(value, ValueView::TextString(v) if v == SENTINEL)));
}

#[test]
fn table_251_success_requires_empty_payload() {
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
            response(OPERATION, 0, None, Some(structure([item(MESSAGE, value)]))).unwrap_err();
        assert_eq!(error, LogError::MalformedResponsePayload);
        assert!(!format!("{error:?} {error}").contains(SENTINEL));
    }
}

#[test]
fn table_252_failure_pending_and_unknown_results_are_preserved() {
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
        LogError::UnexpectedOperation
    );
}

#[test]
fn response_errors_are_payload_free_and_preserve_sources() {
    for error in [
        LogError::UnexpectedOperation,
        LogError::MissingResultStatus,
        LogError::MissingResponsePayload,
        LogError::MalformedResponsePayload,
        LogError::MalformedResponseItem,
    ] {
        assert!(!format!("{error:?} {error}").contains(SENTINEL));
        assert!(std::error::Error::source(&error).is_none());
    }
    let error =
        LogError::InvalidOperationResult(crate::ResultValidationError::SuccessForbidsReason);
    assert!(std::error::Error::source(&error).is_some());
    assert!(error.to_string().contains("Log"));
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
    let typed = LogResponse::try_from_response_item(batch).unwrap();
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
fn unknown_request_extensions_preserve_order_without_overriding_table_fields() {
    let request = LogRequest::new(SecretText::new("message".to_owned()))
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
    for known in [4325697] {
        let error = LogRequest::new(SecretText::new("message".to_owned()))
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
    let error = LogRequest::new(SecretText::new("message".to_owned()))
        .with_extension(item(0x541234, Value::structure(nested)))
        .into_ttlv_payload()
        .unwrap_err();
    assert_eq!(error.kind(), crate::ProtocolErrorKind::InvalidValue);
}
