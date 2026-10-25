//! Re-certify contracts derived from OASIS KMIP Specification v2.1 §6.1.45,
//! Tables 298–302. Traceability: `KMIPKIT-REQ-SPEC-6.1.45-001`, `-002-001`,
//! `-002-002`, `-004`, `-006-001`, `-006-002`, and `-008`. These are derived
//! tests, not official OASIS Test Case executions.

use crate::async_operation_fixtures::{response_message, structure};
use crate::{
    RecertifyError, RecertifyRequest, RecertifyResponse, ResponseBatchItemView, ResponseMessage,
    ResultReason, SecretBytes, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Value, ValueView};

const RECERTIFY_OPERATION: u32 = 0x0000_0007;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const INVALID_CSR: u32 = 0x0000_002F;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CERTIFICATE_REQUEST_UNIQUE_IDENTIFIER: u32 = 0x0042_0139;
const CERTIFICATE_REQUEST_TYPE: u32 = 0x0042_0019;
const CERTIFICATE_REQUEST_VALUE: u32 = 0x0042_0140;
const OFFSET: u32 = 0x0042_0058;
const ATTRIBUTES: u32 = 0x0042_0125;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const CERTIFICATE_TYPE: u32 = 0x0042_001D;
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

fn typed_response(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<RecertifyResponse, RecertifyError> {
    let message = response_message(RECERTIFY_OPERATION, status, reason, None, payload);
    RecertifyResponse::try_from_response_item(response_item(&message))
}

#[test]
fn request_emits_table_300_fields_once_in_order_and_preserves_values() {
    let identifier = UniqueIdentifier::TextString("existing-certificate".to_owned());
    let request_identifier = UniqueIdentifier::Integer(17);
    let request_value = b"opaque-certificate-request";
    let attributes = structure([item(CERTIFICATE_TYPE, Value::enumeration(1))]);
    let masks = structure([item(0x0042_0160, Value::integer(3))]);
    let request = RecertifyRequest::new()
        .with_unique_identifier(identifier.clone())
        .with_certificate_request_unique_identifier(request_identifier.clone())
        .with_certificate_request_type(1)
        .with_certificate_request_value(SecretBytes::new(request_value.to_vec()))
        .with_offset(86_400)
        .with_attributes(attributes)
        .with_protection_storage_masks(masks);

    assert_eq!(request.unique_identifier(), Some(&identifier));
    assert_eq!(
        request.certificate_request_unique_identifier(),
        Some(&request_identifier)
    );
    let payload = request
        .into_ttlv_payload()
        .expect("the complete Table 300 request form is valid");

    assert_eq!(
        field_tags(&payload),
        [
            UNIQUE_IDENTIFIER,
            CERTIFICATE_REQUEST_UNIQUE_IDENTIFIER,
            CERTIFICATE_REQUEST_TYPE,
            CERTIFICATE_REQUEST_VALUE,
            OFFSET,
            ATTRIBUTES,
            PROTECTION_STORAGE_MASKS,
        ]
    );
    assert_eq!(
        payload.view().children()[3].with_value(|value| match value {
            ValueView::ByteString(bytes) => Some(bytes.to_vec()),
            _ => None,
        }),
        Some(request_value.to_vec())
    );
    assert_eq!(
        payload.view().children()[4].with_value(|value| match value {
            ValueView::Interval(value) => Some(*value),
            _ => None,
        }),
        Some(86_400)
    );
}

#[test]
fn request_requires_type_when_certificate_request_value_is_present() {
    let request = RecertifyRequest::new()
        .with_certificate_request_value(SecretBytes::new(b"opaque-request".to_vec()));

    assert!(request.into_ttlv_payload().is_err());
}

#[test]
fn request_requires_unique_identifier_when_certificate_request_value_is_omitted() {
    assert!(RecertifyRequest::new().into_ttlv_payload().is_err());
}

#[test]
fn request_rejects_id_placeholder_when_certificate_request_value_is_omitted() {
    let request = RecertifyRequest::new().with_unique_identifier(UniqueIdentifier::Enumeration(1));

    assert!(request.into_ttlv_payload().is_err());
}

#[test]
fn request_preserves_optional_identifier_when_certificate_request_value_is_present() {
    let payload = RecertifyRequest::new()
        .with_certificate_request_type(1)
        .with_certificate_request_value(SecretBytes::new(b"opaque-request".to_vec()))
        .into_ttlv_payload()
        .expect("Certificate Request Value permits an omitted Unique Identifier");

    assert_eq!(
        field_tags(&payload),
        [CERTIFICATE_REQUEST_TYPE, CERTIFICATE_REQUEST_VALUE]
    );
}

#[test]
fn request_preserves_id_placeholder_when_certificate_request_value_is_present() {
    let payload = RecertifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::Enumeration(1))
        .with_certificate_request_type(1)
        .with_certificate_request_value(SecretBytes::new(b"opaque-request".to_vec()))
        .into_ttlv_payload()
        .expect("the omission-specific rule does not constrain this request form");

    assert_eq!(
        field_tags(&payload),
        [
            UNIQUE_IDENTIFIER,
            CERTIFICATE_REQUEST_TYPE,
            CERTIFICATE_REQUEST_VALUE
        ]
    );
}

#[test]
fn request_can_omit_request_fields_to_reuse_existing_certificate_type() {
    let payload = RecertifyRequest::new()
        .into_ttlv_payload()
        .expect("§6.1.45 reuses the existing type when request fields are omitted");

    assert!(payload.view().children().is_empty());
}

#[test]
fn successful_response_returns_new_identifier_and_preserves_unknown_fields() {
    let payload = structure([
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string("replacement-certificate".to_owned()),
        ),
        item(UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ]);
    let response = typed_response(SUCCESS, None, Some(payload))
        .expect("Table 301 success includes the new certificate identifier");

    assert_eq!(response.result().status().raw(), SUCCESS);
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString(
            "replacement-certificate".to_owned()
        ))
    );
    assert!(response.with_ttlv(|view| structure_contains_tag(&view, UNKNOWN_VENDOR_TAG)));
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
        assert!(
            typed_response(SUCCESS, None, Some(payload)).is_err(),
            "Table 301 success requires exactly one Unique Identifier"
        );
    }
}

#[test]
fn failure_preserves_table_302_reason_without_success_identifier() {
    let response = typed_response(OPERATION_FAILED, Some(INVALID_CSR), None)
        .expect("Table 302 Operation Failed remains a common KMIP result");

    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(INVALID_CSR)
    );
    assert!(response.unique_identifier().is_none());
}

#[test]
fn request_debug_does_not_expose_certificate_request_value() {
    let sentinel = b"RECERTIFY_REQUEST_SECRET_SENTINEL";
    let request = RecertifyRequest::new()
        .with_certificate_request_type(1)
        .with_certificate_request_value(SecretBytes::new(sentinel.to_vec()));

    assert!(!format!("{request:?}").contains("RECERTIFY_REQUEST_SECRET_SENTINEL"));
}
