//! Fake-transport Validate behavior derived from OASIS KMIP v2.1 §6.1.57,
//! Tables 340–342. These tests are not official fixture passes.

use kmipkit_protocol::{UniqueIdentifier, ValidateRequest};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::ValueView;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};

const VALIDATE: u32 = 0x0000_0017;
const BATCH_ITEM: u32 = 0x0042_000f;
const OPERATION: u32 = 0x0042_005c;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;

fn request() -> ValidateRequest {
    ValidateRequest::new()
        .with_certificate(test_structure([
            test_item(0x0042_001d, kmipkit_ttlv::Value::enumeration(1)),
            test_item(
                0x0042_001e,
                kmipkit_ttlv::Value::byte_string(b"validate-private-sentinel".to_vec()),
            ),
        ]))
        .with_unique_identifier(UniqueIdentifier::Enumeration(0xf123_4567))
        .with_validity_date(17)
}

fn success_payload() -> kmipkit_ttlv::Structure {
    test_structure([test_item(
        0x0042_009b,
        kmipkit_ttlv::Value::enumeration(0xf123_4567),
    )])
}

fn request_shape(request: &[u8]) -> Option<(u32, usize)> {
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
            let payload_len = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)?
                .with_value(|value| match value {
                    ValueView::Structure(payload) => Some(payload.children().len()),
                    _ => None,
                })?;
            Some((operation, payload_len))
        })
    })
}

#[test]
fn validate_sends_chain_and_date_and_returns_open_indicator() {
    let response = asynchronous_response_bytes(VALIDATE, 0, None, None, Some(success_payload()));
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![2, 5],
    });

    let outcome = client
        .validate(request(), &CodecLimits::defaults())
        .expect("the fake server returns a Table 341 Validity Indicator");
    let ClientBatchOutcome::Validate(validate) = outcome.outcome() else {
        panic!("the typed outcome is Validate");
    };

    assert_eq!(validate.result().status().raw(), 0);
    assert_eq!(validate.validity_indicator().unwrap().raw(), 0xf123_4567);
    assert_eq!(outcome.outcome().operation(), ClientOperation::Validate);
    assert_eq!(outcome.outcome().result().status().raw(), 0);
    let response = outcome.outcome().response();
    assert!(response.validate().is_some());
    assert!(response.query().is_none());
    assert_eq!(response.result().status().raw(), 0);
    assert_eq!(
        request_shape(captured.borrow().as_deref().expect("request was captured")),
        Some((VALIDATE, 3))
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn validate_failure_preserves_transport_delivery_and_does_not_retry() {
    let (mut client, transport, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });

    let error = client
        .validate(request(), &CodecLimits::defaults())
        .expect_err("the scripted transport fails after a partial write");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn validate_pending_preserves_typed_response_and_correlation_without_follow_up() {
    const CORRELATION: &[u8] = b"validate-pending-correlation";
    let response = asynchronous_response_bytes(
        VALIDATE,
        2,
        None,
        Some(CORRELATION),
        Some(success_payload()),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::validate(request())))
        .with_asynchronous_indicator(1);

    let result = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the request permits an asynchronous Pending response");
    let outcome = result.get(0).expect("one result exists").outcome();
    let ClientBatchOutcome::Pending(pending) = outcome else {
        panic!("Validate Pending remains explicitly resumable");
    };

    assert_eq!(pending.operation(), ClientOperation::Validate);
    assert_eq!(pending.result().status().raw(), 2);
    assert_eq!(pending.asynchronous_correlation_value(), CORRELATION);
    assert_eq!(
        pending
            .response()
            .validate()
            .map(|response| response.result().status().raw()),
        Some(2)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn malformed_validate_response_is_rejected_after_one_exchange() {
    let malformed_payload = test_structure([test_item(
        0x0042_0012,
        kmipkit_ttlv::Value::text_string("unexpected".to_owned()),
    )]);
    let response = asynchronous_response_bytes(VALIDATE, 0, None, None, Some(malformed_payload));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let error = client
        .validate(request(), &CodecLimits::defaults())
        .expect_err("Validate success requires one Validity Indicator");

    assert!(error.delivery_state().is_some());
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn failure_reason_message_and_diagnostics_are_lossless_and_redacted() {
    let response = crate::execute_test_support::one_item_response_bytes(
        VALIDATE,
        1,
        Some(0xf123_4567),
        Some("validate-private-sentinel"),
        None,
        None,
        None,
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let outcome = client
        .validate(request(), &CodecLimits::defaults())
        .unwrap();
    assert_eq!(
        outcome.outcome().result().reason().unwrap().raw(),
        0xf123_4567
    );
    assert_eq!(
        outcome.outcome().result().message().unwrap().as_str(),
        "validate-private-sentinel"
    );
    assert!(
        !format!(
            "{:?} {} {:?}",
            outcome.outcome(),
            outcome.outcome(),
            outcome.outcome().response()
        )
        .contains("validate-private-sentinel")
    );
    assert!(
        !format!("{:?}", ClientRequest::validate(request())).contains("validate-private-sentinel")
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn every_transport_delivery_state_is_preserved_without_retry() {
    for (script, expected) in [
        (
            ExchangeScript::FailBeforeWrite,
            RequestDeliveryState::NotSent,
        ),
        (
            ExchangeScript::FailAfterPartialWrite { written_bytes: 1 },
            RequestDeliveryState::PossiblySent,
        ),
        (
            ExchangeScript::FailAfterPartialRead {
                written_bytes: 1,
                response_bytes: b"private".to_vec(),
            },
            RequestDeliveryState::ResponseStarted,
        ),
    ] {
        let (mut client, transport, _) = client_for(script);
        let error = client
            .validate(request(), &CodecLimits::defaults())
            .unwrap_err();
        assert_eq!(error.delivery_state(), Some(expected));
        assert!(!format!("{error:?} {error}").contains("validate-private-sentinel"));
        assert_eq!(transport.borrow().exchange_count(), 1);
    }
}

#[test]
fn codec_failure_is_not_sent_and_operation_mismatch_is_response_started() {
    let response = asynchronous_response_bytes(VALIDATE, 0, None, None, Some(success_payload()));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let error = client
        .validate(request(), &CodecLimits::new(1024, 8, 1).unwrap())
        .unwrap_err();
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
    let response = asynchronous_response_bytes(0x3b, 0, None, None, Some(success_payload()));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let error = client
        .validate(request(), &CodecLimits::defaults())
        .unwrap_err();
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn malformed_local_certificate_is_not_sent_and_redacted() {
    let response = asynchronous_response_bytes(VALIDATE, 0, None, None, Some(success_payload()));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let bad = ValidateRequest::new().with_certificate(test_structure([test_item(
        0x0042_001d,
        kmipkit_ttlv::Value::text_string("validate-private-sentinel".to_owned()),
    )]));
    let error = client.validate(bad, &CodecLimits::defaults()).unwrap_err();
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert!(!format!("{error:?} {error}").contains("validate-private-sentinel"));
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn omitted_chain_and_date_are_forwarded_without_local_policy() {
    let response = asynchronous_response_bytes(VALIDATE, 0, None, None, Some(success_payload()));
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let result = client
        .validate(ValidateRequest::new(), &CodecLimits::defaults())
        .unwrap();
    assert_eq!(
        result
            .outcome()
            .response()
            .validate()
            .unwrap()
            .validity_indicator()
            .unwrap()
            .raw(),
        0xf123_4567
    );
    assert_eq!(
        request_shape(captured.borrow().as_deref().unwrap()),
        Some((VALIDATE, 0))
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn validate_without_chain_member_is_not_sent() {
    let response = asynchronous_response_bytes(VALIDATE, 0, None, None, Some(success_payload()));
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let error = client
        .validate(ValidateRequest::new(), &CodecLimits::defaults())
        .unwrap_err();
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn unknown_request_extensions_are_sent_once_and_redacted() {
    let response = asynchronous_response_bytes(VALIDATE, 0, None, None, Some(success_payload()));
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let request = request().with_extension(test_item(
        0x0054_1234,
        kmipkit_ttlv::Value::byte_string(b"extension-private-sentinel".to_vec()),
    ));
    let outcome = client.validate(request, &CodecLimits::defaults()).unwrap();
    assert!(
        !format!("{:?} {}", outcome.outcome(), outcome.outcome())
            .contains("extension-private-sentinel")
    );
    let guard = captured.borrow();
    let tree = decode_with_limits(guard.as_deref().unwrap(), &CodecLimits::defaults()).unwrap();
    tree.with_value(|value| {
        let ValueView::Structure(root) = value else {
            panic!("request root")
        };
        assert!(crate::execute_test_support::structure_contains_byte_string(
            &root,
            0x0054_1234,
            b"extension-private-sentinel"
        ));
    });
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn unknown_result_status_is_preserved_with_generic_payload() {
    let response = asynchronous_response_bytes(
        VALIDATE,
        0xf123_4567,
        Some(0xf765_4321),
        None,
        Some(test_structure([test_item(
            0x0054_1234,
            kmipkit_ttlv::Value::byte_string(b"generic-body-private-sentinel".to_vec()),
        )])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let outcome = client
        .validate(request(), &CodecLimits::defaults())
        .unwrap();
    let response = outcome.outcome().response();
    assert_eq!(response.result().status().raw(), 0xf123_4567);
    assert_eq!(response.result().reason().unwrap().raw(), 0xf765_4321);
    response.validate().unwrap().with_ttlv(|tree| {
        assert!(crate::execute_test_support::structure_contains_byte_string(
            &tree,
            0x0054_1234,
            b"generic-body-private-sentinel"
        ));
    });
    assert!(!format!("{response:?}").contains("generic-body-private-sentinel"));
    assert_eq!(transport.borrow().exchange_count(), 1);
}
