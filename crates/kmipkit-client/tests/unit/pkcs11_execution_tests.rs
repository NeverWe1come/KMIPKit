//! Fake-transport PKCS#11 tests derived from OASIS KMIP v2.1 §6.1.37
//! Tables 273–275. No official fixture/profile/interoperability claim.
use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{
    asynchronous_response_bytes, test_item as item, test_structure as structure,
};
use crate::{ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest};
use kmipkit_protocol::{Pkcs11Request, Pkcs11Response, ResultReason, SecretBytes};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Structure, Value, ValueView};
const OP: u32 = 0x33;
const FUNCTION: u32 = 0x0042_015A;
const RETURN: u32 = 0x0042_015D;
const SENTINEL: &[u8] = b"pkcs11-client-secret-sentinel";
fn parity_fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../../tests/fixtures/api/pkcs11-parity.json"
    ))
    .expect("the shared PKCS#11 parity fixture is valid JSON")
}

fn fixture_hex_u32(value: &serde_json::Value) -> u32 {
    u32::from_str_radix(
        value
            .as_str()
            .expect("shared fixture hexadecimal values are strings")
            .strip_prefix("0x")
            .expect("shared fixture hexadecimal values have a prefix"),
        16,
    )
    .expect("shared fixture hexadecimal value fits u32")
}

fn fixture_hex_bytes(value: &serde_json::Value) -> Vec<u8> {
    let text = value
        .as_str()
        .expect("shared fixture byte strings are hexadecimal strings");
    let (pairs, remainder) = text.as_bytes().as_chunks::<2>();
    assert_eq!(remainder, &[] as &[u8]);
    pairs
        .iter()
        .map(|pair| {
            let digits = std::str::from_utf8(pair).expect("hexadecimal digits are ASCII");
            u8::from_str_radix(digits, 16).expect("shared fixture byte is hexadecimal")
        })
        .collect()
}

fn fixture_structure(value: &serde_json::Value) -> Option<Structure> {
    if value.is_null() || !value["present"].as_bool().unwrap_or(false) {
        return None;
    }
    let child_tag = fixture_hex_u32(&value["childTag"]);
    let child_value = match value["childType"].as_str() {
        Some("Integer") => Value::integer(
            i32::try_from(value["childValue"].as_i64().expect("Integer field value"))
                .expect("Integer field fits i32"),
        ),
        Some("Enumeration") => Value::enumeration(fixture_hex_u32(&value["childValue"])),
        Some("ByteString") => Value::byte_string(fixture_hex_bytes(&value["childHex"])),
        _ => panic!("shared fixture uses a supported KMIP TTLV type"),
    };
    Some(structure([item(child_tag, child_value)]))
}

fn expected_response_tags() -> Vec<u32> {
    parity_fixture()["expectedOutput"]["responseTags"]
        .as_array()
        .expect("shared fixture response tags are an array")
        .iter()
        .map(fixture_hex_u32)
        .collect()
}

fn request() -> Pkcs11Request {
    Pkcs11Request::new(structure([item(FUNCTION, Value::integer(-12345))]))
        .with_correlation_value(SecretBytes::new(SENTINEL.to_vec()))
        .with_input_parameters(structure([item(
            0x0042_015B,
            Value::byte_string(SENTINEL.to_vec()),
        )]))
}
fn success() -> Structure {
    let fixture = parity_fixture();
    let expected = &fixture["expectedOutput"];
    let typed = &expected["typedResponse"];
    let mut interface = fixture_structure(&typed["interface"]);
    let mut function = fixture_structure(&typed["function"]);
    let mut output_parameters = fixture_structure(&typed["outputParameters"]);
    let mut return_code = fixture_structure(&typed["returnCode"]);
    let interface_tag =
        (!typed["interface"].is_null()).then(|| fixture_hex_u32(&typed["interface"]["tag"]));
    let function_tag = fixture_hex_u32(&typed["function"]["tag"]);
    let output_parameters_tag = fixture_hex_u32(&typed["outputParameters"]["tag"]);
    let return_code_tag = fixture_hex_u32(&typed["returnCode"]["tag"]);
    let correlation = &typed["correlationValue"];
    let vendor = &expected["vendorExtension"];
    let mut payload = Vec::new();
    for tag in expected_response_tags() {
        if interface.is_some() && interface_tag == Some(tag) {
            payload.push(item(
                tag,
                Value::structure(interface.take().expect("Interface is present")),
            ));
        } else if tag == function_tag {
            payload.push(item(
                tag,
                Value::structure(function.take().expect("Function is present")),
            ));
        } else if correlation["present"].as_bool() == Some(true)
            && tag == fixture_hex_u32(&correlation["tag"])
        {
            payload.push(item(
                tag,
                Value::byte_string(fixture_hex_bytes(&correlation["hex"])),
            ));
        } else if output_parameters.is_some() && tag == output_parameters_tag {
            payload.push(item(
                tag,
                Value::structure(output_parameters.take().expect("Output Parameters present")),
            ));
        } else if return_code.is_some() && tag == return_code_tag {
            payload.push(item(
                tag,
                Value::structure(return_code.take().expect("Return Code is present")),
            ));
        } else if tag == fixture_hex_u32(&vendor["tag"]) {
            payload.push(item(
                tag,
                Value::integer(
                    i32::try_from(
                        vendor["integer"]
                            .as_i64()
                            .expect("vendor extension integer"),
                    )
                    .expect("vendor extension integer fits i32"),
                ),
            ));
        } else {
            panic!("shared fixture has an unmapped PKCS#11 response tag");
        }
    }
    structure(payload)
}

fn assert_typed_response_mapping(result: &Pkcs11Response, fixture: &serde_json::Value) {
    let typed = &fixture["expectedOutput"]["typedResponse"];
    assert_eq!(result.with_interface(|_| true), None);
    assert_eq!(
        result.with_function(|function| {
            function.children()[0].with_value(|value| {
                matches!(value, ValueView::Integer(number)
                    if i64::from(*number) == typed["function"]["childValue"].as_i64().unwrap_or_default())
            })
        }),
        Some(true)
    );
    assert_eq!(
        result.with_correlation_value(<[u8]>::to_vec),
        Some(fixture_hex_bytes(&typed["correlationValue"]["hex"]))
    );
    assert_eq!(
        result.with_output_parameters(|parameters| {
            parameters.children()[0].with_value(|value| {
                matches!(value, ValueView::ByteString(bytes)
                    if bytes == fixture_hex_bytes(&typed["outputParameters"]["childHex"]))
            })
        }),
        Some(true)
    );
    assert_eq!(
        result.with_return_code(|return_code| {
            return_code.children()[0].with_value(|value| {
                matches!(value, ValueView::Enumeration(raw)
                    if *raw == fixture_hex_u32(&typed["returnCode"]["childValue"]))
            })
        }),
        Some(true)
    );
    assert_eq!(
        result.with_ttlv_payload(|payload| {
            payload
                .children()
                .iter()
                .map(|child| child.tag().raw())
                .collect::<Vec<_>>()
        }),
        Some(expected_response_tags())
    );
    assert_eq!(result.with_ttlv(|payload| payload.children().len()), 3);
}

fn assert_request_wire_fields(request_bytes: &[u8]) {
    let decoded = decode_with_limits(request_bytes, &CodecLimits::defaults()).unwrap();
    decoded.with_value(|value| {
        let ValueView::Structure(root) = value else {
            panic!("root")
        };
        let batch = root
            .children()
            .iter()
            .find(|field| field.tag().raw() == 0x0042_000F)
            .unwrap();
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("batch")
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == 0x0042_005C)
                .unwrap();
            operation.with_value(|value| {
                assert!(matches!(value, ValueView::Enumeration(code) if *code == OP));
            });
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == 0x0042_0079)
                .unwrap();
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("payload")
                };
                assert_eq!(
                    payload
                        .children()
                        .iter()
                        .map(|field| field.tag().raw())
                        .collect::<Vec<_>>(),
                    [FUNCTION, 0x0042_00D6, 0x0042_015B]
                );
                payload.children()[1].with_value(|value| {
                    assert!(matches!(value, ValueView::ByteString(bytes) if bytes == SENTINEL));
                });
            });
        });
    });
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
    assert_typed_response_mapping(result, &parity_fixture());
    let captured = captured.borrow();
    assert_request_wire_fields(captured.as_deref().unwrap());
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
