//! Source-derived OASIS KMIP v2.1 §6.1.57, Tables 340–342; §2.1, Table 2.
//! Six KMIPKIT-REQ-SPEC-6.1.57 requirements are named below.
//! No official OASIS fixture or local certificate-validation claim is made.

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{UniqueIdentifier, ValidateError, ValidateRequest, ValidateResponse};
use kmipkit_ttlv::{Structure, Value, ValueView};

const OPERATION: u32 = 0x17;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_TYPE: u32 = 0x0042_001d;
const CERTIFICATE_VALUE: u32 = 0x0042_001e;
const IDENTIFIER: u32 = 0x0042_0094;
const DATE: u32 = 0x0042_009a;
const INDICATOR: u32 = 0x0042_009b;
const VENDOR: u32 = 0x0054_1234;
const SENTINEL: &[u8] = b"validate-certificate-private-sentinel";

fn certificate(bytes: &[u8]) -> Structure {
    structure([
        item(CERTIFICATE_TYPE, Value::enumeration(0xf123_4567)),
        item(CERTIFICATE_VALUE, Value::byte_string(bytes.to_vec())),
    ])
}

fn response(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<ValidateResponse, ValidateError> {
    let message = response_message(
        operation,
        status,
        reason,
        (status == 2).then_some(b"correlation".as_slice()),
        payload,
    );
    ValidateResponse::try_from_response_item(message.batch_items().next().unwrap())
}

fn field_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|i| i.tag().raw())
        .collect()
}

#[test]
fn req_001_models_one_chain_and_rejects_malformed_certificate_wrapper() {
    let request = ValidateRequest::new()
        .with_certificate(certificate(SENTINEL))
        .with_unique_identifier(UniqueIdentifier::Integer(7))
        .with_certificate(certificate(b"second"));
    assert_eq!(request.chain_len(), 3);
    assert_eq!(
        field_tags(&request.into_ttlv_payload().unwrap()),
        [CERTIFICATE, CERTIFICATE, IDENTIFIER]
    );
    let second_chain = structure([item(CERTIFICATE, Value::structure(certificate(SENTINEL)))]);
    assert!(
        ValidateRequest::new()
            .with_certificate(second_chain)
            .into_ttlv_payload()
            .is_err()
    );
}

#[test]
fn req_001_rejects_empty_chain_even_when_validity_date_is_present() {
    let empty = ValidateRequest::new();
    assert_eq!(empty.chain_len(), 0);
    assert!(empty.into_ttlv_payload().is_err());

    let date_only = ValidateRequest::new().with_validity_date(17);
    assert_eq!(date_only.chain_len(), 0);
    assert!(date_only.into_ttlv_payload().is_err());

    let identifier_and_date = ValidateRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(
            "managed-certificate".to_owned(),
        ))
        .with_validity_date(17);
    assert_eq!(
        field_tags(&identifier_and_date.into_ttlv_payload().unwrap()),
        [IDENTIFIER, DATE]
    );
}

#[test]
fn req_002_001_certificate_identifier_and_mixed_inputs_are_available() {
    let certificate_only = ValidateRequest::new().with_certificate(certificate(SENTINEL));
    assert_eq!(
        field_tags(&certificate_only.into_ttlv_payload().unwrap()),
        [CERTIFICATE]
    );
    let identifier_only = ValidateRequest::new().with_unique_identifier(
        UniqueIdentifier::TextString("managed-certificate".to_owned()),
    );
    assert_eq!(
        field_tags(&identifier_only.into_ttlv_payload().unwrap()),
        [IDENTIFIER]
    );
    let mixed = ValidateRequest::new()
        .with_unique_identifier(UniqueIdentifier::Integer(7))
        .with_certificate(certificate(SENTINEL));
    assert_eq!(
        field_tags(&mixed.into_ttlv_payload().unwrap()),
        [CERTIFICATE, IDENTIFIER]
    );
}

#[test]
fn req_002_002_optional_validity_date_preserves_exact_value() {
    for date in [i64::MIN, 0, i64::MAX] {
        let request = ValidateRequest::new()
            .with_unique_identifier(UniqueIdentifier::TextString(
                "managed-certificate".to_owned(),
            ))
            .with_validity_date(date);
        assert_eq!(request.validity_date(), Some(date));
        let payload = request.into_ttlv_payload().unwrap();
        assert_eq!(field_tags(&payload), [IDENTIFIER, DATE]);
        payload.view().children()[1]
            .with_value(|v| assert!(matches!(v, ValueView::DateTime(raw) if *raw == date)));
    }
}

#[test]
fn req_004_repeated_certificate_values_preserve_order() {
    let payload = ValidateRequest::new()
        .with_certificate(certificate(b"first"))
        .with_certificate(certificate(SENTINEL))
        .into_ttlv_payload()
        .unwrap();
    assert_eq!(field_tags(&payload), [CERTIFICATE, CERTIFICATE]);
    for (field, expected) in payload
        .view()
        .children()
        .iter()
        .zip([b"first".as_slice(), SENTINEL])
    {
        field.with_value(|v| {
            let ValueView::Structure(cert) = v else {
                panic!("certificate Structure")
            };
            cert.children()[1].with_value(|v| {
                assert!(matches!(v, ValueView::ByteString(bytes) if bytes == expected));
            });
        });
    }
}

#[test]
fn req_005_repeated_identifiers_preserve_order_and_open_values() {
    let payload = ValidateRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("id".to_owned()))
        .with_unique_identifier(UniqueIdentifier::Enumeration(0xf123_4567))
        .with_unique_identifier(UniqueIdentifier::Integer(-7))
        .into_ttlv_payload()
        .unwrap();
    assert_eq!(field_tags(&payload), [IDENTIFIER, IDENTIFIER, IDENTIFIER]);
    let view = payload.view();
    view.children()[0].with_value(|v| assert!(matches!(v, ValueView::TextString("id"))));
    view.children()[1].with_value(|v| assert!(matches!(v, ValueView::Enumeration(0xf123_4567))));
    view.children()[2].with_value(|v| assert!(matches!(v, ValueView::Integer(-7))));
}

#[test]
fn req_006_omitted_date_remains_absent_in_request_serialization() {
    let request = ValidateRequest::new().with_unique_identifier(UniqueIdentifier::TextString(
        "managed-certificate".to_owned(),
    ));
    assert_eq!(request.validity_date(), None);
    assert_eq!(request.chain_len(), 1);
    assert_eq!(
        field_tags(&request.into_ttlv_payload().unwrap()),
        [IDENTIFIER]
    );
    let payload = ValidateRequest::default()
        .with_certificate(certificate(SENTINEL))
        .into_ttlv_payload()
        .unwrap();
    assert_eq!(field_tags(&payload), [CERTIFICATE]);
}

#[test]
fn table_2_certificate_structure_rejects_missing_wrong_and_duplicate_fields() {
    let cases = [
        Structure::new(),
        structure([
            item(CERTIFICATE_TYPE, Value::integer(1)),
            item(CERTIFICATE_VALUE, Value::byte_string(SENTINEL.to_vec())),
        ]),
        structure([
            item(CERTIFICATE_TYPE, Value::enumeration(1)),
            item(
                CERTIFICATE_VALUE,
                Value::text_string("validate-certificate-private-sentinel".to_owned()),
            ),
        ]),
        structure([item(CERTIFICATE_TYPE, Value::enumeration(1))]),
        structure([item(
            CERTIFICATE_VALUE,
            Value::byte_string(SENTINEL.to_vec()),
        )]),
        structure([
            item(CERTIFICATE_TYPE, Value::enumeration(1)),
            item(CERTIFICATE_TYPE, Value::enumeration(2)),
            item(CERTIFICATE_VALUE, Value::byte_string(SENTINEL.to_vec())),
        ]),
        structure([
            item(CERTIFICATE_TYPE, Value::enumeration(1)),
            item(CERTIFICATE_VALUE, Value::byte_string(SENTINEL.to_vec())),
            item(CERTIFICATE_VALUE, Value::byte_string(SENTINEL.to_vec())),
        ]),
    ];
    for cert in cases {
        let error = ValidateRequest::new()
            .with_certificate(cert)
            .into_ttlv_payload()
            .unwrap_err();
        assert!(!format!("{error:?} {error}").contains("validate-certificate-private-sentinel"));
        assert_eq!(error.kind(), crate::ProtocolErrorKind::InvalidValue);
    }
}

#[test]
fn unknown_certificate_fields_are_retained_without_local_certificate_policy() {
    let mut cert = certificate(SENTINEL);
    cert.try_push(item(VENDOR, Value::byte_string(SENTINEL.to_vec())))
        .unwrap();
    let request = ValidateRequest::new().with_certificate(cert);
    assert!(!format!("{request:?}").contains("validate-certificate-private-sentinel"));
    let payload = request.into_ttlv_payload().unwrap();
    assert!(!format!("{payload:?}").contains("validate-certificate-private-sentinel"));
    payload.view().children()[0].with_value(|v| {
        let ValueView::Structure(cert) = v else {
            panic!("Certificate Structure")
        };
        assert_eq!(cert.children()[2].tag().raw(), VENDOR);
    });
}

#[test]
fn table_341_requires_one_open_indicator_and_retains_unknown_fields() {
    for indicator in [1, 2, 3, 0x8000_0001, 0xf123_4567] {
        let payload = structure([
            item(INDICATOR, Value::enumeration(indicator)),
            item(VENDOR, Value::byte_string(SENTINEL.to_vec())),
        ]);
        let typed = response(OPERATION, 0, None, Some(payload)).unwrap();
        assert_eq!(typed.validity_indicator().unwrap().raw(), indicator);
        assert!(!format!("{typed:?}").contains("validate-certificate-private-sentinel"));
        typed.with_ttlv(|tree| assert!(!tree.children().is_empty()));
    }
    for payload in [
        Structure::new(),
        structure([item(INDICATOR, Value::integer(1))]),
        structure([
            item(INDICATOR, Value::enumeration(1)),
            item(INDICATOR, Value::enumeration(1)),
        ]),
    ] {
        assert_eq!(
            response(OPERATION, 0, None, Some(payload)).unwrap_err(),
            ValidateError::MalformedResponsePayload
        );
    }
}

#[test]
fn table_342_failure_and_pending_preserve_results_without_indicator() {
    for name in [
        "Invalid Field",
        "Invalid Object Type",
        "Object Not Found",
        "Attestation Failed",
        "Attestation Required",
        "Feature Not Supported",
        "Invalid Message",
        "Operation Not Supported",
        "Permission Denied",
        "Response Too Large",
    ] {
        let reason = crate::ResultReason::known_values()
            .iter()
            .find(|(_, n)| *n == name)
            .unwrap()
            .0
            .raw();
        let typed = response(OPERATION, 1, Some(reason), None).unwrap();
        assert_eq!(typed.result().reason().unwrap().raw(), reason);
        assert_eq!(typed.validity_indicator(), None);
    }
    let typed = response(OPERATION, 2, None, Some(Structure::new())).unwrap();
    assert_eq!(typed.result().status().raw(), 2);
    assert_eq!(typed.validity_indicator(), None);
    assert_eq!(
        response(0x3b, 0, None, Some(Structure::new())).unwrap_err(),
        ValidateError::UnexpectedOperation
    );
}

#[test]
fn response_errors_are_payload_free_and_preserve_sources() {
    for error in [
        ValidateError::UnexpectedOperation,
        ValidateError::MissingResultStatus,
        ValidateError::MissingResponsePayload,
        ValidateError::MalformedResponsePayload,
        ValidateError::MalformedResponseItem,
    ] {
        assert!(std::error::Error::source(&error).is_none());
        assert!(!format!("{error:?} {error}").contains("validate-certificate-private-sentinel"));
    }
    let error =
        ValidateError::InvalidOperationResult(crate::ResultValidationError::SuccessForbidsReason);
    assert!(std::error::Error::source(&error).is_some());
    assert!(error.to_string().contains("Validate"));
}

#[test]
fn generic_unknown_response_data_remains_exact_and_redacted() {
    let payload = structure([item(
        0x0054_1234,
        Value::byte_string(b"generic-body-private-sentinel".to_vec()),
    )]);
    let message = response_message(
        OPERATION,
        0xf123_4567,
        Some(0xf765_4321),
        None,
        Some(payload),
    );
    let batch = message.batch_items().next().unwrap();
    let expected = batch
        .with_ttlv(|tree| {
            tree.children()
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>()
        })
        .unwrap();
    let typed = ValidateResponse::try_from_response_item(batch).unwrap();
    typed.with_ttlv(|tree| {
        assert_eq!(tree.children().iter().map(|field| field.tag().raw()).collect::<Vec<_>>(), expected);
        let payload = tree.children().iter().find(|field| field.tag().raw() == 0x0042_007c).unwrap();
        payload.with_value(|v| {
            let ValueView::Structure(payload) = v else { panic!("Response Payload Structure") };
            assert_eq!(payload.children()[0].tag().raw(), 0x0054_1234);
            payload.children()[0].with_value(|v| assert!(matches!(v, ValueView::ByteString(bytes) if bytes == b"generic-body-private-sentinel")));
        });
    });
    assert_eq!(typed.result().reason().unwrap().raw(), 0xf765_4321);
    assert!(!format!("{typed:?}").contains("generic-body-private-sentinel"));
}

#[test]
fn unknown_request_extensions_preserve_order_without_overriding_table_fields() {
    let request = ValidateRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(
            "managed-certificate".to_owned(),
        ))
        .with_extension(item(
            0x0054_1234,
            Value::byte_string(b"extension-private-sentinel".to_vec()),
        ))
        .with_extension(item(0x0054_1235, Value::enumeration(0xf123_4567)));
    assert!(!format!("{request:?}").contains("extension-private-sentinel"));
    let payload = request.into_ttlv_payload().unwrap();
    let view = payload.view();
    let fields = view.children();
    assert_eq!(fields[fields.len() - 2].tag().raw(), 0x0054_1234);
    assert_eq!(fields[fields.len() - 1].tag().raw(), 0x0054_1235);
    fields[fields.len() - 2].with_value(|v| {
        assert!(
            matches!(v, ValueView::ByteString(bytes) if bytes == b"extension-private-sentinel")
        );
    });
    fields[fields.len() - 1]
        .with_value(|v| assert!(matches!(v, ValueView::Enumeration(0xf123_4567))));
    for known in [0x0042_0013, 0x0042_0094, 0x0042_009a] {
        let error = ValidateRequest::new()
            .with_unique_identifier(UniqueIdentifier::TextString(
                "managed-certificate".to_owned(),
            ))
            .with_extension(item(
                known,
                Value::text_string("extension-private-sentinel".to_owned()),
            ))
            .into_ttlv_payload()
            .unwrap_err();
        assert_eq!(error.kind(), crate::ProtocolErrorKind::InvalidValue);
        assert!(!format!("{error:?} {error}").contains("extension-private-sentinel"));
    }
}

#[test]
fn request_extension_depth_limit_is_checked_before_encoding() {
    let mut nested = Structure::new();
    for _ in 0..63 {
        nested = structure([item(0x0054_1234, Value::structure(nested))]);
    }
    let error = ValidateRequest::new()
        .with_extension(item(0x0054_1234, Value::structure(nested)))
        .into_ttlv_payload()
        .unwrap_err();
    assert_eq!(error.kind(), crate::ProtocolErrorKind::InvalidValue);
}

// KMIPKIT-REQ-SPEC-10.1.2-001: section 10.1.2, unnumbered Type table Structure row requires description order.
#[test]
fn table_340_groups_certificates_before_identifiers_preserving_each_list_order() {
    let request = ValidateRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("first-id".to_owned()))
        .with_certificate(certificate(b"first-certificate"))
        .with_unique_identifier(UniqueIdentifier::TextString("second-id".to_owned()))
        .with_certificate(certificate(b"second-certificate"))
        .with_validity_date(17);
    let payload = request.into_ttlv_payload().unwrap();
    assert_eq!(
        field_tags(&payload),
        [CERTIFICATE, CERTIFICATE, IDENTIFIER, IDENTIFIER, DATE]
    );
    let view = payload.view();
    for (field, expected) in view.children()[..2].iter().zip([
        b"first-certificate".as_slice(),
        b"second-certificate".as_slice(),
    ]) {
        field.with_value(|v| {
            let ValueView::Structure(cert) = v else {
                panic!("Certificate Structure")
            };
            cert.children()[1].with_value(|v| {
                assert!(matches!(v, ValueView::ByteString(bytes) if bytes == expected));
            });
        });
    }
    view.children()[2].with_value(|v| assert!(matches!(v, ValueView::TextString("first-id"))));
    view.children()[3].with_value(|v| assert!(matches!(v, ValueView::TextString("second-id"))));
}

// Table 2 field order follows the same section 10.1.2 Structure rule.
#[test]
fn table_2_certificate_value_before_type_is_rejected_structurally() {
    let reversed = structure([
        item(CERTIFICATE_VALUE, Value::byte_string(SENTINEL.to_vec())),
        item(CERTIFICATE_TYPE, Value::enumeration(1)),
    ]);
    let error = ValidateRequest::new()
        .with_certificate(reversed)
        .into_ttlv_payload()
        .unwrap_err();
    assert_eq!(error.kind(), crate::ProtocolErrorKind::InvalidValue);
    assert!(!format!("{error:?} {error}").contains("validate-certificate-private-sentinel"));
}
