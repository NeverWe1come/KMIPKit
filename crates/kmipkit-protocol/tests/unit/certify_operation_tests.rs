//! Source-derived Certify contracts from OASIS KMIP v2.1 §6.1.6,
//! Tables 179–181. Stable requirement coverage:
//! `KMIPKIT-REQ-SPEC-6.1.6-001`, `-002-001`, `-002-002`, `-002-003`,
//! `-003`, and `-007`. These tests are derived from the pinned standard and
//! do not claim an official OASIS Test Case pass.

use crate::async_operation_fixtures::{response_message, structure};
use crate::{
    CertifyRequest, CertifyResponse, ResponseBatchItemView, ResponseMessage, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Value, ValueView};

const CERTIFY_OPERATION: u32 = 0x0000_0006;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const INVALID_CSR: u32 = 0x0000_002F;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CERTIFICATE_REQUEST_TYPE: u32 = 0x0042_0019;
const ATTRIBUTES: u32 = 0x0042_0125;
const CERTIFICATE_TYPE: u32 = 0x0042_001D;
const CERTIFICATE_REQUEST_VALUE: u32 = 0x0042_0140;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one validated response batch item")
}

fn field_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

fn structure_contains_tag(view: &StructureView<'_>, raw_tag: u32) -> bool {
    view.children().iter().any(|field| {
        field.tag().raw() == raw_tag
            || field.with_value(|value| match value {
                ValueView::Structure(nested) => structure_contains_tag(&nested, raw_tag),
                _ => false,
            })
    })
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw_tag)
            .expect("the source-derived tag fits TTLV width")
            .try_checked()
            .expect("the source-derived tag is allocated"),
        value,
    )
    .expect("the source-derived tag and value form a valid TTLV item")
}

fn certificate_type_attributes() -> Structure {
    structure([item(CERTIFICATE_TYPE, Value::enumeration(1))])
}

#[test]
fn request_emits_table_179_fields_once_and_in_order() {
    let request_value = [0x00, 0x7F, 0x80, 0xFF];
    let request = CertifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("public-key".to_owned()))
        .with_certificate_request_type(1)
        .with_certificate_request_value(crate::SecretBytes::new(request_value.to_vec()))
        .with_attributes(certificate_type_attributes())
        .with_protection_storage_masks(Structure::new());

    let payload = request
        .into_ttlv_payload()
        .expect("the Table 179 fields form one representable Certify request");

    assert_eq!(
        field_tags(&payload),
        [
            UNIQUE_IDENTIFIER,
            CERTIFICATE_REQUEST_TYPE,
            CERTIFICATE_REQUEST_VALUE,
            ATTRIBUTES,
            PROTECTION_STORAGE_MASKS,
        ],
        "Table 179 has one Certificate Request Value field, so a request cannot fan out into multiple certificates"
    );
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert!(fields[2].with_value(|value| {
        matches!(value, ValueView::ByteString(bytes) if bytes == request_value.as_slice())
    }));
}

#[test]
fn request_may_omit_unique_identifier_when_certificate_request_value_is_present() {
    let request = CertifyRequest::new()
        .with_certificate_request_type(1)
        .with_certificate_request_value(crate::SecretBytes::new(
            b"request-without-public-key-id".to_vec(),
        ));

    let payload = request
        .into_ttlv_payload()
        .expect("a Certificate Request Value can identify the request without a public-key Unique Identifier");

    assert_eq!(
        field_tags(&payload),
        [CERTIFICATE_REQUEST_TYPE, CERTIFICATE_REQUEST_VALUE]
    );
}

#[test]
fn request_requires_certificate_request_type_when_request_value_is_present() {
    let request = CertifyRequest::new().with_certificate_request_value(crate::SecretBytes::new(
        b"certificate-request-sentinel".to_vec(),
    ));

    assert!(
        request.into_ttlv_payload().is_err(),
        "Table 179 requires Certificate Request Type whenever Certificate Request Value is present"
    );
}

#[test]
fn request_without_request_value_requires_public_key_identifier_and_certificate_type_attribute() {
    let valid = CertifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("public-key".to_owned()))
        .with_attributes(certificate_type_attributes())
        .into_ttlv_payload()
        .expect(
            "§6.1.6 permits omitting request material with the public key ID and Certificate Type",
        );
    assert_eq!(field_tags(&valid), [UNIQUE_IDENTIFIER, ATTRIBUTES]);

    let missing_identifier = CertifyRequest::new()
        .with_attributes(certificate_type_attributes())
        .into_ttlv_payload();
    assert!(
        missing_identifier.is_err(),
        "§6.1.6 requires the public key Unique Identifier when request material is omitted"
    );

    let missing_certificate_type = CertifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("public-key".to_owned()))
        .into_ttlv_payload();
    assert!(
        missing_certificate_type.is_err(),
        "§6.1.6 requires Certificate Type in Attributes when both request fields are omitted"
    );
}

#[test]
fn successful_response_returns_required_identifier_and_keeps_unknown_fields() {
    let generated_identifier = UniqueIdentifier::TextString("generated-certificate".to_owned());
    let payload = structure([
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string("generated-certificate".to_owned()),
        ),
        item(UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ]);
    let message = response_message(CERTIFY_OPERATION, SUCCESS, None, None, Some(payload));
    let response = CertifyResponse::try_from_response_item(response_item(&message))
        .expect("Table 180 success contains its required generated-certificate identifier");

    assert_eq!(response.result().status().raw(), SUCCESS);
    assert_eq!(response.unique_identifier(), Some(&generated_identifier));
    drop(message);
    assert_eq!(
        response.with_ttlv(|view| structure_contains_tag(&view, UNKNOWN_VENDOR_TAG)),
        Some(true)
    );
}

#[test]
fn successful_response_rejects_missing_or_duplicate_unique_identifier() {
    for payload in [
        Structure::new(),
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            item(UNIQUE_IDENTIFIER, Value::text_string("second".to_owned())),
        ]),
    ] {
        let message = response_message(CERTIFY_OPERATION, SUCCESS, None, None, Some(payload));
        assert!(
            CertifyResponse::try_from_response_item(response_item(&message)).is_err(),
            "Table 180 requires exactly one well-formed Unique Identifier"
        );
    }
}

#[test]
fn failure_response_preserves_the_table_181_reason_without_success_payload() {
    let message = response_message(
        CERTIFY_OPERATION,
        OPERATION_FAILED,
        Some(INVALID_CSR),
        None,
        None,
    );
    let response = CertifyResponse::try_from_response_item(response_item(&message))
        .expect("Table 181 Operation Failed remains a common KMIP result");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(crate::ResultReason::raw),
        Some(INVALID_CSR)
    );
    assert!(response.unique_identifier().is_none());
    assert!(response.with_ttlv(|_| ()).is_none());
}

#[test]
fn request_debug_redacts_certificate_request_value() {
    let sentinel = b"CERTIFY_REQUEST_SECRET_SENTINEL";
    let request = CertifyRequest::new()
        .with_certificate_request_value(crate::SecretBytes::new(sentinel.to_vec()));

    assert!(!format!("{request:?}").contains("CERTIFY_REQUEST_SECRET_SENTINEL"));
}
