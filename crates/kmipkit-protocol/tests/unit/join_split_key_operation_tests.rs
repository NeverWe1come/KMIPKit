//! Join Split Key request/result contracts from OASIS KMIP v2.1 §6.1.27, Tables 244–246.
//! These local vectors are table-derived and are not official OASIS test-case executions.

use std::error::Error as _;

use crate::{
    AttributeSet, JoinSplitKeyError, JoinSplitKeyRequest, JoinSplitKeyResponse, ObjectType,
    ResultReason, ResultValidationError, SplitKeyReference, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

const JOIN_SPLIT_KEY_OPERATION: u32 = 0x0000_0029;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const SECRET_DATA_TYPE: u32 = 0x0042_0086;
const ATTRIBUTES: u32 = 0x0042_0125;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const SPLIT_KEY_THRESHOLD: u32 = 0x0042_008C;
const FUTURE_RESULT_REASON: u32 = 0xF123_4567;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits TTLV")
        .try_checked()
        .expect("fixture tag is allocated")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("fixture item is valid")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut value = Structure::new();
    for child in items {
        value.try_push(child).expect("fixture structure is valid");
    }
    value
}

fn identifier(value: &str) -> UniqueIdentifier {
    UniqueIdentifier::TextString(value.to_owned())
}

fn reference(value: &str, threshold: u32) -> SplitKeyReference {
    SplitKeyReference::new(identifier(value)).with_split_key_threshold(threshold)
}

fn request(references: Vec<SplitKeyReference>) -> JoinSplitKeyRequest {
    JoinSplitKeyRequest::new(ObjectType::from_raw(7), references)
}

fn response_message(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> crate::ResponseMessage {
    response_message_for_operation(JOIN_SPLIT_KEY_OPERATION, status, reason, payload)
}

fn response_message_for_operation(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> crate::ResponseMessage {
    let version = structure([
        item(0x0042_006A, Value::integer(2)),
        item(0x0042_006B, Value::integer(1)),
    ]);
    let header = structure([
        item(0x0042_0069, Value::structure(version)),
        item(0x0042_0092, Value::date_time(1)),
        item(0x0042_000D, Value::integer(1)),
    ]);
    let mut fields = vec![
        item(0x0042_005C, Value::enumeration(operation)),
        item(0x0042_007F, Value::enumeration(status)),
    ];
    if let Some(reason) = reason {
        fields.push(item(0x0042_007E, Value::enumeration(reason)));
    }
    if let Some(payload) = payload {
        fields.push(item(0x0042_007C, Value::structure(payload)));
    }
    crate::ResponseMessage::try_from_ttlv(structure([
        item(0x0042_007A, Value::structure(header)),
        item(0x0042_000F, Value::structure(structure(fields))),
    ]))
    .expect("fixture is a valid KMIP 2.1 response")
}

fn response_item(message: &crate::ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture has one response batch item")
}

#[test]
fn missing_or_misaligned_reference_threshold_context_is_rejected() {
    let request = request(vec![
        reference("split-a", 2),
        SplitKeyReference::new(identifier("split-b")),
    ]);
    assert!(request.into_ttlv_payload().is_err());
    assert!(
        JoinSplitKeyRequest::new(ObjectType::from_raw(7), Vec::new())
            .into_ttlv_payload()
            .is_err()
    );
}

#[test]
fn entry_count_must_meet_the_maximum_threshold_and_thresholds_may_differ() {
    let below = request(vec![reference("a", 3), reference("b", 1)]);
    assert!(below.into_ttlv_payload().is_err());

    let equal = request(vec![
        reference("a", 2),
        reference("b", 3),
        reference("c", 1),
    ]);
    assert!(equal.into_ttlv_payload().is_ok());

    let above = request(vec![
        reference("a", 2),
        reference("b", 3),
        reference("c", 1),
        reference("d", 2),
    ]);
    assert!(above.into_ttlv_payload().is_ok());
}

#[test]
fn request_preserves_repeated_identifiers_in_order_and_keeps_thresholds_off_wire() {
    let payload = request(vec![
        reference("split-a", 2),
        reference("split-b", 3),
        reference("split-a", 1),
    ])
    .with_secret_data_type(0x8123_4568)
    .with_attributes(
        AttributeSet::try_new([item(0x0042_0028, Value::enumeration(3))])
            .expect("fixture attribute is valid"),
    )
    .with_protection_storage_masks(structure([item(0x0042_015E, Value::integer(1))]))
    .into_ttlv_payload()
    .expect("three identifier occurrences meet maximum threshold three");

    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(fields[0].tag().raw(), OBJECT_TYPE);
    let identifiers = fields
        .iter()
        .filter(|field| field.tag().raw() == UNIQUE_IDENTIFIER)
        .map(|field| {
            field.with_value(|value| match value {
                ValueView::TextString(value) => (*value).to_owned(),
                _ => panic!("Unique Identifier remains a Text String"),
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(identifiers, ["split-a", "split-b", "split-a"]);
    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        vec![
            OBJECT_TYPE,
            UNIQUE_IDENTIFIER,
            UNIQUE_IDENTIFIER,
            UNIQUE_IDENTIFIER,
            SECRET_DATA_TYPE,
            ATTRIBUTES,
            PROTECTION_STORAGE_MASKS,
        ]
    );
    assert!(
        !fields
            .iter()
            .any(|field| field.tag().raw() == SPLIT_KEY_THRESHOLD)
    );
    assert!(
        fields[4].with_value(
            |value| matches!(value, ValueView::Enumeration(raw) if *raw == 0x8123_4568)
        )
    );
}

#[test]
fn public_request_views_preserve_local_thresholds_and_optional_fields() {
    let first = SplitKeyReference::new(UniqueIdentifier::Enumeration(0xF123_4567))
        .with_split_key_threshold(1);
    assert_eq!(
        first.unique_identifier(),
        &UniqueIdentifier::Enumeration(0xF123_4567)
    );
    assert_eq!(first.split_key_threshold(), Some(1));
    assert!(!format!("{first:?}").contains("F1234567"));

    let attributes = AttributeSet::try_new([item(0x0042_0028, Value::enumeration(3))])
        .expect("fixture attribute is valid");
    let request = JoinSplitKeyRequest::new(
        ObjectType::from_raw(7),
        vec![first, reference("split-b", 2)],
    )
    .with_secret_data_type(0x8123_4568)
    .with_attributes(attributes)
    .with_protection_storage_masks(structure([item(0x0042_015E, Value::integer(2))]));
    assert_eq!(request.object_type(), ObjectType::from_raw(7));
    assert_eq!(request.references().len(), 2);
    assert_eq!(request.references()[1].split_key_threshold(), Some(2));
    assert_eq!(request.secret_data_type(), Some(0x8123_4568));
    assert_eq!(request.attributes().map(AttributeSet::len), Some(1));
    assert!(request.protection_storage_masks().is_some());
    let payload = request
        .into_ttlv_payload()
        .expect("count two satisfies both explicit per-entry thresholds");
    assert_eq!(payload.view().children()[1].tag().raw(), UNIQUE_IDENTIFIER);
}

#[test]
fn success_returns_identifier_and_failure_preserves_unknown_reason() {
    let success = response_message(
        SUCCESS,
        None,
        Some(structure([
            item(0x0054_1234, Value::integer(5)),
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string("joined-key".to_owned()),
            ),
        ])),
    );
    let response = JoinSplitKeyResponse::try_from_response_item(response_item(&success))
        .expect("Table 245 success returns the joined object's identifier");
    assert_eq!(
        response.unique_identifier(),
        Some(&identifier("joined-key"))
    );

    let failure = response_message(OPERATION_FAILED, Some(FUTURE_RESULT_REASON), None);
    let response = JoinSplitKeyResponse::try_from_response_item(response_item(&failure))
        .expect("Table 246 operation errors remain typed results");
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(FUTURE_RESULT_REASON)
    );
    assert!(response.unique_identifier().is_none());
    assert!(response.with_ttlv(|ttlv| !ttlv.children().is_empty()));
    assert!(format!("{response:?}").contains("JoinSplitKeyResponse"));
}

#[test]
fn malformed_success_payload_is_rejected() {
    let malformed = response_message(SUCCESS, None, Some(structure([])));
    assert!(matches!(
        JoinSplitKeyResponse::try_from_response_item(response_item(&malformed)),
        Err(JoinSplitKeyError::MalformedSuccessPayload)
    ));
}

#[test]
fn malformed_joined_identifiers_and_error_displays_are_covered() {
    for payload in [
        structure([
            item(UNIQUE_IDENTIFIER, Value::text_string("first".to_owned())),
            item(UNIQUE_IDENTIFIER, Value::integer(2)),
        ]),
        structure([item(UNIQUE_IDENTIFIER, Value::boolean(true))]),
    ] {
        let message = response_message(SUCCESS, None, Some(payload));
        assert_eq!(
            JoinSplitKeyResponse::try_from_response_item(response_item(&message)).unwrap_err(),
            JoinSplitKeyError::MalformedSuccessPayload
        );
    }

    for (identifier, value) in [
        (
            UniqueIdentifier::Enumeration(0xF123_4569),
            Value::enumeration(0xF123_4569),
        ),
        (UniqueIdentifier::Integer(-10), Value::integer(-10)),
    ] {
        let message = response_message(
            SUCCESS,
            None,
            Some(structure([item(UNIQUE_IDENTIFIER, value)])),
        );
        let response = JoinSplitKeyResponse::try_from_response_item(response_item(&message))
            .expect("the wire identifier form is preserved");
        assert_eq!(response.unique_identifier(), Some(&identifier));
    }

    let invalid_result =
        JoinSplitKeyError::InvalidOperationResult(ResultValidationError::SuccessForbidsReason);
    assert!(invalid_result.source().is_some());
    assert!(JoinSplitKeyError::UnexpectedOperation.source().is_none());
    for error in [
        JoinSplitKeyError::UnexpectedOperation,
        JoinSplitKeyError::MissingResultStatus,
        invalid_result,
        JoinSplitKeyError::MissingSuccessPayload,
        JoinSplitKeyError::MalformedSuccessPayload,
        JoinSplitKeyError::MalformedResponseItem,
    ] {
        assert_ne!(error.to_string(), "");
    }
}

#[test]
fn join_response_rejects_another_operation() {
    let other_operation = response_message_for_operation(
        JOIN_SPLIT_KEY_OPERATION + 1,
        OPERATION_FAILED,
        Some(FUTURE_RESULT_REASON),
        None,
    );
    assert_eq!(
        JoinSplitKeyResponse::try_from_response_item(response_item(&other_operation)).unwrap_err(),
        JoinSplitKeyError::UnexpectedOperation
    );
}

#[test]
fn request_and_response_debug_redact_identifier_values() {
    let request = request(vec![reference("private-split-identifier", 1)]);
    let request_debug = format!("{request:?}");
    assert!(!request_debug.contains("private-split-identifier"));

    let success = response_message(
        SUCCESS,
        None,
        Some(structure([item(
            UNIQUE_IDENTIFIER,
            Value::text_string("private-joined-identifier".to_owned()),
        )])),
    );
    let response = JoinSplitKeyResponse::try_from_response_item(response_item(&success))
        .expect("success response has its required identifier");
    let response_debug = format!("{response:?}");
    assert!(!response_debug.contains("private-joined-identifier"));
}
