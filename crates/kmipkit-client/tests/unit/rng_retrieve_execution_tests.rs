//! Fake-transport RNG Retrieve behavior derived from OASIS KMIP v2.1
//! §6.1.49, Tables 316–318. These tests are source-derived, not official
//! Test Case passes.
//!
//! Traceability: KMIPKIT-0041 FR-001–FR-008, SC-001, SC-003;
//! `KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE`,
//! `KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-RETRIEVE-00000025`,
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-10-DATA-LENGTH`, and
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA`.

use kmipkit_protocol::RngRetrieveRequest;
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::ValueView;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};

const RNG_RETRIEVE: u32 = 0x0000_0025;
const DATA_LENGTH: u32 = 0x0042_00D5;
const DATA: u32 = 0x0042_00C2;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const PENDING_CORRELATION: &[u8] = b"RNG_RETRIEVE_PENDING\x00\xff";
const OUTPUT_SENTINEL: &[u8] = b"KMIP_RNG_RETRIEVE_SECRET_SENTINEL";

fn output_length() -> i32 {
    i32::try_from(OUTPUT_SENTINEL.len()).expect("test output length fits i32")
}

fn request_operation_and_length(request: &[u8]) -> Option<(u32, i32)> {
    let decoded = decode_with_limits(request, &CodecLimits::defaults()).ok()?;
    decoded.with_value(|value| {
        let ValueView::Structure(root) = value else {
            return None;
        };
        let batch = root
            .children()
            .iter()
            .find(|field| field.tag().raw() == BATCH_ITEM)?;
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                return None;
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)?
                .with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                })?;
            let length = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)?
                .with_value(|value| {
                    let ValueView::Structure(payload) = value else {
                        return None;
                    };
                    payload
                        .children()
                        .iter()
                        .find(|field| field.tag().raw() == DATA_LENGTH)?
                        .with_value(|value| match value {
                            ValueView::Integer(value) => Some(*value),
                            _ => None,
                        })
                })?;
            Some((operation, length))
        })
    })
}

#[test]
fn rng_retrieve_sends_required_length_once_and_returns_redacted_secret_output() {
    let response = asynchronous_response_bytes(
        RNG_RETRIEVE,
        0,
        None,
        None,
        Some(test_structure([test_item(
            DATA,
            kmipkit_ttlv::Value::byte_string(OUTPUT_SENTINEL.to_vec()),
        )])),
    );
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });

    let item = client
        .rng_retrieve(
            RngRetrieveRequest::new(output_length()),
            &CodecLimits::defaults(),
        )
        .expect("the fake server returns a valid RNG Retrieve success response");
    assert_eq!(item.outcome().operation(), ClientOperation::RngRetrieve);
    assert_eq!(item.outcome().result().status().raw(), 0);
    let ClientBatchOutcome::RngRetrieve(response) = item.outcome() else {
        panic!("RNG Retrieve returns its typed outcome");
    };
    let output = response.data().expect("Table 317 returns required Data");
    assert!(output.with_bytes(|bytes| bytes == OUTPUT_SENTINEL));
    assert!(!format!("{:?}", item.outcome()).contains("KMIP_RNG_RETRIEVE_SECRET_SENTINEL"));
    assert!(
        !item
            .outcome()
            .to_string()
            .contains("KMIP_RNG_RETRIEVE_SECRET_SENTINEL")
    );
    assert_eq!(
        request_operation_and_length(captured.borrow().as_deref().expect("request was captured")),
        Some((RNG_RETRIEVE, output_length()))
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn rng_retrieve_pending_is_explicit_and_does_not_poll_or_retry() {
    let response = asynchronous_response_bytes(
        RNG_RETRIEVE,
        2,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::RngRetrieve(
        RngRetrieveRequest::new(32),
    )))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the request explicitly permits a Pending response");
    let outcome = response.get(0).expect("one result is associated").outcome();
    let ClientBatchOutcome::Pending(pending) = outcome else {
        panic!("Pending remains explicit and resumable by the caller");
    };
    assert_eq!(pending.operation(), ClientOperation::RngRetrieve);
    assert_eq!(pending.result().status().raw(), 2);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn rng_retrieve_malformed_secret_response_is_not_exposed_in_errors() {
    let response = asynchronous_response_bytes(
        RNG_RETRIEVE,
        0,
        None,
        None,
        Some(test_structure([
            test_item(
                DATA,
                kmipkit_ttlv::Value::byte_string(OUTPUT_SENTINEL.to_vec()),
            ),
            test_item(DATA, kmipkit_ttlv::Value::byte_string(vec![1, 2, 3])),
        ])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let error = client
        .rng_retrieve(RngRetrieveRequest::new(16), &CodecLimits::defaults())
        .expect_err("duplicate Table 317 Data is rejected");

    assert!(
        !error
            .to_string()
            .contains("KMIP_RNG_RETRIEVE_SECRET_SENTINEL")
    );
    assert!(!format!("{error:?}").contains("KMIP_RNG_RETRIEVE_SECRET_SENTINEL"));
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn rng_retrieve_transport_failure_preserves_delivery_and_is_not_retried() {
    let (mut client, transport, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });

    let error = client
        .rng_retrieve(RngRetrieveRequest::new(16), &CodecLimits::defaults())
        .expect_err("the scripted transport fails after a partial write");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert!(
        !error
            .to_string()
            .contains("KMIP_RNG_RETRIEVE_SECRET_SENTINEL")
    );
    assert!(!format!("{error:?}").contains("KMIP_RNG_RETRIEVE_SECRET_SENTINEL"));
    assert_eq!(transport.borrow().exchange_count(), 1);
}
