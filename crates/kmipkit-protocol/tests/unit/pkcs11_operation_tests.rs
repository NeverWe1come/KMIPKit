//! Source-derived OASIS KMIP v2.1 §6.1.37 Tables 273–275 and
//! §§7.26–7.30 Tables 378–382; KMIPKIT-ELEM-OP-C2S-PKCS-11.
//! No official PKCS#11 fixture or profile-conformance claim is implied.
use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{Pkcs11Error, Pkcs11Request, Pkcs11Response, ResultReason, SecretBytes};
use kmipkit_ttlv::{Structure, Value, ValueView};
const OP: u32 = 0x33;
const INTERFACE: u32 = 0x420159;
const FUNCTION: u32 = 0x42015a;
const INPUT: u32 = 0x42015b;
const OUTPUT: u32 = 0x42015c;
const RETURN: u32 = 0x42015d;
const CORRELATION: u32 = 0x4200d6;
const UNKNOWN: u32 = 0x541234;
const SENTINEL: &[u8] = b"pkcs11-secret-sentinel";
fn function(value: Value) -> Structure {
    structure([item(FUNCTION, value)])
}
fn success() -> Structure {
    structure([
        item(
            FUNCTION,
            Value::structure(function(Value::enumeration(0xffff1234))),
        ),
        item(
            RETURN,
            Value::structure(structure([item(RETURN, Value::enumeration(0x8fff1234))])),
        ),
    ])
}
fn response(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<Pkcs11Response, Pkcs11Error> {
    let message = response_message(
        OP,
        status,
        reason,
        if status == 2 { Some(SENTINEL) } else { None },
        payload,
    );
    Pkcs11Response::try_from_response_item(message.batch_items().next().unwrap())
}
#[test]
fn table_273_request_preserves_order_and_opaque_members() {
    let interface = structure([item(
        INTERFACE,
        Value::text_string("vendor-interface".into()),
    )]);
    let input = structure([
        item(INPUT, Value::byte_string(SENTINEL.to_vec())),
        item(UNKNOWN, Value::integer(-71)),
    ]);
    let request = Pkcs11Request::new(function(Value::enumeration(1)))
        .with_interface(interface)
        .with_correlation_value(SecretBytes::new(SENTINEL.to_vec()))
        .with_input_parameters(input)
        .with_extension(item(UNKNOWN, Value::integer(19)));
    assert!(!format!("{request:?}").contains("pkcs11-secret-sentinel"));
    let payload = request.into_ttlv_payload().unwrap();
    assert_eq!(
        payload
            .view()
            .children()
            .iter()
            .map(|x| x.tag().raw())
            .collect::<Vec<_>>(),
        [INTERFACE, FUNCTION, CORRELATION, INPUT, UNKNOWN]
    );
    payload.view().children()[2]
        .with_value(|v| assert!(matches!(v,ValueView::ByteString(b) if b==SENTINEL)));
    payload.view().children()[3].with_value(|v| {
        let ValueView::Structure(s) = v else {
            panic!("Structure")
        };
        assert_eq!(s.children()[1].tag().raw(), UNKNOWN);
    });
}
#[test]
fn table_273_function_preserves_enumeration_or_integer() {
    for value in [Value::enumeration(0xffff1234), Value::integer(-123)] {
        let payload = Pkcs11Request::new(function(value))
            .into_ttlv_payload()
            .unwrap();
        assert_eq!(payload.view().children().len(), 1);
        payload.view().children()[0].with_value(|v| {
            let ValueView::Structure(s) = v else {
                panic!("Structure")
            };
            s.children()[0].with_value(|v| {
                assert!(matches!(
                    v,
                    ValueView::Enumeration(0xffff1234) | ValueView::Integer(-123)
                ))
            });
        });
    }
}
#[test]
fn table_273_rejects_missing_duplicate_or_wrong_function() {
    for f in [
        Structure::new(),
        function(Value::text_string("pkcs11-secret-sentinel".into())),
        structure([
            item(FUNCTION, Value::integer(1)),
            item(FUNCTION, Value::integer(2)),
        ]),
    ] {
        let error = Pkcs11Request::new(f).into_ttlv_payload().unwrap_err();
        assert!(!format!("{error:?} {error}").contains("pkcs11-secret-sentinel"));
    }
}
#[test]
fn table_274_success_retains_required_optional_and_unknown_fields() {
    let payload = structure([
        item(INTERFACE, Value::structure(Structure::new())),
        item(FUNCTION, Value::structure(function(Value::integer(-123)))),
        item(CORRELATION, Value::byte_string(SENTINEL.to_vec())),
        item(
            OUTPUT,
            Value::structure(structure([
                item(OUTPUT, Value::byte_string(SENTINEL.to_vec())),
                item(UNKNOWN, Value::integer(5)),
            ])),
        ),
        item(
            RETURN,
            Value::structure(structure([
                item(RETURN, Value::enumeration(0x8fff1234)),
                item(UNKNOWN, Value::integer(6)),
            ])),
        ),
        item(UNKNOWN, Value::integer(7)),
    ]);
    let result = response(0, None, Some(payload)).unwrap();
    assert_eq!(result.result().status().raw(), 0);
    assert_eq!(result.with_interface(|x| x.children().len()), Some(0));
    assert_eq!(result.with_function(|x| x.children().len()), Some(1));
    assert_eq!(
        result.with_correlation_value(<[u8]>::to_vec),
        Some(SENTINEL.to_vec())
    );
    assert_eq!(
        result.with_output_parameters(|x| x.children().len()),
        Some(2)
    );
    assert_eq!(result.with_return_code(|x| x.children().len()), Some(2));
    assert_eq!(result.with_ttlv_payload(|x| x.children().len()), Some(6));
    assert!(!format!("{result:?}").contains("pkcs11-secret-sentinel"));
}
#[test]
fn table_274_absence_remains_absent() {
    let result = response(0, None, Some(success())).unwrap();
    assert_eq!(result.with_interface(|_| ()), None);
    assert_eq!(result.with_output_parameters(|_| ()), None);
    assert_eq!(result.with_correlation_value(|_| ()), None);
}
#[test]
fn table_274_rejects_missing_duplicate_wrong_type_and_out_of_order_fields() {
    for payload in [
        Structure::new(),
        structure([
            item(FUNCTION, Value::enumeration(1)),
            item(RETURN, Value::structure(function(Value::enumeration(1)))),
        ]),
        structure([item(
            FUNCTION,
            Value::structure(function(Value::enumeration(1))),
        )]),
        structure([
            item(
                RETURN,
                Value::structure(structure([item(RETURN, Value::enumeration(1))])),
            ),
            item(FUNCTION, Value::structure(function(Value::enumeration(1)))),
        ]),
        structure([
            item(FUNCTION, Value::structure(function(Value::enumeration(1)))),
            item(FUNCTION, Value::structure(function(Value::enumeration(1)))),
            item(
                RETURN,
                Value::structure(structure([item(RETURN, Value::enumeration(1))])),
            ),
        ]),
        structure([
            item(FUNCTION, Value::structure(function(Value::enumeration(1)))),
            item(RETURN, Value::structure(Structure::new())),
        ]),
    ] {
        assert!(response(0, None, Some(payload)).is_err());
    }
}
#[test]
fn table_275_preserves_failure_reasons() {
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
        let result = response(1, Some(reason.raw()), None).unwrap();
        assert_eq!(result.result().reason(), Some(reason));
        assert_eq!(result.with_function(|_| ()), None);
        assert_eq!(result.with_return_code(|_| ()), None);
    }
    assert_eq!(
        response(1, Some(0xffff1234), None)
            .unwrap()
            .result()
            .reason()
            .unwrap()
            .raw(),
        0xffff1234
    );
}
#[test]
fn pending_retains_result_and_generic_payload_without_fabricated_fields() {
    let result = response(2, None, Some(Structure::new())).unwrap();
    assert_eq!(result.result().status().raw(), 2);
    assert_eq!(result.with_function(|_| ()), None);
    assert_eq!(result.with_ttlv_payload(|x| x.children().len()), Some(0));
}
#[test]
fn diagnostics_redact_all_pkcs11_sentinels() {
    let malformed = structure([item(FUNCTION, Value::byte_string(SENTINEL.to_vec()))]);
    let error = response(0, None, Some(malformed)).unwrap_err();
    assert!(!format!("{error:?} {error}").contains("pkcs11-secret-sentinel"));
    let message = response_message(OP + 1, 0, None, None, Some(success()));
    assert_eq!(
        Pkcs11Response::try_from_response_item(message.batch_items().next().unwrap()).unwrap_err(),
        Pkcs11Error::UnexpectedOperation
    );
}
