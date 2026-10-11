//! Login contracts from OASIS KMIP v2.1 §6.1.30, Tables 253–255, and
//! §7.39/Table 391. The fixture case is pinned as KMIPKIT-TEST-CN01-2-48.
//! These assertions do not claim that a KMIP server was run.

use crate::{LoginError, LoginRequest, LoginResponse, ResponseMessage, ResultReason};
use kmipkit_ttlv::{Structure, Value, ValueView};

const LOGIN: u32 = 0x0000_002D;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const LEASE_TIME: u32 = 0x0042_0049;
const REQUEST_COUNT: u32 = 0x0042_014C;
const USAGE_LIMITS: u32 = 0x0042_0095;
const TICKET: u32 = 0x0042_0149;
const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;
const SECRET_SENTINEL: &[u8] = b"login-ticket-secret-sentinel";
const LOGIN_FIXTURE: &str =
    include_str!("../../../../specification/oasis/kmip-2.1/fixtures/TC-LOGIN-1-21.xml");

use crate::async_operation_fixtures::{item, response_message, structure};

fn response_item(message: &ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one validated response batch item")
}

fn typed_response(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<LoginResponse, LoginError> {
    let message = response_message(LOGIN, status, reason, None, payload);
    LoginResponse::try_from_response_item(response_item(&message))
}

fn ticket_payload() -> Structure {
    structure([item(
        TICKET,
        Value::structure(structure([
            item(TICKET_TYPE, Value::enumeration(1)),
            item(TICKET_VALUE, Value::byte_string(SECRET_SENTINEL.to_vec())),
            item(UNKNOWN_VENDOR_TAG, Value::enumeration(0xF123_4567)),
        ])),
    )])
}

#[test]
fn empty_login_request_keeps_all_optional_table_253_fields_absent() {
    let payload = LoginRequest::new()
        .into_ttlv_payload()
        .expect("Table 253 permits an empty Login request");
    assert!(payload.view().children().is_empty());
}

#[test]
fn optional_request_fields_keep_table_253_order_and_both_lease_forms() {
    let with_interval = LoginRequest::new()
        .with_lease_time_interval(0)
        .with_request_count(3)
        .with_usage_limits(structure([item(0x0042_00C2, Value::integer(2))]))
        .into_ttlv_payload()
        .expect("fixture Table 253 fields encode in order");
    let with_date_time = LoginRequest::new()
        .with_lease_time_date_time(1_700_000_000)
        .into_ttlv_payload()
        .expect("the alternate Lease Time form is preserved");

    assert_eq!(
        with_interval
            .view()
            .children()
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [LEASE_TIME, REQUEST_COUNT, USAGE_LIMITS]
    );
    with_interval.view().children()[0].with_value(|value| {
        assert!(matches!(value, ValueView::Interval(0)));
    });
    with_interval.view().children()[1].with_value(|value| {
        assert!(matches!(value, ValueView::Integer(3)));
    });
    with_interval.view().children()[2].with_value(|value| {
        assert!(matches!(value, ValueView::Structure(_)));
    });
    assert_eq!(with_date_time.view().children()[0].tag().raw(), LEASE_TIME);
    with_date_time.view().children()[0].with_value(|value| {
        assert!(matches!(value, ValueView::DateTime(1_700_000_000)));
    });
}

#[test]
fn table_254_success_requires_one_secret_safe_ticket_and_preserves_extensions() {
    let response = typed_response(SUCCESS, None, Some(ticket_payload()))
        .expect("Table 254 success returns its required Ticket");
    let ticket = response.ticket().expect("successful Login exposes Ticket");
    assert_eq!(ticket.ticket_type().raw(), 1);
    ticket.with_ticket_value(|value| assert_eq!(value, SECRET_SENTINEL));
    let tags = ticket.with_ttlv(|tree| {
        tree.children()
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>()
    });
    assert_eq!(tags, [TICKET_TYPE, TICKET_VALUE, UNKNOWN_VENDOR_TAG]);
    assert!(!format!("{response:?}").contains("login-ticket-secret-sentinel"));
}

#[test]
fn table_255_failure_reasons_are_preserved_without_a_ticket() {
    let expected_names = [
        "Invalid Field",
        "Attestation Failed",
        "Attestation Required",
        "Feature Not Supported",
        "Invalid Message",
        "Operation Not Supported",
        "Permission Denied",
        "Response Too Large",
    ];
    for expected_name in expected_names {
        let (reason, _) = ResultReason::known_values()
            .iter()
            .find(|(_, name)| *name == expected_name)
            .expect("Table 255 reason is cataloged");
        let response = typed_response(OPERATION_FAILED, Some(reason.raw()), None)
            .expect("Table 255 error preserves the common operation result");
        assert_eq!(response.result().reason(), Some(*reason));
        assert!(response.ticket().is_none());
    }
}

#[test]
fn malformed_success_ticket_is_rejected_without_disclosing_secret_bytes() {
    let malformed = structure([item(
        TICKET,
        Value::structure(structure([item(TICKET_TYPE, Value::enumeration(1))])),
    )]);
    let error = typed_response(SUCCESS, None, Some(malformed))
        .expect_err("Table 254 requires a structurally valid Ticket");
    assert!(!error.to_string().contains("login-ticket-secret-sentinel"));
}

#[test]
fn successful_login_rejects_duplicate_ticket_fields() {
    let ticket = || {
        item(
            TICKET,
            Value::structure(structure([
                item(TICKET_TYPE, Value::enumeration(1)),
                item(TICKET_VALUE, Value::byte_string(SECRET_SENTINEL.to_vec())),
            ])),
        )
    };
    let error = typed_response(SUCCESS, None, Some(structure([ticket(), ticket()])))
        .expect_err("Table 254 has one Ticket result field");

    assert_eq!(error, LoginError::MalformedSuccessPayload);
    assert!(!error.to_string().contains("login-ticket-secret-sentinel"));
}

#[test]
fn pinned_cn01_2_48_fixture_matches_the_typed_request_and_response_contract() {
    let document = roxmltree::Document::parse(LOGIN_FIXTURE)
        .expect("the checked-in OASIS Login fixture is valid XML");
    let login_request = document
        .descendants()
        .find(|node| {
            node.has_tag_name("RequestMessage")
                && node.descendants().any(|field| {
                    field.has_tag_name("Operation") && field.attribute("value") == Some("Login")
                })
        })
        .expect("TC-LOGIN-1-21 includes the Login request");
    let request_payload = login_request
        .descendants()
        .find(|node| node.has_tag_name("RequestPayload"))
        .expect("fixture Login request has Request Payload");
    let request_fields = request_payload
        .children()
        .filter(roxmltree::Node::is_element)
        .collect::<Vec<_>>();
    assert_eq!(
        request_fields
            .iter()
            .map(|field| field.tag_name().name())
            .collect::<Vec<_>>(),
        ["LeaseTime", "RequestCount"]
    );
    assert_eq!(request_fields[0].attribute("type"), Some("Interval"));
    assert_eq!(request_fields[0].attribute("value"), Some("0"));
    assert_eq!(request_fields[1].attribute("type"), Some("Integer"));
    assert_eq!(request_fields[1].attribute("value"), Some("3"));

    let typed_payload = LoginRequest::new()
        .with_lease_time_interval(0)
        .with_request_count(3)
        .into_ttlv_payload()
        .expect("typed request encodes the pinned fixture values");
    assert_eq!(
        typed_payload
            .view()
            .children()
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [LEASE_TIME, REQUEST_COUNT]
    );

    let login_response = document
        .descendants()
        .find(|node| {
            node.has_tag_name("ResponseMessage")
                && node.descendants().any(|field| {
                    field.has_tag_name("Operation") && field.attribute("value") == Some("Login")
                })
        })
        .expect("TC-LOGIN-1-21 includes the Login response");
    let fixture_ticket = login_response
        .descendants()
        .find(|node| node.has_tag_name("Ticket"))
        .expect("fixture Login success returns Ticket");
    let fixture_type = fixture_ticket
        .children()
        .find(|node| node.has_tag_name("TicketType"))
        .expect("fixture Ticket has Ticket Type");
    let fixture_value = fixture_ticket
        .children()
        .find(|node| node.has_tag_name("TicketValue"))
        .expect("fixture Ticket has Ticket Value");
    assert_eq!(fixture_type.attribute("value"), Some("Login"));
    assert_eq!(fixture_value.attribute("value"), Some("$TICKET_VALUE"));
    let response = typed_response(SUCCESS, None, Some(ticket_payload()))
        .expect("typed response contract is compatible with the fixture Ticket");
    assert_eq!(response.ticket().expect("Ticket").ticket_type().raw(), 1);
}
