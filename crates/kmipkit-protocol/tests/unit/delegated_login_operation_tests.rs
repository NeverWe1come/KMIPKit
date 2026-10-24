//! Delegated Login contracts from OASIS KMIP Specification v2.1 §6.1.12,
//! Tables 199–201, with Ticket encoding from §7.39, Table 391.
//!
//! Traceability: `KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN`,
//! `KMIPKIT-024-T199-REQUEST`, `KMIPKIT-024-T200-RESPONSE`, and
//! `KMIPKIT-024-T201-ERROR`. These are source-derived tests, not official
//! Test Case executions.

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{
    DelegatedLoginError, DelegatedLoginRequest, DelegatedLoginResponse, ResponseMessage,
    ResultReason, ResultValidationError,
};
use kmipkit_ttlv::{Structure, Value, ValueView};

const DELEGATED_LOGIN: u32 = 0x0000_002F;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const TICKET: u32 = 0x0042_0149;
const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;
const LEASE_TIME: u32 = 0x0042_0049;
const REQUEST_COUNT: u32 = 0x0042_014C;
const USAGE_LIMITS: u32 = 0x0042_0095;
const USAGE_LIMITS_COUNT: u32 = 0x0042_0096;
const USAGE_LIMITS_TOTAL: u32 = 0x0042_0097;
const USAGE_LIMITS_UNIT: u32 = 0x0042_0098;
const RIGHTS: u32 = 0x0042_014D;
const RIGHT: u32 = 0x0042_014E;
const VENDOR_EXTENSION: u32 = 0x0054_1234;
const SECRET_SENTINEL: &[u8] = b"delegated-login-ticket-secret";

fn response_item(message: &ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one validated response batch item")
}

fn item_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

fn valid_rights() -> Structure {
    structure([item(RIGHT, Value::structure(Structure::new()))])
}

fn valid_usage_limits() -> Structure {
    // KMIP Specification v2.1 §7.40 Table 392 requires all three members;
    // §11.60 Table 491 assigns Byte the Usage Limits Unit value 1.
    structure([
        item(USAGE_LIMITS_TOTAL, Value::long_integer(10)),
        item(USAGE_LIMITS_COUNT, Value::long_integer(2)),
        item(USAGE_LIMITS_UNIT, Value::enumeration(1)),
    ])
}

fn structure_at_depth(depth: usize) -> Structure {
    assert!((1..=64).contains(&depth));
    let mut nested = Structure::new();
    for _ in 1..depth {
        nested = structure([item(VENDOR_EXTENSION, Value::structure(nested))]);
    }
    nested
}

fn valid_ticket() -> Structure {
    structure([
        item(TICKET_TYPE, Value::enumeration(0x8000_0042)),
        item(TICKET_VALUE, Value::byte_string(SECRET_SENTINEL.to_vec())),
        item(VENDOR_EXTENSION, Value::enumeration(0xF123_4567)),
    ])
}

fn delegated_login_response(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<DelegatedLoginResponse, DelegatedLoginError> {
    delegated_login_response_for(DELEGATED_LOGIN, status, reason, payload)
}

fn delegated_login_response_for(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<DelegatedLoginResponse, DelegatedLoginError> {
    let message = response_message(operation, status, reason, None, payload);
    DelegatedLoginResponse::try_from_response_item(response_item(&message))
}

#[test]
fn request_emits_table_199_fields_in_order_and_preserves_both_lease_time_forms() {
    let rights_only = DelegatedLoginRequest::new(valid_rights())
        .into_ttlv_payload()
        .expect("all optional fields may be omitted");
    assert_eq!(item_tags(&rights_only), [RIGHTS]);

    let interval = DelegatedLoginRequest::new(valid_rights())
        .with_lease_time_interval(60)
        .with_request_count(4)
        .with_usage_limits(valid_usage_limits())
        .into_ttlv_payload()
        .expect("valid Table 199 values encode");
    let date_time = DelegatedLoginRequest::new(valid_rights())
        .with_lease_time_date_time(1_700_000_000)
        .into_ttlv_payload()
        .expect("Date Time is the alternate permitted Lease Time form");

    assert_eq!(
        item_tags(&interval),
        [LEASE_TIME, REQUEST_COUNT, USAGE_LIMITS, RIGHTS]
    );
    interval.view().children()[0].with_value(|value| {
        assert!(matches!(value, ValueView::Interval(60)));
    });
    interval.view().children()[1].with_value(|value| {
        assert!(matches!(value, ValueView::Integer(4)));
    });
    interval.view().children()[2].with_value(|value| {
        assert!(matches!(value, ValueView::Structure(_)));
    });
    interval.view().children()[3].with_value(|value| {
        assert!(matches!(value, ValueView::Structure(_)));
    });
    assert_eq!(item_tags(&date_time), [LEASE_TIME, RIGHTS]);
    date_time.view().children()[0].with_value(|value| {
        assert!(matches!(value, ValueView::DateTime(1_700_000_000)));
    });
}

#[test]
fn request_rejects_a_nested_optional_value_that_exceeds_ttlv_depth() {
    let error = DelegatedLoginRequest::new(valid_rights())
        .with_usage_limits(structure_at_depth(64))
        .into_ttlv_payload()
        .expect_err("adding Usage Limits would exceed the generic TTLV depth limit");

    assert_eq!(error.kind(), crate::ProtocolErrorKind::InvalidValue);
}

#[test]
fn request_debug_does_not_format_rights_ttlv_content() {
    const RIGHTS_SECRET: &str = "rights-debug-sentinel";
    let rights = structure([item(
        RIGHT,
        Value::structure(structure([item(
            0x0042_0042,
            Value::text_string(RIGHTS_SECRET.to_owned()),
        )])),
    )]);
    let request = DelegatedLoginRequest::new(rights);
    let debug = format!("{request:?}");

    assert!(debug.contains("PRESERVED TTLV"));
    assert!(!debug.contains(RIGHTS_SECRET));
}

#[test]
fn successful_response_requires_ticket_and_preserves_open_type_and_extensions() {
    let payload = structure([
        item(VENDOR_EXTENSION, Value::enumeration(42)),
        item(TICKET, Value::structure(valid_ticket())),
    ]);
    let response = delegated_login_response(SUCCESS, None, Some(payload))
        .expect("Table 200 success contains the required Ticket");
    let ticket = response
        .ticket()
        .expect("successful response exposes Ticket");

    assert_eq!(ticket.ticket_type().raw(), 0x8000_0042);
    ticket.with_ticket_value(|actual| assert_eq!(actual, SECRET_SENTINEL));
    let retained_tags = ticket.with_ttlv(|tree| {
        tree.children()
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>()
    });
    assert_eq!(retained_tags, [TICKET_TYPE, TICKET_VALUE, VENDOR_EXTENSION]);
    let debug = format!("{response:?}");
    assert!(!debug.contains("delegated-login-ticket-secret"));
    assert!(debug.contains("REDACTED"));
}

#[test]
fn failure_preserves_each_distinct_table_201_reason_without_a_ticket() {
    for raw_reason in [2, 4, 5, 7, 8, 12, 20, 21] {
        let response = delegated_login_response(OPERATION_FAILED, Some(raw_reason), None)
            .expect("a valid Table 201 failure remains inspectable");
        assert_eq!(response.result().status().raw(), OPERATION_FAILED);
        assert_eq!(
            response.result().reason(),
            Some(ResultReason::from_raw(raw_reason))
        );
        assert!(response.ticket().is_none());
    }
}

#[test]
fn malformed_success_ticket_is_rejected_without_echoing_ticket_bytes() {
    let malformed_ticket = structure([item(TICKET_TYPE, Value::enumeration(1))]);
    let payload = structure([item(TICKET, Value::structure(malformed_ticket))]);
    let error = delegated_login_response(SUCCESS, None, Some(payload))
        .expect_err("Table 200 requires a structurally valid Ticket");

    assert!(!error.to_string().contains("delegated-login-ticket-secret"));
}

#[test]
fn response_errors_have_payload_free_display_and_sources() {
    assert_eq!(
        delegated_login_response_for(DELEGATED_LOGIN + 1, OPERATION_FAILED, Some(2), None)
            .unwrap_err(),
        DelegatedLoginError::UnexpectedOperation
    );

    let error = DelegatedLoginError::UnexpectedOperation;
    assert_eq!(error.to_string(), "response item is not Delegated Login");
    assert!(std::error::Error::source(&error).is_none());

    let error = DelegatedLoginError::MissingResultStatus;
    assert_eq!(
        error.to_string(),
        "Delegated Login result status is missing"
    );
    assert!(std::error::Error::source(&error).is_none());

    let cause = ResultValidationError::SuccessForbidsReason;
    let error = DelegatedLoginError::InvalidOperationResult(cause);
    assert_eq!(
        error.to_string(),
        "Delegated Login operation result is invalid: Success forbids a Result Reason"
    );
    assert_eq!(
        std::error::Error::source(&error).unwrap().to_string(),
        cause.to_string()
    );

    let error = DelegatedLoginError::MalformedSuccessPayload;
    assert_eq!(
        error.to_string(),
        "successful Delegated Login Ticket payload is malformed"
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn successful_response_rejects_duplicate_ticket_fields() {
    let payload = structure([
        item(TICKET, Value::structure(valid_ticket())),
        item(TICKET, Value::structure(valid_ticket())),
    ]);
    let error = delegated_login_response(SUCCESS, None, Some(payload))
        .expect_err("Table 200 contains a single required Ticket field");

    assert_eq!(error, DelegatedLoginError::MalformedSuccessPayload);
}

#[test]
fn unknown_ticket_enumeration_values_are_not_rejected() {
    for raw_type in [0, 2, 0x7FFF_FFFF, 0x8000_0001, u32::MAX] {
        let ticket = structure([
            item(TICKET_TYPE, Value::enumeration(raw_type)),
            item(TICKET_VALUE, Value::byte_string(vec![0x5A])),
        ]);
        let payload = structure([item(TICKET, Value::structure(ticket))]);
        let response = delegated_login_response(SUCCESS, None, Some(payload))
            .expect("unknown or extension values remain representable");
        assert_eq!(
            response.ticket().expect("Ticket").ticket_type().raw(),
            raw_type
        );
    }
}
