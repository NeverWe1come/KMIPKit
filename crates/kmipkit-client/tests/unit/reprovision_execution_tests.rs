#![cfg(test)]

//! Fake-transport Re-Provision contracts from OASIS KMIP v2.1 §6.1.48,
//! Tables 313–315. Stable requirements: `KMIPKIT-REQ-SPEC-6.1.48-001`, `-004`,
//! and `-006`. `KMIPKIT-DISC-040` approves the three credential forms and the
//! Table 315 caption interpretation. These are table-derived tests, not
//! official OASIS fixture runs.

use kmipkit_protocol::{ReProvisionRequest, UniqueIdentifier};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const REPROVISION_OPERATION: u32 = 0x0000_0035;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const CERTIFICATE_REQUEST: u32 = 0x0042_0018;
const CERTIFICATE: u32 = 0x0042_0013;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const NESTED_VALUE: u32 = 0x0054_1234;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const PERMISSION_DENIED: u32 = 0x0000_0005;
const PENDING_CORRELATION: &[u8] = b"REPROVISION_PENDING_CORRELATION\xFF";

fn certificate_material(marker: &str) -> Structure {
    test_structure([test_item(
        NESTED_VALUE,
        Value::byte_string(marker.as_bytes().to_vec()),
    )])
}

fn request_fields(bytes: &[u8]) -> (Option<u32>, Vec<u32>) {
    let message = decode(bytes).expect("captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("KMIP request message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .find(|field| field.tag().raw() == BATCH_ITEM)
            .expect("request contains a Batch Item");
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
                .expect("request contains a Request Payload");
            let tags = payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                payload
                    .children()
                    .iter()
                    .map(|field| field.tag().raw())
                    .collect()
            });
            (operation, tags)
        })
    })
}

#[test]
fn reprovision_sends_certificate_request_form_and_typed_identifier_once() {
    crate::execute_test_support::assert_lifecycle_request_debug(
        &crate::ClientRequest::reprovision(
            ReProvisionRequest::new().with_certificate_request(certificate_material("debug-csr")),
        ),
        "ReProvision",
    );
    let response = asynchronous_response_bytes(
        REPROVISION_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("new-client-credential".to_owned()),
        )])),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![3, 8],
    });

    let item = client
        .reprovision(
            ReProvisionRequest::new().with_certificate_request(certificate_material("csr-data")),
            &CodecLimits::defaults(),
        )
        .expect("server returns a valid Re-Provision result");
    let outcome = item.outcome();
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        outcome,
        ClientOperation::ReProvision,
        "ReProvision(",
        "Re-Provision(",
    );
    let typed = outcome
        .response()
        .reprovision()
        .expect("the response exposes typed Re-Provision fields");
    assert_eq!(outcome.operation(), ClientOperation::ReProvision);
    assert_eq!(outcome.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.unique_identifier(),
        Some(&UniqueIdentifier::TextString(
            "new-client-credential".to_owned()
        ))
    );
    assert_eq!(
        request_fields(captured.borrow().as_deref().expect("request captured")),
        (Some(REPROVISION_OPERATION), vec![CERTIFICATE_REQUEST])
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn reprovision_sends_parameterless_server_credential_form() {
    let response = asynchronous_response_bytes(
        REPROVISION_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([])),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    client
        .reprovision(ReProvisionRequest::new(), &CodecLimits::defaults())
        .expect("parameterless server-generated credential request is valid");
    assert_eq!(
        request_fields(captured.borrow().as_deref().expect("request captured")),
        (Some(REPROVISION_OPERATION), Vec::new())
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn reprovision_sends_replacement_certificate_form() {
    let response = asynchronous_response_bytes(
        REPROVISION_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([])),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    client
        .reprovision(
            ReProvisionRequest::new().with_certificate(certificate_material("certificate-data")),
            &CodecLimits::defaults(),
        )
        .expect("replacement certificate request is valid");
    assert_eq!(
        request_fields(captured.borrow().as_deref().expect("request captured")),
        (Some(REPROVISION_OPERATION), vec![CERTIFICATE])
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn reprovision_sends_both_optional_certificate_fields() {
    let response = asynchronous_response_bytes(
        REPROVISION_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([])),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    client
        .reprovision(
            ReProvisionRequest::new()
                .with_certificate_request(certificate_material("csr-data"))
                .with_certificate(certificate_material("certificate-data")),
            &CodecLimits::defaults(),
        )
        .expect("§6.1.48 and Table 313 do not prohibit both optional fields");
    assert_eq!(
        request_fields(captured.borrow().as_deref().expect("request captured")),
        (
            Some(REPROVISION_OPERATION),
            vec![CERTIFICATE_REQUEST, CERTIFICATE]
        )
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn reprovision_failure_preserves_table_315_result_without_retry() {
    let response = asynchronous_response_bytes(
        REPROVISION_OPERATION,
        OPERATION_FAILED,
        Some(PERMISSION_DENIED),
        None,
        None,
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let item = client
        .reprovision(ReProvisionRequest::new(), &CodecLimits::defaults())
        .expect("Table 315 failure is a KMIP result, not a local error");
    assert_eq!(item.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        item.outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(PERMISSION_DENIED)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn reprovision_pending_preserves_correlation_without_implicit_poll() {
    let response = asynchronous_response_bytes(
        REPROVISION_OPERATION,
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::ReProvision(
        ReProvisionRequest::new().with_certificate(certificate_material("certificate")),
    )))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("caller-enabled typed batch preserves Re-Provision Pending");
    let item = response.get(0).expect("one result is associated");
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        item.outcome(),
        ClientOperation::ReProvision,
        "ReProvision(",
        "Pending(",
    );
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("Operation Pending remains Pending");
    };
    assert_eq!(pending.operation(), ClientOperation::ReProvision);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn reprovision_transport_failure_preserves_delivery_state() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 4 });
    let error = client
        .reprovision(ReProvisionRequest::new(), &CodecLimits::defaults())
        .expect_err("failed exchange is surfaced");
    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
