//! Source-derived Logout contracts from OASIS KMIP v2.1 §6.1.31,
//! Tables 256–258, and the Ticket structure in §7.39, Table 391.
//!
//! Traceability: `KMIPKIT-ELEM-OP-C2S-LOGOUT`,
//! `KMIPKIT-024-T256-REQUEST`, `KMIPKIT-024-T257-RESPONSE`, and
//! `KMIPKIT-024-T258-ERROR`. No direct official Logout test case is cataloged.

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{
    LogoutError, LogoutRequest, LogoutResponse, ResponseMessage, ResultReason, SecretBytes, Ticket,
    TicketType,
};
use kmipkit_ttlv::{Structure, Value};

const LOGOUT: u32 = 0x0000_002E;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const TICKET: u32 = 0x0042_0149;
const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;
const SECRET_SENTINEL: &[u8] = b"logout-ticket-secret-sentinel";

fn response_item(message: &ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one validated response batch item")
}

fn ticket() -> Ticket {
    Ticket::new(
        TicketType::LOGIN,
        SecretBytes::new(SECRET_SENTINEL.to_vec()),
    )
    .expect("the required Ticket uses representable fields")
}

fn typed_response(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<LogoutResponse, LogoutError> {
    let message = response_message(operation, status, reason, None, payload);
    LogoutResponse::try_from_response_item(response_item(&message))
}

#[test]
fn table_256_request_requires_and_preserves_the_explicit_secret_ticket() {
    let request = LogoutRequest::new(ticket());
    assert!(!format!("{request:?}").contains("logout-ticket-secret-sentinel"));
    let payload = request
        .into_ttlv_payload()
        .expect("the required Table 256 Ticket is encodable");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), TICKET);
    fields[0].with_value(|value| {
        let kmipkit_ttlv::ValueView::Structure(ticket) = value else {
            panic!("Table 256 Ticket is a Structure")
        };
        assert_eq!(ticket.children()[0].tag().raw(), TICKET_TYPE);
        assert_eq!(ticket.children()[1].tag().raw(), TICKET_VALUE);
        ticket.children()[1].with_value(|value| {
            let kmipkit_ttlv::ValueView::ByteString(bytes) = value else {
                panic!("Ticket Value is a Byte String")
            };
            assert_eq!(bytes, SECRET_SENTINEL);
        });
    });
}

#[test]
fn table_257_success_requires_an_empty_response_payload() {
    let response = typed_response(LOGOUT, SUCCESS, None, Some(Structure::new()))
        .expect("Table 257 is an empty success structure");
    assert_eq!(response.result().status().raw(), SUCCESS);
}

#[test]
fn success_with_fields_in_payload_is_rejected() {
    assert_eq!(
        typed_response(
            LOGOUT,
            SUCCESS,
            None,
            Some(structure([item(TICKET_TYPE, Value::enumeration(1))])),
        )
        .unwrap_err(),
        LogoutError::MalformedResponsePayload
    );
}

#[test]
fn table_258_failure_reasons_are_preserved_without_a_success_payload() {
    let expected_names = [
        "Invalid Ticket",
        "Attestation Failed",
        "Attestation Required",
        "Feature Not Supported",
        "Invalid Field",
        "Invalid Message",
        "Operation Not Supported",
        "Permission Denied",
        "Response Too Large",
    ];
    for expected_name in expected_names {
        let (reason, _) = ResultReason::known_values()
            .iter()
            .find(|(_, name)| *name == expected_name)
            .expect("Table 258 reason is cataloged");
        let response = typed_response(LOGOUT, OPERATION_FAILED, Some(reason.raw()), None)
            .expect("Table 258 error preserves the common KMIP result");
        assert_eq!(response.result().reason(), Some(*reason));
    }
}

#[test]
fn unknown_table_258_reason_remains_open_and_lossless() {
    const UNKNOWN_REASON: u32 = 0xF123_4567;
    let response = typed_response(LOGOUT, OPERATION_FAILED, Some(UNKNOWN_REASON), None)
        .expect("an unknown Result Reason remains a valid open value");

    assert_eq!(
        response.result().reason(),
        Some(ResultReason::from_raw(UNKNOWN_REASON))
    );
}

#[test]
fn wrong_operation_is_rejected_with_payload_free_error() {
    assert_eq!(
        typed_response(LOGOUT - 1, SUCCESS, None, Some(Structure::new())).unwrap_err(),
        LogoutError::UnexpectedOperation
    );
}
