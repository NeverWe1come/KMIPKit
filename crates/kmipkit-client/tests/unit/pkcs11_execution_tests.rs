//! Fake-transport PKCS#11 tests derived from OASIS KMIP v2.1 §6.1.37
//! Tables 273–275. No official fixture/profile/interoperability claim.
use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{
    asynchronous_response_bytes, test_item as item, test_structure as structure,
};
use crate::{ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest};
use kmipkit_protocol::{Pkcs11Request, ResultReason, SecretBytes};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Structure, Value, ValueView};
const OP: u32 = 0x33;
const FUNCTION: u32 = 0x0042_015A;
const RETURN: u32 = 0x0042_015D;
const SENTINEL: &[u8] = b"pkcs11-client-secret-sentinel";
fn request() -> Pkcs11Request {
    Pkcs11Request::new(structure([item(FUNCTION, Value::integer(-12345))]))
        .with_correlation_value(SecretBytes::new(SENTINEL.to_vec()))
        .with_input_parameters(structure([item(
            0x0042_015B,
            Value::byte_string(SENTINEL.to_vec()),
        )]))
}
fn success() -> Structure {
    structure([
        item(
            FUNCTION,
            Value::structure(structure([item(FUNCTION, Value::integer(-12345))])),
        ),
        item(0x0042_00D6, Value::byte_string(SENTINEL.to_vec())),
        item(
            0x0042_015C,
            Value::structure(structure([item(
                0x0042_015C,
                Value::byte_string(SENTINEL.to_vec()),
            )])),
        ),
        item(
            RETURN,
            Value::structure(structure([item(RETURN, Value::enumeration(0xFFFF_1234))])),
        ),
        item(0x0054_1234, Value::integer(7)),
    ])
}
#[test]
fn pkcs11_sends_table_273_once_and_exposes_table_274_losslessly() {
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(OP, 0, None, None, Some(success())),
        request_write_chunks: vec![2, 5],
    });
    let outcome = client.pkcs11(request(), &CodecLimits::defaults()).unwrap();
    let ClientBatchOutcome::Pkcs11(result) = outcome.outcome() else {
        panic!("Pkcs11 typed result")
    };
    assert_eq!(outcome.outcome().operation(), ClientOperation::Pkcs11);
    assert_eq!(outcome.outcome().result().status().raw(), 0);
    assert!(outcome.outcome().response().pkcs11().is_some());
    assert!(outcome.outcome().response().ping().is_none());
    assert_eq!(
        result.with_correlation_value(<[u8]>::to_vec),
        Some(SENTINEL.to_vec())
    );
    assert_eq!(result.with_ttlv_payload(|p| p.children().len()), Some(5));
    assert_eq!(result.with_ttlv(|p| p.children().len()), 3);
    assert_eq!(
        result
            .with_return_code(|p| p.children()[0]
                .with_value(|v| matches!(v, ValueView::Enumeration(0xFFFF_1234)))),
        Some(true)
    );
    let captured = captured.borrow();
    let decoded =
        decode_with_limits(captured.as_deref().unwrap(), &CodecLimits::defaults()).unwrap();
    decoded.with_value(|v| {
        let ValueView::Structure(root) = v else {
            panic!("root")
        };
        let batch = root
            .children()
            .iter()
            .find(|f| f.tag().raw() == 0x0042_000F)
            .unwrap();
        batch.with_value(|v| {
            let ValueView::Structure(batch) = v else {
                panic!("batch")
            };
            let operation = batch
                .children()
                .iter()
                .find(|f| f.tag().raw() == 0x0042_005C)
                .unwrap();
            operation
                .with_value(|v| assert!(matches!(v,ValueView::Enumeration(code) if *code==OP)));
            let payload = batch
                .children()
                .iter()
                .find(|f| f.tag().raw() == 0x0042_0079)
                .unwrap();
            payload.with_value(|v| {
                let ValueView::Structure(p) = v else {
                    panic!("payload")
                };
                assert_eq!(
                    p.children()
                        .iter()
                        .map(|f| f.tag().raw())
                        .collect::<Vec<_>>(),
                    [FUNCTION, 0x0042_00D6, 0x0042_015B]
                );
                p.children()[1]
                    .with_value(|v| assert!(matches!(v,ValueView::ByteString(b) if b==SENTINEL)));
            });
        });
    });
    assert_eq!(transport.borrow().exchange_count(), 1);
    let diagnostics = format!(
        "{:?} {} {:?}",
        outcome.outcome(),
        outcome.outcome(),
        outcome.outcome().response()
    );
    assert!(!diagnostics.contains("pkcs11-client-secret-sentinel"));
    assert!(!diagnostics.contains("12345"));
}
#[test]
fn pkcs11_failure_preserves_all_table_275_reasons_without_retry() {
    for name in [
        "Invalid Asynchronous Correlation Value",
        "Attestation Failed",
        "Attestation Required",
        "Feature Not Supported",
        "Invalid Field",
        "Invalid Message",
        "Operation Not Supported",
        "Permission Denied",
        "Response Too Large",
        "PKCS#11 Codec Error",
        "PKCS#11 Invalid Function",
        "PKCS#11 Invalid Interface",
    ] {
        let reason = ResultReason::known_values()
            .iter()
            .find(|(_, n)| *n == name)
            .unwrap()
            .0;
        let (mut client, transport, _) = client_for(ExchangeScript::Success {
            response: asynchronous_response_bytes(OP, 1, Some(reason.raw()), None, None),
            request_write_chunks: vec![],
        });
        let outcome = client.pkcs11(request(), &CodecLimits::defaults()).unwrap();
        assert_eq!(outcome.outcome().result().reason(), Some(reason));
        assert_eq!(
            outcome
                .outcome()
                .response()
                .pkcs11()
                .unwrap()
                .with_function(|_| ()),
            None
        );
        assert_eq!(transport.borrow().exchange_count(), 1);
    }
}
#[test]
fn pkcs11_pending_preserves_exact_batch_correlation_without_poll() {
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(OP, 2, None, Some(SENTINEL), Some(Structure::new())),
        request_write_chunks: vec![],
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::pkcs11(request())))
        .with_asynchronous_indicator(1);
    let result = client.execute(batch, &CodecLimits::defaults()).unwrap();
    let ClientBatchOutcome::Pending(p) = result.get(0).unwrap().outcome() else {
        panic!("Pending")
    };
    assert_eq!(p.operation(), ClientOperation::Pkcs11);
    assert_eq!(p.asynchronous_correlation_value(), SENTINEL);
    assert_eq!(p.response().pkcs11().unwrap().result().status().raw(), 2);
    assert_eq!(transport.borrow().exchange_count(), 1);
    assert!(!format!("{p:?}").contains("pkcs11-client-secret-sentinel"));
}
#[test]
fn pkcs11_transport_failure_preserves_delivery_and_never_retries() {
    let (mut client, transport, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = client
        .pkcs11(request(), &CodecLimits::defaults())
        .unwrap_err();
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
    assert!(!format!("{error:?} {error}").contains("pkcs11-client-secret-sentinel"));
}
#[test]
fn pkcs11_invalid_function_is_not_sent_and_diagnostics_are_redacted() {
    let (mut client, transport, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let request = Pkcs11Request::new(structure([item(
        FUNCTION,
        Value::byte_string(SENTINEL.to_vec()),
    )]));
    let error = client
        .pkcs11(request, &CodecLimits::defaults())
        .unwrap_err();
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
    assert!(!format!("{error:?} {error}").contains("pkcs11-client-secret-sentinel"));
}
#[test]
fn pkcs11_malformed_response_retains_response_started_delivery() {
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            OP,
            0,
            None,
            None,
            Some(structure([item(
                FUNCTION,
                Value::byte_string(SENTINEL.to_vec()),
            )])),
        ),
        request_write_chunks: vec![],
    });
    let error = client
        .pkcs11(request(), &CodecLimits::defaults())
        .unwrap_err();
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
    assert!(!format!("{error:?} {error}").contains("pkcs11-client-secret-sentinel"));
}
#[test]
fn pkcs11_options_method_and_request_enum_preserve_redaction() {
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(OP, 0, None, None, Some(success())),
        request_write_chunks: vec![],
    });
    let debug = format!("{:?}", ClientRequest::pkcs11(request()));
    assert!(!debug.contains("pkcs11-client-secret-sentinel"));
    assert!(!debug.contains("12345"));
    client
        .pkcs11_with_options(
            request(),
            &CodecLimits::defaults(),
            &RequestOptions::default(),
        )
        .unwrap();
    assert_eq!(transport.borrow().exchange_count(), 1);
}

// Source: OASIS KMIP v2.1 Tables 273/274 and §§7.27–7.29 Tables 379–381.
#[test]
fn pkcs11_malformed_known_nested_request_members_are_not_sent() {
    for tag in [0x0042_0159, 0x0042_015B] {
        let (mut client, transport, _) =
            client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
        let nested = structure([item(tag, Value::integer(7))]);
        let request = if tag == 0x0042_0159 {
            request().with_interface(nested)
        } else {
            request().with_input_parameters(nested)
        };
        let error = client
            .pkcs11(request, &CodecLimits::defaults())
            .expect_err("invalid known nested scalar is local validation failure");
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        assert_eq!(transport.borrow().exchange_count(), 0);
    }
}
#[test]
fn pkcs11_malformed_known_nested_response_member_is_response_started() {
    let payload = structure([
        item(
            FUNCTION,
            Value::structure(structure([item(FUNCTION, Value::enumeration(1))])),
        ),
        item(
            0x0042_015C,
            Value::structure(structure([item(
                0x0042_015C,
                Value::text_string("pkcs11-client-secret-sentinel".into()),
            )])),
        ),
        item(
            RETURN,
            Value::structure(structure([item(RETURN, Value::enumeration(0))])),
        ),
    ]);
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(OP, 0, None, None, Some(payload)),
        request_write_chunks: vec![],
    });
    let error = client
        .pkcs11(request(), &CodecLimits::defaults())
        .expect_err("Table 381 nested Output Parameters is ByteString");
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
    assert!(!format!("{error:?} {error}").contains("pkcs11-client-secret-sentinel"));
}
