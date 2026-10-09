#![cfg(test)]

//! Derived Set Attribute request/response vectors for OASIS KMIP v2.1
//! §6.1.51, Tables 322–324, using the New Attribute structure in §5.7,
//! Table 163. Traceability: KMIPKIT-0016-FR-009/FR-010 and SC-002/SC-003.
//! These structural vectors do not claim that an official OASIS case passed.

use crate::{
    NewAttribute, ResponseBatchItemView, ResponseMessage, ResultReason, ResultStatus,
    SetAttributeRequest, SetAttributeResponse,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const SET_ATTRIBUTE_OPERATION: u32 = 0x0000_0031;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const NEW_ATTRIBUTE: u32 = 0x0042_013D;
const COMMENT: u32 = 0x0042_00FD;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const OBJECT_IDENTIFIER: &str = "object-id-17";

fn tag(raw_tag: u32) -> Tag {
    RawTag::new(raw_tag)
        .expect("fixture tag fits the 24-bit KMIP field")
        .try_checked()
        .expect("fixture tag is assigned by the KMIP 2.1 catalog")
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("fixture item uses an allocated tag")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("fixture structure stays within the model depth limit");
    }
    structure
}

fn response_message(
    status: u32,
    reason: Option<u32>,
    result_message: Option<&str>,
    payload: Option<Structure>,
) -> ResponseMessage {
    let version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(version)),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut batch = vec![
        item(OPERATION, Value::enumeration(SET_ATTRIBUTE_OPERATION)),
        item(RESULT_STATUS, Value::enumeration(status)),
    ];
    if let Some(reason) = reason {
        batch.push(item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(result_message) = result_message {
        batch.push(item(
            RESULT_MESSAGE,
            Value::text_string(result_message.to_owned()),
        ));
    }
    if let Some(payload) = payload {
        batch.push(item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    let message = structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(structure(batch))),
    ]);
    ResponseMessage::try_from_ttlv(message).expect("fixture is a valid KMIP 2.1 response message")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one response batch item")
}

#[test]
fn request_preserves_the_required_new_attribute_direct_item_and_table_322_order() {
    // Table 322 orders optional Unique Identifier before required New
    // Attribute. §5.7 Table 163 defines New Attribute as one direct Item for
    // an object attribute; it is not an attribute-name/value pair.
    let request = SetAttributeRequest::new(
        Some(OBJECT_IDENTIFIER.to_owned()),
        NewAttribute::new(item(
            COMMENT,
            Value::text_string("caller-supplied comment".to_owned()),
        )),
    );
    let payload = request
        .to_ttlv_payload()
        .expect("Table 322 fields use allocated KMIP tags");
    let fields = payload.view().children();

    assert_eq!(
        fields
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [
            (UNIQUE_IDENTIFIER, ItemType::TextString),
            (NEW_ATTRIBUTE, ItemType::Structure),
        ],
        "the payload contains only the ordered Table 322 fields"
    );
    assert_eq!(
        fields[0].with_value(|value| match value {
            ValueView::TextString(value) => Some(value.to_owned()),
            _ => None,
        }),
        Some(OBJECT_IDENTIFIER.to_owned())
    );
    assert_eq!(
        fields[1].with_value(|value| match value {
            ValueView::Structure(wrapper) => Some(
                wrapper
                    .children()
                    .iter()
                    .map(|attribute| (attribute.tag().raw(), attribute.item_type()))
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        }),
        Some(vec![(COMMENT, ItemType::TextString)]),
        "New Attribute wraps exactly the caller's direct attribute Item"
    );
    assert_eq!(
        fields[1].with_value(|value| match value {
            ValueView::Structure(wrapper) => wrapper.children().first().and_then(|attribute| {
                attribute.with_value(|value| match value {
                    ValueView::TextString(value) => Some(value.to_owned()),
                    _ => None,
                })
            }),
            _ => None,
        }),
        Some("caller-supplied comment".to_owned()),
        "the caller's New Attribute value is unchanged"
    );
}

#[test]
fn request_omits_optional_identifier_without_changing_new_attribute() {
    // Table 322 permits omitting Unique Identifier so the server may use its
    // batch-scoped ID Placeholder. The required New Attribute remains present.
    let request = SetAttributeRequest::new(
        None,
        NewAttribute::new(item(
            COMMENT,
            Value::text_string("unchanged without identifier".to_owned()),
        )),
    );
    let payload = request
        .to_ttlv_payload()
        .expect("the optional Unique Identifier may be omitted");
    let fields = payload.view().children();

    assert_eq!(
        fields
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [(NEW_ATTRIBUTE, ItemType::Structure)],
        "omitting Unique Identifier does not omit or reorder New Attribute"
    );
    assert_eq!(
        fields[0].with_value(|value| match value {
            ValueView::Structure(wrapper) => wrapper.children().first().and_then(|attribute| {
                attribute.with_value(|value| match value {
                    ValueView::TextString(value) => Some(value.to_owned()),
                    _ => None,
                })
            }),
            _ => None,
        }),
        Some("unchanged without identifier".to_owned())
    );
}

#[test]
fn existing_instance_cardinality_does_not_change_the_server_authoritative_request() {
    // §6.1.51 prose assigns the zero-instance add, one-instance modify, and
    // multi-instance error behavior to the server. Table 322 has no instance
    // count or replacement selector. These named scenarios assert only that
    // the same caller request is sent unchanged; they do not construct remote
    // object state or assert any server result.
    let server_scenarios = [
        "server has zero instances",
        "server has one instance",
        "server has multiple instances",
    ];
    let request = SetAttributeRequest::new(
        Some(OBJECT_IDENTIFIER.to_owned()),
        NewAttribute::new(item(
            COMMENT,
            Value::text_string("the server decides".to_owned()),
        )),
    );
    let payload = request
        .to_ttlv_payload()
        .expect("the request shape does not depend on remote instance cardinality");
    let fields = payload.view().children();

    for server_scenario in server_scenarios {
        assert_eq!(
            fields
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>(),
            [UNIQUE_IDENTIFIER, NEW_ATTRIBUTE],
            "{server_scenario}: keep the Table 322 request shape unchanged"
        );
        assert_eq!(
            fields[1].with_value(|value| match value {
                ValueView::Structure(wrapper) => wrapper.children().first().and_then(|attribute| {
                    attribute.with_value(|value| match value {
                        ValueView::TextString(value) => Some(value.to_owned()),
                        _ => None,
                    })
                }),
                _ => None,
            }),
            Some("the server decides".to_owned()),
            "{server_scenario}: preserve the caller's New Attribute"
        );
    }
}

#[test]
fn successful_response_contains_the_required_unique_identifier() {
    // Table 323 defines Unique Identifier as the sole required response field.
    let payload = structure([item(
        UNIQUE_IDENTIFIER,
        Value::text_string(OBJECT_IDENTIFIER.to_owned()),
    )]);
    assert_eq!(
        payload
            .view()
            .children()
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [(UNIQUE_IDENTIFIER, ItemType::TextString)]
    );
    let message = response_message(0, None, None, Some(payload));
    let response = SetAttributeResponse::try_from_response_item(response_item(&message))
        .expect("Table 323 response has its required Unique Identifier");

    assert_eq!(response.result().status(), ResultStatus::from_raw(0));
    assert_eq!(response.result().reason(), None);
    assert_eq!(response.unique_identifier(), Some(OBJECT_IDENTIFIER));
}

#[test]
fn response_preserves_each_unique_table_324_failure_reason_and_message() {
    // Table 324 lists Operation Failed with Invalid Attribute Value,
    // Multi Valued Attribute, Non Unique Name Attribute, Object Not Found,
    // Read Only Attribute, Attestation Failed, Attestation Required, Feature
    // Not Supported, Invalid Field, Invalid Message, Operation Not Supported,
    // Permission Denied, Response Too Large, and Wrong Key Lifecycle State.
    // Its printed list repeats Invalid Attribute Value; each unique assigned
    // reason is therefore exercised once.
    let table_324_reasons = [
        0x2D, 0x1E, 0x35, 0x37, 0x1D, 0x15, 0x14, 0x08, 0x07, 0x04, 0x05, 0x0C, 0x02, 0x43,
    ];

    for expected_reason in table_324_reasons {
        let message = response_message(
            1,
            Some(expected_reason),
            Some("Set Attribute rejected by server"),
            None,
        );
        let response = SetAttributeResponse::try_from_response_item(response_item(&message))
            .expect("Table 324 errors retain the common KMIP result fields");

        assert_eq!(response.result().status(), ResultStatus::from_raw(1));
        assert_eq!(
            response.result().reason(),
            Some(ResultReason::from_raw(expected_reason))
        );
        assert_eq!(
            response
                .result()
                .message()
                .map(crate::ResultMessage::as_str),
            Some("Set Attribute rejected by server")
        );
        assert!(response.unique_identifier().is_none());
    }
}
