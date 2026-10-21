//! Project-derived checks for OASIS KMIP Specification v2.1 §6.1.53,
//! Tables 328–330, and Defaults Information §7.11. These are not official
//! OASIS Test Cases.
//! Traceability: KMIPKIT-025-T328-REQUEST, T329-RESPONSE, T330-ERROR,
//! KMIPKIT-025-COMMON-RESULT.

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{
    ProtocolErrorKind, ResponseMessage, ResultReason, ResultValidationError, SetDefaultsError,
    SetDefaultsRequest, SetDefaultsResponse,
};
use kmipkit_ttlv::{ItemType, Structure, Value, ValueView};
use std::error::Error as _;

const SET_DEFAULTS: u32 = 0x0000_0036;
const DEFAULTS_INFORMATION: u32 = 0x0042_0152;
const VENDOR_MEMBER: u32 = 0x0054_1234;
const TABLE_330_DISTINCT_REASONS: [u32; 9] = [0x07, 0x30, 0x15, 0x14, 0x08, 0x04, 0x05, 0x0C, 0x02];

fn response_item(message: &ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("project fixture contains one response item")
}

#[test]
fn table_328_preserves_omitted_empty_and_populated_defaults_information() {
    let omitted_request = SetDefaultsRequest::default();
    assert!(omitted_request.defaults_information().is_none());
    assert_eq!(
        format!("{omitted_request:?}"),
        "SetDefaultsRequest { has_defaults_information: false, defaults_information: \"[REDACTED]\" }"
    );
    let omitted = omitted_request
        .to_ttlv_payload()
        .expect("omitting Defaults Information is a valid request");
    assert!(omitted.view().children().is_empty());

    let supplied_empty = SetDefaultsRequest::new().with_defaults_information(Structure::new());
    assert!(
        supplied_empty
            .defaults_information()
            .is_some_and(|information| information.children().is_empty())
    );
    let empty_payload = supplied_empty
        .to_ttlv_payload()
        .expect("an explicitly supplied empty structure remains present");
    let empty_view = empty_payload.view();
    let empty_fields = empty_view.children();
    assert_eq!(empty_fields.len(), 1);
    assert_eq!(empty_fields[0].tag().raw(), DEFAULTS_INFORMATION);
    assert_eq!(empty_fields[0].item_type(), ItemType::Structure);
    empty_fields[0].with_value(|value| {
        let ValueView::Structure(information) = value else {
            panic!("Defaults Information is a Structure");
        };
        assert!(information.children().is_empty());
    });

    let supplied = SetDefaultsRequest::new().with_defaults_information(structure([
        item(VENDOR_MEMBER, Value::integer(7)),
        item(
            VENDOR_MEMBER,
            Value::structure(structure([item(
                VENDOR_MEMBER,
                Value::text_string("ordered-default-value".to_owned()),
            )])),
        ),
    ]));
    let payload = supplied
        .to_ttlv_payload()
        .expect("generic Defaults Information is retained");
    let view = payload.view();
    let fields = view.children();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), DEFAULTS_INFORMATION);
    fields[0].with_value(|value| {
        let ValueView::Structure(information) = value else {
            panic!("Defaults Information is a Structure");
        };
        assert_eq!(information.children().len(), 2);
        assert_eq!(information.children()[0].tag().raw(), VENDOR_MEMBER);
        assert_eq!(information.children()[1].tag().raw(), VENDOR_MEMBER);
        assert!(
            information.children()[0].with_value(|value| matches!(value, ValueView::Integer(7)))
        );
        information.children()[1].with_value(|value| {
            let ValueView::Structure(nested) = value else {
                panic!("second default child is a Structure");
            };
            assert!(nested.children()[0].with_value(|value| matches!(
                value,
                ValueView::TextString("ordered-default-value")
            )));
        });
    });
}

#[test]
fn table_328_rejects_defaults_information_that_exceeds_the_local_nesting_limit() {
    let mut information = Structure::new();
    for _ in 0..63 {
        let mut parent = Structure::new();
        parent
            .try_push(item(VENDOR_MEMBER, Value::structure(information)))
            .expect("Defaults Information can be represented at depth 64");
        information = parent;
    }

    let error = SetDefaultsRequest::new()
        .with_defaults_information(information)
        .to_ttlv_payload()
        .expect_err("adding the Request Payload wrapper would exceed the local depth limit");
    assert_eq!(error.kind(), ProtocolErrorKind::InvalidValue);
}

#[test]
fn set_defaults_errors_format_messages_and_expose_only_wrapped_sources() {
    let result_error = ResultValidationError::FailureRequiresReason;
    let cases = [
        (
            SetDefaultsError::UnexpectedOperation,
            "response item is not Set Defaults",
        ),
        (
            SetDefaultsError::MissingResultStatus,
            "Set Defaults result status is missing",
        ),
        (
            SetDefaultsError::InvalidOperationResult(result_error),
            "invalid Set Defaults result: Failure requires a Result Reason",
        ),
        (
            SetDefaultsError::MissingSuccessPayload,
            "successful Set Defaults payload is missing",
        ),
        (
            SetDefaultsError::MalformedResponseItem,
            "Set Defaults response item is malformed",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
    }

    let invalid_result = SetDefaultsError::InvalidOperationResult(result_error);
    assert_eq!(
        invalid_result
            .source()
            .and_then(|source| source.downcast_ref::<ResultValidationError>()),
        Some(&result_error)
    );
    for error in [
        SetDefaultsError::UnexpectedOperation,
        SetDefaultsError::MissingResultStatus,
        SetDefaultsError::MissingSuccessPayload,
        SetDefaultsError::MalformedResponseItem,
    ] {
        assert!(error.source().is_none());
    }
}

#[test]
fn table_329_empty_success_retains_the_complete_ttlv_response() {
    let message = response_message(SET_DEFAULTS, 0, None, None, Some(Structure::new()));
    let response = SetDefaultsResponse::try_from_response_item(response_item(&message))
        .expect("Table 329 empty success response is typed");
    assert_eq!(response.result().status().raw(), 0);
    response.with_ttlv(|item| {
        let payload = item
            .children()
            .iter()
            .find(|field| field.tag().raw() == 0x0042_007C)
            .expect("successful response retains Response Payload");
        payload.with_value(|value| {
            let ValueView::Structure(payload) = value else {
                panic!("Response Payload is a Structure");
            };
            assert!(payload.children().is_empty());
        });
    });
}

#[test]
fn table_329_retains_unknown_response_fields_without_rejecting_them() {
    let message = response_message(
        SET_DEFAULTS,
        0,
        None,
        None,
        Some(structure([item(VENDOR_MEMBER, Value::integer(23))])),
    );
    let response = SetDefaultsResponse::try_from_response_item(response_item(&message))
        .expect("unknown valid response extensions are retained");
    response.with_ttlv(|item| {
        let payload = item
            .children()
            .iter()
            .find(|field| field.tag().raw() == 0x0042_007C)
            .expect("Response Payload is retained");
        payload.with_value(|value| {
            let ValueView::Structure(payload) = value else {
                panic!("Response Payload is a Structure");
            };
            assert_eq!(payload.children().len(), 1);
            assert_eq!(payload.children()[0].tag().raw(), VENDOR_MEMBER);
            assert!(
                payload.children()[0].with_value(|value| matches!(value, ValueView::Integer(23)))
            );
        });
    });
}

#[test]
fn table_329_success_without_response_payload_is_rejected_by_common_validator() {
    // §8.6/Table 399 requires Response Payload for a non-Failure result.
    // Construct the malformed tree directly because the shared response
    // fixture intentionally accepts only valid ResponseMessage values.
    let version = structure([
        item(0x0042_006A, Value::integer(2)),
        item(0x0042_006B, Value::integer(1)),
    ]);
    let header = structure([
        item(0x0042_0069, Value::structure(version)),
        item(0x0042_0092, Value::date_time(1)),
        item(0x0042_000D, Value::integer(1)),
    ]);
    let batch = structure([
        item(0x0042_005C, Value::enumeration(SET_DEFAULTS)),
        item(0x0042_007F, Value::enumeration(0)),
    ]);
    let message = structure([
        item(0x0042_007A, Value::structure(header)),
        item(0x0042_000F, Value::structure(batch)),
    ]);
    assert!(ResponseMessage::try_from_ttlv(message).is_err());
}

#[test]
fn table_330_preserves_all_distinct_reasons_and_open_unknown_reason() {
    // Table 330 repeats Invalid Field; exercise each distinct value once.
    for reason in TABLE_330_DISTINCT_REASONS.into_iter().chain([0x8000_0054]) {
        let message = response_message(SET_DEFAULTS, 1, Some(reason), None, None);
        let response = SetDefaultsResponse::try_from_response_item(response_item(&message))
            .expect("Table 330 Failure uses the common result contract");
        assert_eq!(response.result().status().raw(), 1);
        assert_eq!(
            response.result().reason(),
            Some(ResultReason::from_raw(reason))
        );
    }
}

#[test]
fn failure_and_pending_do_not_expose_generic_response_ttlv() {
    let failure_message = response_message(
        SET_DEFAULTS,
        1,
        Some(0x07),
        Some(b"defaults update was rejected"),
        None,
    );
    let failure = SetDefaultsResponse::try_from_response_item(response_item(&failure_message))
        .expect("Table 330 Failure uses the common result contract");
    assert_eq!(failure.result().status().raw(), 1);
    assert!(failure.with_ttlv(|_| ()).is_none());

    let pending_message = response_message(
        SET_DEFAULTS,
        2,
        None,
        Some(b"set-defaults-pending"),
        Some(Structure::new()),
    );
    let pending = SetDefaultsResponse::try_from_response_item(response_item(&pending_message))
        .expect("Pending uses the common result contract");
    assert_eq!(pending.result().status().raw(), 2);
    assert!(pending.with_ttlv(|_| ()).is_none());
}

#[test]
fn response_retains_unknown_payload_after_source_message_is_dropped() {
    const OPAQUE_VALUE: &str = "opaque defaults response survives source drop";
    let response = {
        let payload = structure([item(
            VENDOR_MEMBER,
            Value::text_string(OPAQUE_VALUE.to_owned()),
        )]);
        let message = response_message(SET_DEFAULTS, 0, None, None, Some(payload));
        SetDefaultsResponse::try_from_response_item(response_item(&message))
            .expect("unknown Table 329 extension remains representable")
    };

    response.with_ttlv(|item| {
        let payload = item
            .children()
            .iter()
            .find(|field| field.tag().raw() == 0x0042_007C)
            .expect("owned response retains its payload");
        payload.with_value(|value| {
            let ValueView::Structure(payload) = value else {
                panic!("Response Payload is a Structure");
            };
            assert_eq!(payload.children().len(), 1);
            assert!(
                payload.children()[0]
                    .with_value(|value| matches!(value, ValueView::TextString(OPAQUE_VALUE)))
            );
        });
    });
}

#[test]
fn request_and_response_debug_redact_defaults_values() {
    const SENSITIVE_VALUE: &str = "sensitive-default-value";
    let request = SetDefaultsRequest::new().with_defaults_information(structure([item(
        VENDOR_MEMBER,
        Value::text_string(SENSITIVE_VALUE.to_owned()),
    )]));
    assert!(!format!("{request:?}").contains(SENSITIVE_VALUE));

    let response = {
        let payload = structure([item(
            VENDOR_MEMBER,
            Value::text_string(SENSITIVE_VALUE.to_owned()),
        )]);
        let message = response_message(SET_DEFAULTS, 0, None, None, Some(payload));
        SetDefaultsResponse::try_from_response_item(response_item(&message))
            .expect("unknown response payload remains retained")
    };
    assert!(!format!("{response:?}").contains(SENSITIVE_VALUE));
}

#[test]
fn pending_is_preserved_without_defaults_response_fields() {
    let message = response_message(
        SET_DEFAULTS,
        2,
        None,
        Some(b"set-defaults-pending"),
        Some(Structure::new()),
    );
    let response = SetDefaultsResponse::try_from_response_item(response_item(&message))
        .expect("Pending uses the common result contract");
    assert_eq!(response.result().status().raw(), 2);
}

#[test]
fn typed_response_rejects_another_operation() {
    let message = response_message(0x0000_0001, 1, Some(0x07), None, None);
    assert!(SetDefaultsResponse::try_from_response_item(response_item(&message)).is_err());
}
