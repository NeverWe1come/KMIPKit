//! Revoke contracts from OASIS KMIP v2.1 §6.1.44, Tables 295–297, and §11.48, Table 481.
//! Traceability: `KMIPKIT-REQ-SPEC-6.1.44-001`, `-003-001`, and `-003-002`.
//! These vectors are derived from the pinned tables, not official fixture runs.

use crate::async_operation_fixtures::{response_message, structure};
use crate::{
    ResponseBatchItemView, ResponseMessage, ResultReason, RevocationReason, RevokeError,
    RevokeRequest, RevokeResponse, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

const REVOKE_OPERATION: u32 = 0x0000_0011;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OBJECT_NOT_FOUND: u32 = 0x0000_0037;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const REVOCATION_REASON: u32 = 0x0042_0081;
const REVOCATION_MESSAGE: u32 = 0x0042_0080;
const REVOCATION_REASON_CODE: u32 = 0x0042_0082;
const COMPROMISE_OCCURRENCE_DATE: u32 = 0x0042_0021;
const KEY_COMPROMISE: u32 = 2;
const CA_COMPROMISE: u32 = 3;
const UNKNOWN_REASON: u32 = 0x8000_0001;

fn tag(raw_tag: u32) -> Tag {
    RawTag::new(raw_tag)
        .expect("source-derived tag fits TTLV width")
        .try_checked()
        .expect("source-derived tag is allocated")
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("fixture item is structurally valid")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one validated response batch item")
}

fn typed_response(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<RevokeResponse, RevokeError> {
    let message = response_message(operation, status, reason, None, payload);
    RevokeResponse::try_from_response_item(response_item(&message))
}

fn request(reason: u32, date: Option<i64>) -> RevokeRequest {
    let request = RevokeRequest::new(RevocationReason::from_raw(reason))
        .with_unique_identifier(UniqueIdentifier::TextString("object-id".to_owned()));
    match date {
        Some(date) => request.with_compromise_occurrence_date(date),
        None => request,
    }
}

fn field_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

fn reason_fields(payload: &Structure) -> (Vec<u32>, Option<u32>, Option<String>) {
    let view = payload.view();
    let reason = view
        .children()
        .iter()
        .find(|field| field.tag().raw() == REVOCATION_REASON)
        .expect("Table 295 contains the required Revocation Reason");
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
}

#[test]
fn request_emits_table_295_fields_in_order_and_preserves_open_reason() {
    let payload = request(UNKNOWN_REASON, None)
        .into_ttlv_payload()
        .expect("unknown open reason remains valid when no compromise date is supplied");
    assert_eq!(field_tags(&payload), [UNIQUE_IDENTIFIER, REVOCATION_REASON]);
    assert_eq!(
        reason_fields(&payload),
        (vec![REVOCATION_REASON_CODE], Some(UNKNOWN_REASON), None)
    );
}

#[test]
fn request_encodes_optional_audit_message_inside_revocation_reason_structure() {
    let payload = RevokeRequest::new(RevocationReason::KEY_COMPROMISE)
        .with_revocation_message("audit explanation".to_owned())
        .into_ttlv_payload()
        .expect("§4.47 Table 123 permits Revocation Message");
    assert_eq!(
        reason_fields(&payload),
        (
            vec![REVOCATION_REASON_CODE, REVOCATION_MESSAGE],
            Some(KEY_COMPROMISE),
            Some("audit explanation".to_owned())
        )
    );
}

#[test]
fn request_allows_compromise_date_only_for_key_or_ca_compromise() {
    for reason in [KEY_COMPROMISE, CA_COMPROMISE] {
        let payload = request(reason, Some(1_700_000_000))
            .into_ttlv_payload()
            .expect("Table 295 permits the date for key or CA compromise");
        assert_eq!(
            field_tags(&payload),
            [
                UNIQUE_IDENTIFIER,
                REVOCATION_REASON,
                COMPROMISE_OCCURRENCE_DATE
            ]
        );
    }
}

#[test]
fn request_rejects_compromise_date_for_every_other_reason() {
    for reason in [1, 4, 7, UNKNOWN_REASON] {
        assert!(
            request(reason, Some(1_700_000_000))
                .into_ttlv_payload()
                .is_err(),
            "§6.1.44 SHALL NOT allow Compromise Occurrence Date for reason {reason}"
        );
    }
}

#[test]
fn request_can_omit_identifier_for_batch_id_placeholder() {
    let payload = RevokeRequest::new(RevocationReason::from_raw(KEY_COMPROMISE))
        .into_ttlv_payload()
        .expect("Table 295 allows Unique Identifier omission");
    assert_eq!(field_tags(&payload), [REVOCATION_REASON]);
}

#[test]
fn request_debug_does_not_expose_identifier_or_request_values() {
    let request = RevokeRequest::new(RevocationReason::from_raw(KEY_COMPROMISE))
        .with_unique_identifier(UniqueIdentifier::TextString(
            "revoke-secret-sentinel".to_owned(),
        ))
        .with_revocation_message("revoke-message-sentinel".to_owned())
        .with_compromise_occurrence_date(1_700_000_000);
    let debug = format!("{request:?}");
    assert!(!debug.contains("revoke-secret-sentinel"));
    assert!(!debug.contains("revoke-message-sentinel"));
}

#[test]
fn successful_response_requires_and_preserves_unique_identifier() {
    let response = typed_response(
        REVOKE_OPERATION,
        SUCCESS,
        None,
        Some(structure([item(
            UNIQUE_IDENTIFIER,
            Value::text_string("revoked-object".to_owned()),
        )])),
    )
    .expect("Table 296 success includes a Unique Identifier");
    assert_eq!(response.result().status().raw(), SUCCESS);
    assert_eq!(
        response.unique_identifier(),
        Some(&UniqueIdentifier::TextString("revoked-object".to_owned()))
    );
}

#[test]
fn successful_response_rejects_missing_or_duplicate_identifier() {
    assert!(typed_response(REVOKE_OPERATION, SUCCESS, None, Some(structure([]))).is_err());
    assert!(
        typed_response(
            REVOKE_OPERATION,
            SUCCESS,
            None,
            Some(structure([
                item(UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
                item(UNIQUE_IDENTIFIER, Value::text_string("second".to_owned())),
            ]))
        )
        .is_err()
    );
}

#[test]
fn failure_preserves_common_result_without_success_identifier() {
    let response = typed_response(
        REVOKE_OPERATION,
        OPERATION_FAILED,
        Some(OBJECT_NOT_FOUND),
        None,
    )
    .expect("Table 297 failure remains a common KMIP result");
    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(OBJECT_NOT_FOUND)
    );
    assert!(response.unique_identifier().is_none());
}

#[test]
fn response_conversion_rejects_a_different_operation() {
    let error = typed_response(0x0000_0012, OPERATION_FAILED, Some(OBJECT_NOT_FOUND), None)
        .expect_err("Activate is not Revoke");
    assert_eq!(error, RevokeError::UnexpectedOperation);
}
