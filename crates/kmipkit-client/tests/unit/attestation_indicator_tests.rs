//! OASIS KMIP v2.1 §9.3, Table 402; derived capability-boundary test.
//!
//! Traceability: KMIPKIT-REQ-SPEC-9.3-001-001/-002; KMIPKIT-0008-FR-008;
//! KMIPKIT-0008-SC-004. This test asserts the non-secret header indicator and
//! confirms that execution adds no Authentication or Credential payload.

use kmipkit_protocol::PollRequest;
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Item, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{ClientBatch, ClientBatchItem, ClientRequest};
use crate::execute_test_support::asynchronous_response_bytes;
use crate::execute_test_support::{ResponseItemFixture, response_bytes};

const MESSAGE: u32 = 0x0042_0078;
const REQUEST_HEADER: u32 = 0x0042_0077;
const ATTESTATION_CAPABLE_INDICATOR: u32 = 0x0042_00D3;
const AUTHENTICATION: u32 = 0x0042_000C;
const CREDENTIAL: u32 = 0x0042_0023;
const POLL_OPERATION: u32 = 0x0000_001A;
const POLL_CORRELATION: &[u8] = b"ATTESTATION_POLL_CORRELATION";

#[test]
fn execute_advertises_attestation_without_authentication_or_credential_payload() {
    let response = response_bytes((2, 1), &[ResponseItemFixture::success(None)]);
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));

    client
        .execute(batch, &kmipkit_ttlv::codec::CodecLimits::defaults())
        .expect("a valid Discover Versions response completes the fake exchange");

    assert_eq!(fake.borrow().exchange_count(), 1);
    let captured = captured.borrow();
    let request = captured
        .as_ref()
        .expect("the fake transport captures the outbound request");
    let message = decode(request.as_slice()).expect("the captured request is valid TTLV");

    assert_eq!(message.tag().raw(), MESSAGE);
    let (indicator, authentication_present, credential_present) = request_indicators(&message);

    assert_eq!(indicator, Some(true));
    assert!(!authentication_present);
    assert!(
        !credential_present,
        "the request contains no Credential structure"
    );
}

#[test]
fn async_follow_up_request_advertises_attestation_capability() {
    let response =
        asynchronous_response_bytes(POLL_OPERATION, 2, None, Some(POLL_CORRELATION), None);
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let outcome = client
        .execute_poll(PollRequest::new(POLL_CORRELATION), &CodecLimits::defaults())
        .expect("a valid Pending Poll response completes the fake exchange");

    assert!(outcome.is_pending());
    assert_eq!(fake.borrow().exchange_count(), 1);
    let captured = captured.borrow();
    let request = captured
        .as_ref()
        .expect("the fake transport captures the outbound request");
    let message = decode(request.as_slice()).expect("the captured request is valid TTLV");

    let (indicator, authentication_present, credential_present) = request_indicators(&message);

    assert_eq!(indicator, Some(true));
    assert!(!authentication_present);
    assert!(!credential_present);
}

fn request_indicators(message: &Item) -> (Option<bool>, bool, bool) {
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return (None, false, false);
        };
        let Some(header) = message
            .children()
            .iter()
            .find(|field| field.tag().raw() == REQUEST_HEADER)
        else {
            return (None, false, false);
        };
        let (indicator, authentication_present) = header.with_value(|value| {
            let ValueView::Structure(header) = value else {
                return (None, false);
            };
            let indicator = header
                .children()
                .iter()
                .find(|field| field.tag().raw() == ATTESTATION_CAPABLE_INDICATOR)
                .and_then(|field| {
                    field.with_value(|value| match value {
                        ValueView::Boolean(value) => Some(*value),
                        _ => None,
                    })
                });
            let authentication_present = header
                .children()
                .iter()
                .any(|field| field.tag().raw() == AUTHENTICATION);
            (indicator, authentication_present)
        });
        let credential_present = message
            .children()
            .iter()
            .any(|field| item_tree_contains_tag(field, CREDENTIAL));
        (indicator, authentication_present, credential_present)
    })
}

fn item_tree_contains_tag(item: &kmipkit_ttlv::Item, target: u32) -> bool {
    if item.tag().raw() == target {
        return true;
    }

    item.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .any(|child| item_tree_contains_tag(child, target)),
        _ => false,
    })
}
