//! Destroy payload tests derived from OASIS KMIP Specification v2.1 §6.1.15,
//! Tables 208–210, and §11.56. These are source-derived tests, not official
//! OASIS Test Cases.
//!
//! Traceability: `KMIPKIT-ELEM-OP-C2S-DESTROY`; KMIPKIT-0018 FR-001, FR-002,
//! FR-003, FR-006, FR-008, FR-009; SC-001 and SC-002.

use crate::async_operation_fixtures::response_message;
use crate::lifecycle_fixtures::{
    request_payload, success_payload_duplicate_identifier, success_payload_missing_identifier,
    success_payload_wrong_identifier_type, successful_response_payload,
};
use crate::{
    DestroyError, DestroyRequest, DestroyResponse, ResponseBatchItemView, ResponseMessage,
    ResultReason, ResultStatus, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, Structure, ValueView};

const ACTIVATE_OPERATION: u32 = 0x0000_0012;
const DESTROY_OPERATION: u32 = 0x0000_0014;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
// Table 210 maps Object Not Found to catalog record
// KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-NOT-FOUND-00000037.
const OBJECT_NOT_FOUND_RESULT_REASON: u32 = 0x0000_0037;

fn identifier_forms() -> [UniqueIdentifier; 3] {
    [
        UniqueIdentifier::TextString("object-identifier".to_owned()),
        UniqueIdentifier::Enumeration(0xA1B2_C3D4),
        UniqueIdentifier::Integer(-12_345),
    ]
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one validated response batch item")
}

fn assert_identifier_field_matches_fixture(actual: &Item, expected: &Item) {
    assert_eq!(actual.tag().raw(), 0x0042_0094);
    assert_eq!(actual.tag().raw(), expected.tag().raw());
    assert_eq!(actual.item_type(), expected.item_type());
    assert!(actual.with_value(|actual_value| {
        expected.with_value(|expected_value| match (actual_value, expected_value) {
            (ValueView::TextString(actual), ValueView::TextString(expected)) => actual == expected,
            (ValueView::Enumeration(actual), ValueView::Enumeration(expected)) => {
                actual == expected
            }
            (ValueView::Integer(actual), ValueView::Integer(expected)) => actual == expected,
            _ => false,
        })
    }));
}

fn assert_identifier_payload_matches_fixture(actual: &Structure, expected: &Structure) {
    let actual_view = actual.view();
    let expected_view = expected.view();
    let actual_fields = actual_view.children();
    let expected_fields = expected_view.children();

    assert_eq!(actual_fields.len(), 1);
    assert_eq!(expected_fields.len(), 1);
    assert_identifier_field_matches_fixture(&actual_fields[0], &expected_fields[0]);
}

fn malformed_success_result(payload: Structure) -> Result<DestroyResponse, DestroyError> {
    let message = response_message(DESTROY_OPERATION, SUCCESS, None, None, Some(payload));
    DestroyResponse::try_from_response_item(response_item(&message))
}

#[test]
fn request_omits_the_optional_unique_identifier() {
    let request = DestroyRequest::new(None);
    let actual = request
        .to_ttlv_payload()
        .expect("an omitted optional identifier forms a valid Destroy payload");
    let expected = request_payload(None);

    assert!(actual.view().children().is_empty());
    assert!(expected.view().children().is_empty());
}

#[test]
fn request_preserves_each_supported_unique_identifier_wire_form() {
    for identifier in identifier_forms() {
        let expected = request_payload(Some(identifier.clone()));
        let actual = DestroyRequest::new(Some(identifier))
            .to_ttlv_payload()
            .expect("Table 208 permits a supported Unique Identifier wire form");

        assert_identifier_payload_matches_fixture(&actual, &expected);
    }
}

#[test]
fn successful_response_returns_the_required_identifier_without_normalizing_its_wire_form() {
    for identifier in identifier_forms() {
        let payload = successful_response_payload(identifier.clone());
        let message = response_message(DESTROY_OPERATION, SUCCESS, None, None, Some(payload));
        let response = DestroyResponse::try_from_response_item(response_item(&message))
            .expect("Table 209 success contains its required Unique Identifier");

        assert_eq!(response.result().status(), ResultStatus::from_raw(SUCCESS));
        assert_eq!(response.unique_identifier(), Some(&identifier));
    }
}

#[test]
fn successful_response_rejects_a_missing_unique_identifier() {
    assert!(matches!(
        malformed_success_result(success_payload_missing_identifier()),
        Err(DestroyError::MalformedSuccessPayload)
    ));
}

#[test]
fn successful_response_rejects_a_wrong_item_type_for_unique_identifier() {
    assert!(matches!(
        malformed_success_result(success_payload_wrong_identifier_type()),
        Err(DestroyError::MalformedSuccessPayload)
    ));
}

#[test]
fn successful_response_rejects_duplicate_unique_identifiers() {
    let identifier = UniqueIdentifier::TextString("object-identifier".to_owned());
    assert!(matches!(
        malformed_success_result(success_payload_duplicate_identifier(identifier)),
        Err(DestroyError::MalformedSuccessPayload)
    ));
}

#[test]
fn failure_response_preserves_object_not_found_without_a_typed_success_identifier() {
    let message = response_message(
        DESTROY_OPERATION,
        OPERATION_FAILED,
        Some(OBJECT_NOT_FOUND_RESULT_REASON),
        None,
        None,
    );
    let response = DestroyResponse::try_from_response_item(response_item(&message))
        .expect("a valid non-success response remains an operation result");

    assert_eq!(
        response.result().status(),
        ResultStatus::from_raw(OPERATION_FAILED)
    );
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(OBJECT_NOT_FOUND_RESULT_REASON)
    );
    assert!(response.unique_identifier().is_none());
}

#[test]
fn failure_response_preserves_an_unknown_result_reason_for_fr008() {
    // KMIPKIT-0018 FR-008 requires unknown values to remain unchanged.
    let unknown_reason = 0xF001_0001;
    let message = response_message(
        DESTROY_OPERATION,
        OPERATION_FAILED,
        Some(unknown_reason),
        None,
        None,
    );
    let response = DestroyResponse::try_from_response_item(response_item(&message))
        .expect("an unknown Result Reason remains a valid non-success result");

    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(unknown_reason)
    );
}

#[test]
fn response_rejects_a_different_operation() {
    let payload =
        successful_response_payload(UniqueIdentifier::TextString("object-identifier".to_owned()));
    let message = response_message(ACTIVATE_OPERATION, SUCCESS, None, None, Some(payload));

    assert!(matches!(
        DestroyResponse::try_from_response_item(response_item(&message)),
        Err(DestroyError::UnexpectedOperation)
    ));
}
