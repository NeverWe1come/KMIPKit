#![cfg(test)]

//! Fake-transport Revoke checks from OASIS KMIP v2.1 §6.1.44, Tables 295–297,
//! and §11.48, Table 481. Traceability: `KMIPKIT-REQ-SPEC-6.1.44-001`,
//! `-003-001`, and `-003-002`. These are derived tests, not official fixture passes.

use kmipkit_protocol::{
    AttributeSet, ObjectType, RegisterRequest, RevocationReason, RevokeRequest, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const REVOKE_OPERATION: u32 = 0x0000_0013;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const REVOCATION_REASON: u32 = 0x0042_0081;
const REVOCATION_MESSAGE: u32 = 0x0042_0080;
const REVOCATION_REASON_CODE: u32 = 0x0042_0082;
const COMPROMISE_OCCURRENCE_DATE: u32 = 0x0042_0021;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_LENGTH: u32 = 0x0042_00AD;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const OBJECT_NOT_FOUND: u32 = 0x0000_0037;
const KEY_COMPROMISE: u32 = 2;
const CA_COMPROMISE: u32 = 3;
const PENDING_CORRELATION: &[u8] = b"REVOKE_PENDING_CORRELATION\xFF";

fn revoke_request(reason: u32, compromise_date: Option<i64>) -> RevokeRequest {
    let request = RevokeRequest::new(RevocationReason::from_raw(reason))
        .with_unique_identifier(UniqueIdentifier::TextString("revoke-target".to_owned()));
    match compromise_date {
        Some(date) => request.with_compromise_occurrence_date(date),
        None => request,
    }
}

fn request_fields(bytes: &[u8], batch_index: usize) -> (Option<u32>, Vec<(u32, Option<i64>)>) {
    let message = decode(bytes).expect("captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("KMIP request message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM)
            .nth(batch_index)
            .expect("request contains the selected Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("Batch Item is a Structure");
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)
                .and_then(|field| {
                    field.with_value(|value| match value {
                        ValueView::Enumeration(raw) => Some(*raw),
                        _ => None,
                    })
                });
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("Batch Item contains Request Payload");
            let fields = payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                payload
                    .children()
                    .iter()
                    .map(|field| {
                        let raw_value = field.with_value(|value| match value {
                            ValueView::DateTime(value) => Some(*value),
                            ValueView::Enumeration(value) => Some(i64::from(*value)),
                            _ => None,
                        });
                        (field.tag().raw(), raw_value)
                    })
                    .collect()
            });
            (operation, fields)
        })
    })
}

fn reason_fields(bytes: &[u8], batch_index: usize) -> (Vec<u32>, Option<u32>, Option<String>) {
    let message = decode(bytes).expect("captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("KMIP request message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM)
            .nth(batch_index)
            .expect("request contains the selected Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("Batch Item is a Structure");
            };
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("Batch Item contains Request Payload");
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                let reason = payload
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == REVOCATION_REASON)
                    .expect("Table 295 contains Revocation Reason");
                reason.with_value(|value| {
                    let ValueView::Structure(reason) = value else {
                        panic!("§4.47 Revocation Reason is a Structure");
                    };
                    let tags = reason
                        .children()
                        .iter()
                        .map(|field| field.tag().raw())
                        .collect();
                    let code = reason
                        .children()
                        .iter()
                        .find(|field| field.tag().raw() == REVOCATION_REASON_CODE)
                        .and_then(|field| {
                            field.with_value(|value| match value {
                                ValueView::Enumeration(value) => Some(*value),
                                _ => None,
                            })
                        });
                    let message = reason
                        .children()
                        .iter()
                        .find(|field| field.tag().raw() == REVOCATION_MESSAGE)
                        .and_then(|field| {
                            field.with_value(|value| match value {
                                ValueView::TextString(value) => Some(value.to_owned()),
                                _ => None,
                            })
                        });
                    (tags, code, message)
                })
            })
        })
    })
}

#[test]
fn revoke_sends_table_295_and_exposes_table_296_identifier_once() {
    crate::execute_test_support::assert_lifecycle_request_debug(
        &crate::ClientRequest::revoke(
            revoke_request(KEY_COMPROMISE, Some(1_700_000_000))
                .with_revocation_message("debug audit explanation".to_owned()),
        ),
        "Revoke",
    );
    let response = asynchronous_response_bytes(
        REVOKE_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("revoked-object".to_owned()),
        )])),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![4, 11],
    });

    let item = client
        .revoke(
            revoke_request(KEY_COMPROMISE, Some(1_700_000_000))
                .with_revocation_message("audit explanation".to_owned()),
            &CodecLimits::defaults(),
        )
        .expect("valid Revoke request receives typed Table 296 success");
    let outcome = item.outcome();
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        outcome,
        ClientOperation::Revoke,
        "Revoke(",
        "Revoke(",
    );
    let typed = outcome
        .response()
        .revoke()
        .expect("result retains its typed Revoke response");
    assert_eq!(outcome.operation(), ClientOperation::Revoke);
    assert_eq!(outcome.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.unique_identifier(),
        Some(&UniqueIdentifier::TextString("revoked-object".to_owned()))
    );
    let captured = captured.borrow();
    let request = captured
        .as_deref()
        .expect("fake transport captured request");
    assert_eq!(
        request_fields(request, 0),
        (
            Some(REVOKE_OPERATION),
            vec![
                (UNIQUE_IDENTIFIER, None),
                (REVOCATION_REASON, None),
                (COMPROMISE_OCCURRENCE_DATE, Some(1_700_000_000)),
            ],
        )
    );
    assert_eq!(
        reason_fields(request, 0),
        (
            vec![REVOCATION_REASON_CODE, REVOCATION_MESSAGE],
            Some(KEY_COMPROMISE),
            Some("audit explanation".to_owned())
        )
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn revoke_failure_preserves_table_297_reason_without_retry() {
    let response = asynchronous_response_bytes(
        REVOKE_OPERATION,
        OPERATION_FAILED,
        Some(OBJECT_NOT_FOUND),
        None,
        None,
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let item = client
        .revoke(revoke_request(1, None), &CodecLimits::defaults())
        .expect("Table 297 Operation Failed is preserved as a KMIP result");
    assert_eq!(item.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        item.outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(OBJECT_NOT_FOUND)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn revoke_pending_preserves_correlation_without_implicit_poll() {
    let response = asynchronous_response_bytes(
        REVOKE_OPERATION,
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Revoke(revoke_request(
        CA_COMPROMISE,
        None,
    ))))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("caller-enabled typed batch preserves Revoke Pending");
    let item = response.get(0).expect("one Revoke result is associated");
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        item.outcome(),
        ClientOperation::Revoke,
        "Revoke(",
        "Pending(",
    );
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("server Operation Pending remains Pending");
    };
    assert_eq!(pending.operation(), ClientOperation::Revoke);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn revoke_rejects_prohibited_date_before_transport_exchange() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .revoke(
            revoke_request(1, Some(1_700_000_000)),
            &CodecLimits::defaults(),
        )
        .expect_err("§6.1.44 prohibits compromise date for an unrelated reason");
    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn revoke_can_consume_a_preceding_register_id_placeholder() {
    let (mut client, fake, captured) = client_for(ExchangeScript::FailBeforeWrite);
    let register = RegisterRequest::new(
        ObjectType::from_raw(1),
        AttributeSet::try_new([
            test_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(3)),
            test_item(CERTIFICATE_LENGTH, Value::integer(512)),
            test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(4)),
            test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(2048)),
        ])
        .expect("certificate length fixture is valid"),
        test_item(CERTIFICATE, Value::structure(test_structure([]))),
    );
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Register(register))
            .with_unique_batch_item_id(b"register-item".to_vec()),
        ClientBatchItem::new(ClientRequest::Revoke(RevokeRequest::new(
            RevocationReason::from_raw(KEY_COMPROMISE),
        )))
        .with_unique_batch_item_id(b"revoke-item".to_vec()),
    ]);
    let error = client
        .execute(batch, &CodecLimits::defaults())
        .expect_err("fake transport stops before write after local batch validation");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let captured = captured.borrow();
    let request = captured.as_deref().expect("request reaches fake transport");
    assert_eq!(
        request_fields(request, 1),
        (Some(REVOKE_OPERATION), vec![(REVOCATION_REASON, None)],),
        "§4.1.2 and §6.1.44 allow Revoke to consume Register's ID Placeholder"
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn revoke_partial_write_failure_preserves_delivery_state_without_retry() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = client
        .revoke(revoke_request(1, None), &CodecLimits::defaults())
        .expect_err("fake transport fails after a partial request write");
    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
