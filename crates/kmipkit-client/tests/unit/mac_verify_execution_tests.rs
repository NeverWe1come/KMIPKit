//! Source-derived MAC Verify client contracts from OASIS KMIP v2.1
//! §6.1.33, Tables 262–264. These are fake-transport tests, not official
//! Test Case executions.
//!
//! Traceability: FR-001, FR-005–FR-011, FR-013, FR-015, and
//! `KMIPKIT-ELEM-OP-C2S-MAC-VERIFY`.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;

use kmipkit_protocol::{MacVerifyRequest, SecretBytes};
use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;
use zeroize::Zeroizing;

use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientRequest};

const MAC_VERIFY_OPERATION: u32 = 0x0000_0024;
const SUCCESS: u32 = 0;
const VALIDITY_INDICATOR: u32 = 0x0042_0128;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const BATCH_ITEM: u32 = 0x0042_000F;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;

#[derive(Default)]
struct ExchangeState {
    requests: Vec<Zeroizing<Vec<u8>>>,
    responses: VecDeque<Vec<u8>>,
    calls: usize,
    failure: Option<RequestDeliveryState>,
}

struct MacVerifyTransport(Rc<RefCell<ExchangeState>>);

impl Transport for MacVerifyTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        let mut state = self.0.borrow_mut();
        state.calls += 1;
        state.requests.push(Zeroizing::new(request.to_vec()));
        if let Some(delivery_state) = state.failure {
            return Err(TransportError::new(
                delivery_state,
                TransportCauseCategory::Io,
                io::Error::other("test transport failure"),
            ));
        }
        let response = state.responses.pop_front().ok_or_else(|| {
            TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("unexpected additional exchange"),
            )
        })?;
        Ok(TransportResponse::new(response))
    }
}

fn client(response: Vec<u8>) -> (Client, Rc<RefCell<ExchangeState>>) {
    let state = Rc::new(RefCell::new(ExchangeState {
        responses: VecDeque::from([response]),
        ..ExchangeState::default()
    }));
    (
        Client::for_test(MacVerifyTransport(Rc::clone(&state))),
        state,
    )
}

fn request() -> ClientRequest {
    ClientRequest::mac_verify(
        MacVerifyRequest::new().with_mac_data(SecretBytes::new(b"opaque-mac".to_vec())),
    )
}

fn response(indicator: u32) -> Vec<u8> {
    let payload = test_structure([
        test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("verification-key".to_owned()),
        ),
        test_item(VALIDITY_INDICATOR, Value::enumeration(indicator)),
    ]);
    asynchronous_response_bytes(MAC_VERIFY_OPERATION, SUCCESS, None, None, Some(payload))
}

fn operation_failure_response() -> Vec<u8> {
    asynchronous_response_bytes(MAC_VERIFY_OPERATION, 1, Some(0x0000_0100), None, None)
}

fn response_for_id(id: &[u8], indicator: Option<u32>) -> kmipkit_ttlv::Item {
    let mut payload = vec![test_item(
        UNIQUE_IDENTIFIER,
        Value::text_string("verification-key".to_owned()),
    )];
    if let Some(indicator) = indicator {
        payload.push(test_item(VALIDITY_INDICATOR, Value::enumeration(indicator)));
    }
    if indicator.is_none() {
        payload.push(test_item(
            0x0042_00D6,
            Value::byte_string(b"next-part".to_vec()),
        ));
    }
    test_item(
        BATCH_ITEM,
        Value::structure(test_structure([
            test_item(0x0042_005C, Value::enumeration(MAC_VERIFY_OPERATION)),
            test_item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id.to_vec())),
            test_item(RESULT_STATUS, Value::enumeration(SUCCESS)),
            test_item(RESPONSE_PAYLOAD, Value::structure(test_structure(payload))),
        ])),
    )
}

fn reordered_batch_response_bytes() -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(2)),
    ]);
    let tree = test_structure([
        test_item(RESPONSE_HEADER, Value::structure(header)),
        response_for_id(b"multipart", None),
        response_for_id(b"single", Some(1)),
    ]);
    crate::execute::encode_message_for_test(tree, &CodecLimits::defaults())
        .expect("the reordered verification responses form a valid message")
}

#[test]
fn invalid_and_unknown_indicators_remain_operation_results_with_one_exchange() {
    for indicator in [2, 3] {
        let (mut client, state) = client(response(indicator));
        let response = client
            .execute(
                ClientBatch::new(ClientBatchItem::new(request())),
                &CodecLimits::defaults(),
            )
            .expect("Invalid and Unknown are server operation results, not client errors");
        let outcome = response
            .items
            .first()
            .expect("one MAC Verify result is associated")
            .outcome();
        let typed = outcome
            .mac_verify_response()
            .expect("the operation remains a typed MAC Verify outcome");

        assert_eq!(outcome.operation(), crate::ClientOperation::MacVerify);
        assert_eq!(
            typed
                .validity_indicator()
                .map(kmipkit_protocol::ValidityIndicator::raw),
            Some(indicator)
        );
        assert_eq!(
            state.borrow().calls,
            1,
            "one explicit Verify call exchanges once"
        );
        assert_eq!(state.borrow().requests.len(), 1);
    }
}

#[test]
fn server_operation_failure_remains_typed_and_is_not_retried() {
    let (mut client, state) = client(operation_failure_response());
    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect("server Operation Failed remains a typed operation result");
    let outcome = response
        .items
        .first()
        .expect("one failed result is associated")
        .outcome();

    assert!(matches!(outcome, ClientBatchOutcome::MacVerify(_)));
    assert_eq!(outcome.result().status().raw(), 1);
    assert_eq!(
        outcome
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(0x0000_0100)
    );
    assert_eq!(state.borrow().calls, 1);
}

#[test]
fn possibly_sent_verify_transport_failure_preserves_delivery_and_is_not_retried() {
    let state = Rc::new(RefCell::new(ExchangeState {
        failure: Some(RequestDeliveryState::PossiblySent),
        ..ExchangeState::default()
    }));
    let mut client = Client::for_test(MacVerifyTransport(Rc::clone(&state)));
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request())),
            &CodecLimits::defaults(),
        )
        .expect_err("transport failure keeps the request delivery evidence");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(state.borrow().calls, 1, "MAC Verify is never retried");
    assert_eq!(state.borrow().requests.len(), 1);
}

#[test]
fn reordered_batch_responses_keep_each_requests_multipart_indicator_context() {
    let (mut client, state) = client(reordered_batch_response_bytes());
    let multipart = ClientRequest::mac_verify(
        MacVerifyRequest::new()
            .with_correlation_value(SecretBytes::new(b"previous-part".to_vec()))
            .with_init_indicator(false)
            .with_final_indicator(false),
    );
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(request()).with_unique_batch_item_id(b"single".to_vec()),
        ClientBatchItem::new(multipart).with_unique_batch_item_id(b"multipart".to_vec()),
    ]);
    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("response part context follows the matched request IDs");

    let single = response.items[0]
        .outcome()
        .mac_verify_response()
        .expect("the first request is the single-part operation");
    let multipart = response.items[1]
        .outcome()
        .mac_verify_response()
        .expect("the second request is the non-final multipart operation");
    assert_eq!(
        single
            .validity_indicator()
            .map(kmipkit_protocol::ValidityIndicator::raw),
        Some(1)
    );
    assert_eq!(multipart.validity_indicator(), None);
    assert_eq!(
        state.borrow().calls,
        1,
        "a verification batch uses one exchange"
    );
}
