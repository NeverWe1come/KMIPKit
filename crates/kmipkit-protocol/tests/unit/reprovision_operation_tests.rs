//! Re-Provision contracts from OASIS KMIP v2.1 §6.1.48, Tables 313–315.
//! Traceability: `KMIPKIT-REQ-SPEC-6.1.48-001`, `-004`, and `-006`.
//! `KMIPKIT-DISC-040` approves all three request forms and retains Table 315's
//! source caption typo without treating it as another operation. These are
//! source-derived tests, not official OASIS Test Case executions.

use crate::async_operation_fixtures::{response_message, structure};
use crate::{
    ReProvisionError, ReProvisionRequest, ReProvisionResponse, ResponseBatchItemView,
    ResponseMessage, ResultReason, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

const REPROVISION_OPERATION: u32 = 0x0000_0035;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const PERMISSION_DENIED: u32 = 0x0000_0005;
const CERTIFICATE_REQUEST: u32 = 0x0042_0018;
const CERTIFICATE: u32 = 0x0042_0013;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const NESTED_VALUE: u32 = 0x0054_1234;

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one validated response batch item")
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw_tag)
            .expect("source-derived tag fits TTLV width")
            .try_checked()
            .expect("source-derived tag is allocated"),
        value,
    )
    .expect("source-derived tag and value form a valid item")
}

fn field_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

fn certificate_material(sentinel: &str) -> Structure {
    structure([item(
        NESTED_VALUE,
        Value::byte_string(sentinel.as_bytes().to_vec()),
    )])
}

fn deeply_nested_structure() -> Structure {
    let mut nested = Structure::new();
    for _ in 1..64 {
        let mut parent = Structure::new();
        parent
            .try_push(item(NESTED_VALUE, Value::structure(nested)))
            .expect("each level remains within the model depth bound");
        nested = parent;
    }
    nested
}

fn typed_response(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<ReProvisionResponse, ReProvisionError> {
    let message = response_message(REPROVISION_OPERATION, status, reason, None, payload);
    ReProvisionResponse::try_from_response_item(response_item(&message))
}

#[test]
fn request_supports_certificate_request_certificate_or_no_parameters() {
    let csr = ReProvisionRequest::new()
        .with_certificate_request(certificate_material("csr-marker"))
        .into_ttlv_payload()
        .expect("Table 313 permits a certificate request");
    let certificate = ReProvisionRequest::new()
        .with_certificate(certificate_material("certificate-marker"))
        .into_ttlv_payload()
        .expect("Table 313 permits a certificate");
    let generated = ReProvisionRequest::new()
        .into_ttlv_payload()
        .expect("§6.1.48 permits server-created credentials with no parameters");

    assert_eq!(field_tags(&csr), [CERTIFICATE_REQUEST]);
    assert_eq!(field_tags(&certificate), [CERTIFICATE]);
    assert_eq!(field_tags(&generated), Vec::<u32>::new());
}

#[test]
fn request_preserves_both_optional_certificate_fields() {
    let payload = ReProvisionRequest::new()
        .with_certificate_request(certificate_material("csr-marker"))
        .with_certificate(certificate_material("certificate-marker"))
        .into_ttlv_payload()
        .expect("§6.1.48 and Table 313 do not prohibit both optional fields");

    assert_eq!(field_tags(&payload), [CERTIFICATE_REQUEST, CERTIFICATE]);
}

#[test]
fn request_rejects_certificate_material_beyond_model_depth() {
    let result = ReProvisionRequest::new()
        .with_certificate(deeply_nested_structure())
        .into_ttlv_payload();

    assert!(result.is_err());
}

#[test]
fn successful_response_preserves_optional_identifier_and_absence() {
    let response = typed_response(
        SUCCESS,
        None,
        Some(structure([item(
            UNIQUE_IDENTIFIER,
            Value::text_string("reprovisioned-credential".to_owned()),
        )])),
    )
    .expect("Table 314 response with Unique Identifier is valid");
    assert_eq!(response.result().status().raw(), SUCCESS);
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString(
            "reprovisioned-credential".to_owned()
        ))
    );
    assert!(response.with_ttlv(|view| !view.children().is_empty()));

    let absent = typed_response(SUCCESS, None, Some(structure([])))
        .expect("Table 314 marks Unique Identifier optional");
    assert_eq!(absent.unique_identifier(), None);
}

#[test]
fn successful_response_rejects_a_duplicate_unique_identifier() {
    let result = typed_response(
        SUCCESS,
        None,
        Some(structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            item(UNIQUE_IDENTIFIER, Value::text_string("second".to_owned())),
        ])),
    );

    assert!(matches!(
        result,
        Err(ReProvisionError::MalformedSuccessPayload)
    ));
}

#[test]
fn successful_response_rejects_a_mistyped_unique_identifier() {
    let result = typed_response(
        SUCCESS,
        None,
        Some(structure([item(
            UNIQUE_IDENTIFIER,
            Value::byte_string(b"not-an-identifier".to_vec()),
        )])),
    );

    assert!(result.is_err());
}

#[test]
fn failure_preserves_table_315_common_result_without_success_values() {
    let response = typed_response(OPERATION_FAILED, Some(PERMISSION_DENIED), None)
        .expect("Table 315 failure remains a common KMIP operation result");
    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(PERMISSION_DENIED)
    );
    assert!(response.unique_identifier().is_none());
}

#[test]
fn request_debug_redacts_certificate_and_request_contents() {
    let csr_marker = "reprovision-csr-debug-secret-marker";
    let certificate_marker = "reprovision-certificate-debug-secret-marker";
    let request = ReProvisionRequest::new()
        .with_certificate_request(certificate_material(csr_marker))
        .with_certificate(certificate_material(certificate_marker));
    let debug = format!("{request:?}");

    assert!(!debug.contains(csr_marker));
    assert!(!debug.contains(certificate_marker));
}

#[test]
fn response_conversion_rejects_a_different_operation() {
    let message = response_message(0x0000_0004, OPERATION_FAILED, Some(0x0000_0037), None, None);
    assert!(ReProvisionResponse::try_from_response_item(response_item(&message)).is_err());
}
