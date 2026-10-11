//! Project-derived fake-transport checks for OASIS KMIP Specification v2.1
//! §6.1.22, Tables 229–231, and Constraints §7.7.
//! Traceability: KMIPKIT-025-T229-REQUEST, T230-RESPONSE, T231-ERROR,
//! KMIPKIT-025-COMMON-RESULT, and KMIPKIT-025-COMMON-DELIVERY.
//! These are not official OASIS Test Cases.

use kmipkit_protocol::{GetConstraintsResponse, ResultReason};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{ItemType, Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::encode_message_for_test;
use crate::execute_test_support::{test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const GET_CONSTRAINTS: u32 = 0x0000_0038;
const CONSTRAINTS: u32 = 0x0042_0168;
const VENDOR_MEMBER: u32 = 0x0054_1234;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const BATCH_ID: &[u8] = b"get-constraints-batch";
const CORRELATION: &[u8] = &[0xA5, 0x00, 0x5A, 0xFF];
const SERVER_MESSAGE: &str = "opaque-server-message-get-constraints";

fn response_bytes(
    status: u32,
    reason: Option<u32>,
    message: Option<&str>,
    correlation: Option<&[u8]>,
    batch_id: Option<&[u8]>,
    payload: Option<Structure>,
) -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut fields = vec![test_item(OPERATION, Value::enumeration(GET_CONSTRAINTS))];
    if let Some(id) = batch_id {
        fields.push(test_item(
            UNIQUE_BATCH_ITEM_ID,
            Value::byte_string(id.to_vec()),
        ));
    }
    fields.push(test_item(RESULT_STATUS, Value::enumeration(status)));
    if let Some(reason) = reason {
        fields.push(test_item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(message) = message {
        fields.push(test_item(
            RESULT_MESSAGE,
            Value::text_string(message.to_owned()),
        ));
    }
    if let Some(correlation) = correlation {
        fields.push(test_item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(correlation.to_vec()),
        ));
    }
    if let Some(payload) = payload {
        fields.push(test_item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(test_structure(fields))),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("project fixture is valid KMIP TTLV")
}

fn success_payload() -> Structure {
    test_structure([test_item(
        CONSTRAINTS,
        Value::structure(test_structure([
            test_item(VENDOR_MEMBER, Value::integer(1)),
            test_item(VENDOR_MEMBER, Value::integer(2)),
        ])),
    )])
}

fn request_batch() -> ClientBatch {
    ClientBatch::new(ClientBatchItem::new(ClientRequest::get_constraints()))
}

fn completed(outcome: &ClientBatchOutcome) -> &GetConstraintsResponse {
    assert_eq!(outcome.operation(), ClientOperation::GetConstraints);
    let ClientBatchOutcome::GetConstraints(response) = outcome else {
        panic!("Get Constraints returns its typed response");
    };
    assert!(outcome.response().get_constraints().is_some());
    response
}

#[test]
fn table_229_empty_request_and_table_230_constraints_use_one_exchange() {
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response: response_bytes(0, None, None, None, None, Some(success_payload())),
        request_write_chunks: Vec::new(),
    });
    let result = client
        .get_constraints(&CodecLimits::defaults())
        .expect("the one-item convenience path returns Table 230 success");

    assert_eq!(fake.borrow().exchange_count(), 1);
    let wire = captured.borrow();
    let decoded = decode(wire.as_ref().expect("one request was captured"))
        .expect("outbound request is valid TTLV");
    decoded.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("request message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .find(|item| item.tag().raw() == BATCH_ITEM)
            .expect("request has one Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("Batch Item is a Structure");
            };
            let payload = batch
                .children()
                .iter()
                .find(|item| item.tag().raw() == REQUEST_PAYLOAD)
                .expect("Table 229 has a Request Payload");
            assert_eq!(payload.item_type(), ItemType::Structure);
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                assert!(payload.children().is_empty(), "Table 229 is empty");
            });
        });
    });

    let response = completed(result.outcome());
    assert_eq!(response.result().status().raw(), 0);
    let constraints = response
        .constraints()
        .expect("Table 230 returns Constraints");
    let view = constraints.view();
    let members = view.children();
    assert_eq!(members.len(), 2);
    assert_eq!(members[0].tag().raw(), VENDOR_MEMBER);
    assert_eq!(members[1].tag().raw(), VENDOR_MEMBER);
    assert!(members[0].with_value(|value| matches!(value, ValueView::Integer(1))));
    assert!(members[1].with_value(|value| matches!(value, ValueView::Integer(2))));
}

#[test]
fn table_231_failure_reasons_and_unknown_reason_are_preserved_without_retry() {
    // Table 231 repeats Invalid Field. All distinct alternatives are exercised.
    for reason in [0x07, 0x30, 0x15, 0x14, 0x08, 0x04, 0x05, 0x0C, 0x02, 0x54FF] {
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response: response_bytes(1, Some(reason), Some(SERVER_MESSAGE), None, None, None),
            request_write_chunks: Vec::new(),
        });
        let result = client
            .execute(request_batch(), &CodecLimits::defaults())
            .expect("Operation Failed remains a typed result");
        let outcome = result.get(0).expect("one response").outcome();
        let response = completed(outcome);
        assert_eq!(fake.borrow().exchange_count(), 1, "reason {reason:#x}");
        assert_eq!(response.result().status().raw(), 1);
        assert_eq!(
            response.result().reason(),
            Some(ResultReason::from_raw(reason))
        );
        assert_eq!(
            response
                .result()
                .message()
                .map(kmipkit_protocol::ResultMessage::as_str),
            Some(SERVER_MESSAGE)
        );
        assert!(response.constraints().is_none());
        assert!(!format!("{outcome:?}").contains(SERVER_MESSAGE));
        assert!(!outcome.to_string().contains(SERVER_MESSAGE));
    }
}

#[test]
fn malformed_success_is_response_started_and_never_retried() {
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: response_bytes(0, None, None, None, None, Some(Structure::new())),
        request_write_chunks: Vec::new(),
    });
    let error = client
        .execute(request_batch(), &CodecLimits::defaults())
        .expect_err("Table 230 success requires Constraints");
    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn pending_keeps_exact_correlation_and_does_not_poll() {
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: response_bytes(
            2,
            None,
            None,
            Some(CORRELATION),
            Some(BATCH_ID),
            Some(Structure::new()),
        ),
        request_write_chunks: Vec::new(),
    });
    let result = client
        .execute(
            ClientBatch::new(
                ClientBatchItem::new(ClientRequest::get_constraints())
                    .with_unique_batch_item_id(BATCH_ID.to_vec()),
            )
            .with_asynchronous_indicator(1),
            &CodecLimits::defaults(),
        )
        .expect("Pending remains an explicit outcome");
    assert_eq!(fake.borrow().exchange_count(), 1);
    let response = result.get(0).expect("one response");
    assert_eq!(response.unique_batch_item_id(), Some(BATCH_ID));
    assert_eq!(
        response.outcome().asynchronous_correlation_value(),
        Some(CORRELATION)
    );
    let ClientBatchOutcome::Pending(pending) = response.outcome() else {
        panic!("Pending is not automatically polled");
    };
    assert_eq!(pending.operation(), ClientOperation::GetConstraints);
    assert_eq!(pending.result().status().raw(), 2);
    assert!(response.outcome().response().get_constraints().is_some());
    assert!(!format!("{:?}", response.outcome()).contains("A5005AFF"));
}

#[test]
fn local_encoding_limit_reports_not_sent_without_transport() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 1 });
    let impossible =
        CodecLimits::new(0, 64, 100_000).expect("a zero-byte per-call limit is supported");
    let error = client
        .execute(request_batch(), &impossible)
        .expect_err("the request cannot fit within a zero-byte limit");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn partial_write_keeps_possible_delivery_without_retry() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = client
        .execute(request_batch(), &CodecLimits::defaults())
        .expect_err("partial request write fails safely");
    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
