//! Fake-transport RNG Seed behavior derived from OASIS KMIP v2.1
//! §6.1.50, Tables 319–321. These tests are source-derived, not official
//! Test Case passes.
//!
//! Traceability: KMIPKIT-0041 FR-001–FR-008, SC-002–SC-004;
//! `KMIPKIT-ELEM-OP-C2S-RNG-SEED`,
//! `KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-SEED-00000026`,
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA`,
//! `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-10-DATA-LENGTH`, and
//! `KMIPKIT-REQ-SPEC-6.1.50-001`.

use kmipkit_protocol::{RngSeedRequest, SecretBytes};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::ValueView;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};

use crate::asynchronous_execution_test_support::{client_for, request_contains};
use crate::execute::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};

const RNG_SEED: u32 = 0x0000_0026;
const DATA: u32 = 0x0042_00C2;
const DATA_LENGTH: u32 = 0x0042_00D5;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const SEED_SENTINEL: &[u8] = b"KMIP_RNG_SEED_SECRET_SENTINEL\x00\xff";
const PENDING_CORRELATION: &[u8] = b"RNG_SEED_PENDING\x00\xff";

fn request_operation_and_data(request: &[u8]) -> Option<(u32, Vec<u8>)> {
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
            let seed = batch
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
                        .find(|field| field.tag().raw() == DATA)?
                        .with_value(|value| match value {
                            ValueView::ByteString(value) => Some(value.to_vec()),
                            _ => None,
                        })
                })?;
            Some((operation, seed))
        })
    })
}

#[test]
fn rng_seed_sends_secret_once_and_accepts_zero_bytes_used() {
    let response = asynchronous_response_bytes(
        RNG_SEED,
        0,
        None,
        None,
        Some(test_structure([test_item(
            DATA_LENGTH,
            kmipkit_ttlv::Value::integer(0),
        )])),
    );
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });
    let request = RngSeedRequest::new(SecretBytes::new(SEED_SENTINEL.to_vec()));
    assert!(!format!("{request:?}").contains("KMIP_RNG_SEED_SECRET_SENTINEL"));

    let item = client
        .rng_seed(request, &CodecLimits::defaults())
        .expect("the fake server returns success even though it used zero seed bytes");
    assert_eq!(item.outcome().operation(), ClientOperation::RngSeed);
    assert_eq!(item.outcome().result().status().raw(), 0);
    let ClientBatchOutcome::RngSeed(response) = item.outcome() else {
        panic!("RNG Seed returns its typed outcome");
    };
    assert_eq!(response.data_length(), Some(0));
    assert!(!format!("{:?}", item.outcome()).contains("KMIP_RNG_SEED_SECRET_SENTINEL"));
    assert!(
        !item
            .outcome()
            .to_string()
            .contains("KMIP_RNG_SEED_SECRET_SENTINEL")
    );
    assert_eq!(
        request_operation_and_data(captured.borrow().as_deref().expect("request was captured")),
        Some((RNG_SEED, SEED_SENTINEL.to_vec()))
    );
    assert!(request_contains(&captured, SEED_SENTINEL));
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn rng_seed_pending_is_explicit_and_does_not_poll_or_retry() {
    let response = asynchronous_response_bytes(
        RNG_SEED,
        2,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::RngSeed(
        RngSeedRequest::new(SecretBytes::new(SEED_SENTINEL.to_vec())),
    )))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the request explicitly permits a Pending response");
    let outcome = response.get(0).expect("one result is associated").outcome();
    let ClientBatchOutcome::Pending(pending) = outcome else {
        panic!("Pending remains explicit and resumable by the caller");
    };
    assert_eq!(pending.operation(), ClientOperation::RngSeed);
    assert_eq!(pending.result().status().raw(), 2);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn rng_seed_malformed_response_error_does_not_expose_seed_bytes() {
    let response = asynchronous_response_bytes(RNG_SEED, 0, None, None, Some(test_structure([])));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let error = client
        .rng_seed(
            RngSeedRequest::new(SecretBytes::new(SEED_SENTINEL.to_vec())),
            &CodecLimits::defaults(),
        )
        .expect_err("a successful Table 320 response requires Data Length");

    assert!(!error.to_string().contains("KMIP_RNG_SEED_SECRET_SENTINEL"));
    assert!(!format!("{error:?}").contains("KMIP_RNG_SEED_SECRET_SENTINEL"));
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn rng_seed_transport_failure_preserves_delivery_and_is_not_retried() {
    let (mut client, transport, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });

    let error = client
        .rng_seed(
            RngSeedRequest::new(SecretBytes::new(SEED_SENTINEL.to_vec())),
            &CodecLimits::defaults(),
        )
        .expect_err("the scripted transport fails after a partial write");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert!(!error.to_string().contains("KMIP_RNG_SEED_SECRET_SENTINEL"));
    assert!(!format!("{error:?}").contains("KMIP_RNG_SEED_SECRET_SENTINEL"));
    assert_eq!(transport.borrow().exchange_count(), 1);
}
