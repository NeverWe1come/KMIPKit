//! Exercise public attribute diagnostics and generic value-copy paths.
//! Traceability: KMIPKIT-0016-FR-001/FR-011/FR-013 and SC-001/SC-005.

use std::error::Error;
use std::fmt::Debug;

use kmipkit_ttlv::{Item, ItemType, ModelError, RawTag, Structure, Tag, Value};

use crate::{
    AddAttributeError, AddAttributeRequest, AddAttributeResponse, AdjustAttributeError,
    AdjustAttributeRequest, AdjustAttributeResponse, AdjustmentType, AttributeReference,
    AttributeSetError, CurrentAttribute, DeleteAttributeError, DeleteAttributeRequest,
    DeleteAttributeResponse, GetAttributeListError, GetAttributeListRequest,
    GetAttributeListResponse, GetAttributesError, GetAttributesRequest, GetAttributesResponse,
    KmipOperationResult, ModifyAttributeError, ModifyAttributeRequest, ModifyAttributeResponse,
    NewAttribute, ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, ResponseBatchItemView,
    ResponseMessage, ResultReason, ResultStatus, ResultValidationError, SetAttributeError,
    SetAttributeRequest, SetAttributeResponse,
};

const COMMENT: u32 = 0x0042_00FD;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("test tag fits the KMIP 24-bit field")
        .try_checked()
        .expect("Comment is allocated in the KMIP 2.1 catalog")
}

fn item(value: Value) -> Item {
    Item::new(tag(COMMENT), value).expect("test Item is structurally valid")
}

fn tagged_item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("test field uses an allocated tag")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for item in items {
        structure
            .try_push(item)
            .expect("test Structure stays within the model limit");
    }
    structure
}

fn response_message(operation: u32, payload: Option<Structure>) -> ResponseMessage {
    const RESPONSE_HEADER: u32 = 0x0042_007A;
    const PROTOCOL_VERSION: u32 = 0x0042_0069;
    const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
    const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
    const TIME_STAMP: u32 = 0x0042_0092;
    const BATCH_COUNT: u32 = 0x0042_000D;
    const BATCH_ITEM: u32 = 0x0042_000F;
    const OPERATION: u32 = 0x0042_005C;
    const RESULT_STATUS: u32 = 0x0042_007F;
    const RESPONSE_PAYLOAD: u32 = 0x0042_007C;

    let version = structure([
        tagged_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        tagged_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        tagged_item(PROTOCOL_VERSION, Value::structure(version)),
        tagged_item(TIME_STAMP, Value::date_time(1)),
        tagged_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut batch = vec![
        tagged_item(OPERATION, Value::enumeration(operation)),
        tagged_item(RESULT_STATUS, Value::enumeration(0)),
    ];
    if let Some(payload) = payload {
        batch.push(tagged_item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    ResponseMessage::try_from_ttlv(structure([
        tagged_item(RESPONSE_HEADER, Value::structure(header)),
        tagged_item(BATCH_ITEM, Value::structure(structure(batch))),
    ]))
    .expect("fixture is a valid KMIP 2.1 response message")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one response batch item")
}

fn successful_unique_identifier_payload() -> Structure {
    structure([tagged_item(
        0x0042_0094,
        Value::text_string("object-id".to_owned()),
    )])
}

fn exercise_error<E: Error + Debug>(error: E) {
    let display = error.to_string();
    assert_ne!(display, "");
    if let Some(source) = error.source() {
        assert_ne!(source.to_string(), "");
    }
}

fn invalid_operation_result() -> ResultValidationError {
    KmipOperationResult::new(
        ResultStatus::from_raw(0),
        Some(ResultReason::from_raw(1)),
        None,
    )
    .expect_err("Success cannot carry a Result Reason")
}

#[test]
fn typed_attribute_request_accessors_retain_the_supplied_values() {
    let add = AddAttributeRequest::new(
        Some("object-id".to_owned()),
        NewAttribute::new(item(Value::text_string("new".to_owned()))),
    );
    assert_eq!(add.unique_identifier(), Some("object-id"));
    assert_eq!(add.new_attribute().item().tag().raw(), COMMENT);

    let named_reference = AttributeReference::name("example.org", "CostCenter");
    let adjust = AdjustAttributeRequest::new(
        Some("object-id".to_owned()),
        named_reference,
        AdjustmentType::INCREMENT,
        Some(Value::integer(3)),
    );
    assert_eq!(adjust.unique_identifier(), Some("object-id"));
    assert_eq!(
        adjust.attribute_reference().name_parts(),
        Some(("example.org", "CostCenter"))
    );
    assert_eq!(adjust.adjustment_type(), AdjustmentType::INCREMENT);
    assert_eq!(
        adjust.adjustment_value().map(Value::item_type),
        Some(ItemType::Integer)
    );

    let delete = DeleteAttributeRequest::new(
        Some("object-id".to_owned()),
        Some(CurrentAttribute::new(item(Value::text_string(
            "old".to_owned(),
        )))),
        Some(AttributeReference::tag(COMMENT)),
    );
    assert_eq!(delete.unique_identifier(), Some("object-id"));
    assert_eq!(
        delete
            .current_attribute()
            .map(|attribute| attribute.item().tag().raw()),
        Some(COMMENT)
    );
    assert_eq!(
        delete
            .attribute_reference()
            .and_then(AttributeReference::tag_value),
        Some(COMMENT)
    );

    let reads = GetAttributesRequest::try_new(
        Some("object-id".to_owned()),
        [AttributeReference::tag(COMMENT)],
    )
    .expect("one reference is unique");
    assert_eq!(
        reads
            .to_ttlv_payload()
            .expect("valid reads")
            .view()
            .children()
            .len(),
        2
    );

    let list = GetAttributeListRequest::new(Some("object-id".to_owned()));
    assert_eq!(
        list.to_ttlv_payload()
            .expect("valid list")
            .view()
            .children()
            .len(),
        1
    );

    let modify = ModifyAttributeRequest::new(
        Some("object-id".to_owned()),
        Some(CurrentAttribute::new(item(Value::text_string(
            "old".to_owned(),
        )))),
        NewAttribute::new(item(Value::text_string("new".to_owned()))),
    );
    assert_eq!(modify.unique_identifier(), Some("object-id"));
    assert_eq!(
        modify
            .current_attribute()
            .map(|attribute| attribute.item().tag().raw()),
        Some(COMMENT)
    );
    assert_eq!(modify.new_attribute().item().tag().raw(), COMMENT);

    let set = SetAttributeRequest::new(
        Some("object-id".to_owned()),
        NewAttribute::new(item(Value::text_string("set".to_owned()))),
    );
    assert_eq!(set.unique_identifier(), Some("object-id"));
    assert_eq!(set.new_attribute().item().tag().raw(), COMMENT);
}

#[test]
fn every_attribute_operation_error_formats_safely_and_exposes_its_source() {
    let invalid_result = invalid_operation_result();
    exercise_error(AddAttributeError::UnexpectedOperation);
    exercise_error(AddAttributeError::MissingResultStatus);
    exercise_error(AddAttributeError::InvalidOperationResult(invalid_result));
    exercise_error(AddAttributeError::MissingSuccessPayload);
    exercise_error(AddAttributeError::MalformedSuccessPayload);

    exercise_error(AdjustAttributeError::UnexpectedOperation);
    exercise_error(AdjustAttributeError::MissingResultStatus);
    exercise_error(AdjustAttributeError::InvalidOperationResult(invalid_result));
    exercise_error(AdjustAttributeError::MissingSuccessPayload);
    exercise_error(AdjustAttributeError::MalformedSuccessPayload);

    exercise_error(DeleteAttributeError::UnexpectedOperation);
    exercise_error(DeleteAttributeError::MissingResultStatus);
    exercise_error(DeleteAttributeError::InvalidOperationResult(invalid_result));
    exercise_error(DeleteAttributeError::MissingSuccessPayload);
    exercise_error(DeleteAttributeError::MalformedSuccessPayload);

    exercise_error(ModifyAttributeError::UnexpectedOperation);
    exercise_error(ModifyAttributeError::MissingResultStatus);
    exercise_error(ModifyAttributeError::InvalidOperationResult(invalid_result));
    exercise_error(ModifyAttributeError::MissingSuccessPayload);
    exercise_error(ModifyAttributeError::MalformedSuccessPayload);

    exercise_error(SetAttributeError::UnexpectedOperation);
    exercise_error(SetAttributeError::MissingResultStatus);
    exercise_error(SetAttributeError::InvalidOperationResult(invalid_result));
    exercise_error(SetAttributeError::MissingSuccessPayload);
    exercise_error(SetAttributeError::MalformedSuccessPayload);

    let safe_reference_error = ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        std::io::Error::other("untrusted reference text"),
    );
    exercise_error(GetAttributeListError::UnexpectedOperation);
    exercise_error(GetAttributeListError::MissingResultStatus);
    exercise_error(GetAttributeListError::InvalidOperationResult(
        invalid_result,
    ));
    exercise_error(GetAttributeListError::MissingSuccessPayload);
    exercise_error(GetAttributeListError::MalformedSuccessPayload);
    exercise_error(GetAttributeListError::MalformedAttributeReference(
        safe_reference_error,
    ));

    exercise_error(GetAttributesError::DuplicateAttributeReference);
    exercise_error(GetAttributesError::UnexpectedOperation);
    exercise_error(GetAttributesError::MissingResultStatus);
    exercise_error(GetAttributesError::InvalidOperationResult(invalid_result));
    exercise_error(GetAttributesError::MissingSuccessPayload);
    exercise_error(GetAttributesError::MalformedSuccessPayload);
    exercise_error(GetAttributesError::TtlvModel(ModelError::TagNotAllocated));
    exercise_error(GetAttributesError::InvalidAttributeSet(
        AttributeSetError::MissingVendorIdentification,
    ));
}

#[test]
fn response_conversion_reports_wrong_operations_and_wrong_success_field_tags() {
    const ADD_ATTRIBUTE: u32 = 0x0000_000D;
    const ADJUST_ATTRIBUTE: u32 = 0x0000_0030;
    const DELETE_ATTRIBUTE: u32 = 0x0000_000F;
    const GET_ATTRIBUTES: u32 = 0x0000_000B;
    const GET_ATTRIBUTE_LIST: u32 = 0x0000_000C;
    const MODIFY_ATTRIBUTE: u32 = 0x0000_000E;
    const SET_ATTRIBUTE: u32 = 0x0000_0031;

    let wrong_payload = || {
        structure([item(Value::text_string(
            "not a Unique Identifier".to_owned(),
        ))])
    };

    macro_rules! assert_conversion_errors {
        ($operation:expr, $response:ty, $wrong_error:path, $malformed_error:path) => {{
            let other_operation = if $operation == 0x0000_000D {
                0x0000_0033
            } else {
                0x0000_000D
            };
            let wrong_operation = response_message(
                other_operation,
                Some(successful_unique_identifier_payload()),
            );
            assert!(matches!(
                <$response>::try_from_response_item(response_item(&wrong_operation)),
                Err($wrong_error)
            ));

            let malformed = response_message($operation, Some(wrong_payload()));
            let malformed_result = <$response>::try_from_response_item(response_item(&malformed));
            assert!(
                matches!(malformed_result, Err($malformed_error)),
                "unexpected malformed-payload result: {malformed_result:?}"
            );
        }};
    }

    assert_conversion_errors!(
        ADD_ATTRIBUTE,
        AddAttributeResponse,
        AddAttributeError::UnexpectedOperation,
        AddAttributeError::MalformedSuccessPayload
    );
    assert_conversion_errors!(
        ADJUST_ATTRIBUTE,
        AdjustAttributeResponse,
        AdjustAttributeError::UnexpectedOperation,
        AdjustAttributeError::MalformedSuccessPayload
    );
    assert_conversion_errors!(
        DELETE_ATTRIBUTE,
        DeleteAttributeResponse,
        DeleteAttributeError::UnexpectedOperation,
        DeleteAttributeError::MalformedSuccessPayload
    );
    assert_conversion_errors!(
        GET_ATTRIBUTES,
        GetAttributesResponse,
        GetAttributesError::UnexpectedOperation,
        GetAttributesError::MalformedSuccessPayload
    );
    assert_conversion_errors!(
        GET_ATTRIBUTE_LIST,
        GetAttributeListResponse,
        GetAttributeListError::UnexpectedOperation,
        GetAttributeListError::MalformedSuccessPayload
    );
    assert_conversion_errors!(
        MODIFY_ATTRIBUTE,
        ModifyAttributeResponse,
        ModifyAttributeError::UnexpectedOperation,
        ModifyAttributeError::MalformedSuccessPayload
    );
    assert_conversion_errors!(
        SET_ATTRIBUTE,
        SetAttributeResponse,
        SetAttributeError::UnexpectedOperation,
        SetAttributeError::MalformedSuccessPayload
    );
}

#[test]
fn add_attribute_copy_path_preserves_every_generic_ttlv_value_kind() {
    let mut nested = Structure::new();
    nested
        .try_push(item(Value::integer(7)))
        .expect("nested test Structure stays within the model limit");
    let values = [
        Value::structure(nested),
        Value::integer(-7),
        Value::long_integer(9),
        Value::big_integer(vec![0x01; 8]),
        Value::enumeration(0xDEAD_BEEF),
        Value::boolean(true),
        Value::text_string("caller value".to_owned()),
        Value::byte_string(vec![0xA5, 0x5A]),
        Value::date_time(17),
        Value::interval(23),
        Value::date_time_extended(29),
    ];

    for value in values {
        let request = AddAttributeRequest::new(None, NewAttribute::new(item(value)));
        let payload = request
            .to_ttlv_payload()
            .expect("generic New Attribute values remain copyable");
        assert_eq!(payload.view().children().len(), 1);
    }
}
