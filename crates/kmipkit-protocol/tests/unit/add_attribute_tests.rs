#![cfg(test)]

//! Derived Add Attribute request vectors for OASIS KMIP v2.1 §6.1.2,
//! Tables 167–169, using the New Attribute structure in §5.7, Table 163.
//! Traceability: KMIPKIT-0016-FR-003/FR-010 and SC-002/SC-003.
//! These structural vectors do not claim that an official OASIS case passed.

use crate::{
    AddAttributeRequest, AddAttributeResponse, NewAttribute, ResponseBatchItemView,
    ResponseMessage, ResultReason, ResultStatus,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const NEW_ATTRIBUTE: u32 = 0x0042_013D;
const COMMENT: u32 = 0x0042_00FD;
const VENDOR_ATTRIBUTE: u32 = 0x0042_0008;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const ATTRIBUTE_NAME: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE: u32 = 0x0042_000B;
const ADD_ATTRIBUTE_OPERATION: u32 = 0x0000_000D;
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
        item(OPERATION, Value::enumeration(ADD_ATTRIBUTE_OPERATION)),
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
    let tree = structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(structure(batch))),
    ]);
    ResponseMessage::try_from_ttlv(tree).expect("fixture is a valid KMIP 2.1 response message")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one response batch item")
}

#[test]
fn request_encodes_optional_identifier_before_the_exact_new_attribute_value() {
    // Table 167 orders optional Unique Identifier before required New Attribute.
    // Table 163 requires New Attribute to wrap exactly one direct object attribute.
    let request = AddAttributeRequest::new(
        Some(OBJECT_IDENTIFIER.to_owned()),
        NewAttribute::new(item(
            COMMENT,
            Value::text_string("caller-supplied comment".to_owned()),
        )),
    );
    let payload = request
        .to_ttlv_payload()
        .expect("Table 167 request fields use allocated KMIP tags");
    let payload_view = payload.view();
    let fields = payload_view.children();

    assert_eq!(
        fields
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [
            (UNIQUE_IDENTIFIER, ItemType::TextString),
            (NEW_ATTRIBUTE, ItemType::Structure),
        ],
        "the operation payload contains only the ordered Table 167 fields"
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
        "New Attribute contains the caller's direct attribute Item"
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
        "the submitted Text String is retained exactly"
    );
}

#[test]
fn request_preserves_nested_vendor_value_when_identifier_is_omitted() {
    // Table 167 permits omitting Unique Identifier; Table 150 defines the
    // nested Vendor Attribute member order and Table 163 keeps it one value.
    let vendor_value = item(
        VENDOR_ATTRIBUTE,
        Value::structure(structure([
            item(
                VENDOR_IDENTIFICATION,
                Value::text_string("KMIPKit_TestVendor".to_owned()),
            ),
            item(
                ATTRIBUTE_NAME,
                Value::text_string("Opaque.Attribute".to_owned()),
            ),
            item(ATTRIBUTE_VALUE, Value::byte_string(vec![0x00, 0x80, 0xFF])),
        ])),
    );
    let request = AddAttributeRequest::new(None, NewAttribute::new(vendor_value));
    let payload = request
        .to_ttlv_payload()
        .expect("Table 167 request fields use allocated KMIP tags");
    let payload_view = payload.view();
    let fields = payload_view.children();

    assert_eq!(
        fields
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [(NEW_ATTRIBUTE, ItemType::Structure)],
        "omitting Unique Identifier does not omit the required New Attribute"
    );
    assert_eq!(
        fields[0].with_value(|value| match value {
            ValueView::Structure(wrapper) => Some(
                wrapper
                    .children()
                    .iter()
                    .map(|attribute| (attribute.tag().raw(), attribute.item_type()))
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        }),
        Some(vec![(VENDOR_ATTRIBUTE, ItemType::Structure)]),
        "New Attribute remains a single direct Vendor Attribute Item"
    );

    let vendor_members = fields[0].with_value(|value| match value {
        ValueView::Structure(wrapper) => wrapper.children().first().and_then(|attribute| {
            attribute.with_value(|value| match value {
                ValueView::Structure(vendor) => Some(
                    vendor
                        .children()
                        .iter()
                        .map(|member| {
                            let member_value = member.with_value(|value| match value {
                                ValueView::TextString(value) => Some(value.as_bytes().to_vec()),
                                ValueView::ByteString(value) => Some(value.to_vec()),
                                _ => None,
                            });
                            (member.tag().raw(), member.item_type(), member_value)
                        })
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
        }),
        _ => None,
    });
    assert_eq!(
        vendor_members,
        Some(vec![
            (
                VENDOR_IDENTIFICATION,
                ItemType::TextString,
                Some(b"KMIPKit_TestVendor".to_vec()),
            ),
            (
                ATTRIBUTE_NAME,
                ItemType::TextString,
                Some(b"Opaque.Attribute".to_vec()),
            ),
            (
                ATTRIBUTE_VALUE,
                ItemType::ByteString,
                Some(vec![0x00, 0x80, 0xFF]),
            ),
        ]),
        "the nested vendor name and caller bytes remain unchanged and ordered"
    );
}

#[test]
fn successful_response_preserves_the_required_unique_identifier() {
    // §6.1.2 Table 168 requires Unique Identifier on successful responses.
    let payload = structure([item(
        UNIQUE_IDENTIFIER,
        Value::text_string(OBJECT_IDENTIFIER.to_owned()),
    )]);
    let message = response_message(0, None, None, Some(payload));
    let response = AddAttributeResponse::try_from_response_item(response_item(&message))
        .expect("Table 168 success contains its required Unique Identifier");

    assert_eq!(response.result().status(), ResultStatus::from_raw(0));
    assert_eq!(response.result().reason(), None);
    assert_eq!(response.unique_identifier(), Some(OBJECT_IDENTIFIER));
}

#[test]
fn successful_response_without_unique_identifier_is_rejected() {
    // Table 168 requires Unique Identifier on every successful response.
    let message = response_message(0, None, None, Some(Structure::new()));
    assert!(
        AddAttributeResponse::try_from_response_item(response_item(&message)).is_err(),
        "a successful response without Unique Identifier is malformed"
    );
}

#[test]
fn response_preserves_each_unique_table_169_failure_reason_and_message() {
    // Table 169 assigns Operation Failed to these unique reasons. Its printed
    // list repeats Invalid Message; each assigned reason is checked once.
    let table_169_reasons = [
        0x23, 0x2C, 0x04, 0x35, 0x37, 0x1D, 0x3A, 0x15, 0x14, 0x08, 0x07, 0x05, 0x0C, 0x02, 0x43,
    ];

    for expected_reason in table_169_reasons {
        let message = response_message(
            1,
            Some(expected_reason),
            Some("Add Attribute rejected by server"),
            None,
        );
        let response = AddAttributeResponse::try_from_response_item(response_item(&message))
            .expect("Table 169 errors retain the common operation result");

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
            Some("Add Attribute rejected by server")
        );
        assert!(response.unique_identifier().is_none());
    }
}
