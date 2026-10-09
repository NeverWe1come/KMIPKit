#![cfg(test)]

//! Derived request/response vectors for OASIS KMIP v2.1 §6.1.20, Tables
//! 223–225. Traceability: KMIPKIT-0016-FR-006/FR-010 and SC-002/SC-004.
//! These are derived structural tests; they do not claim an official OASIS case passed.

use crate::{
    AttributeReference, GetAttributesRequest, GetAttributesResponse, ResponseBatchItemView,
    ResponseMessage, ResultReason, ResultStatus,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const GET_ATTRIBUTES_OPERATION: u32 = 0x0000_000B;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ATTRIBUTES: u32 = 0x0042_0125;
const ATTRIBUTE_REFERENCE: u32 = 0x0042_013B;
const VENDOR_ATTRIBUTE: u32 = 0x0042_0008;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const ATTRIBUTE_NAME: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE: u32 = 0x0042_000B;
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

fn successful_payload(attributes: impl IntoIterator<Item = Item>) -> Structure {
    structure([
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string(OBJECT_IDENTIFIER.to_owned()),
        ),
        item(ATTRIBUTES, Value::structure(structure(attributes))),
    ])
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
        item(OPERATION, Value::enumeration(GET_ATTRIBUTES_OPERATION)),
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

fn attribute_integer(attribute: &Item) -> Option<i32> {
    attribute.with_value(|value| match value {
        ValueView::Integer(value) => Some(*value),
        _ => None,
    })
}

#[test]
fn request_follows_table_223_order_and_allows_repeated_reference_fields() {
    let request = GetAttributesRequest::try_new(
        Some(OBJECT_IDENTIFIER.to_owned()),
        [
            AttributeReference::tag(0x0042_002F),
            AttributeReference::name("KMIPKit.TestVendor_1", "Opaque.Attribute"),
        ],
    )
    .expect("the request references two distinct attributes");
    let payload = request
        .to_ttlv_payload()
        .expect("Table 223 request fields use allocated KMIP tags");
    let view = payload.view();
    let fields = view.children();

    let actual = fields
        .iter()
        .map(|field| (field.tag().raw(), field.item_type()))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            (0x0042_0094, ItemType::TextString),
            (ATTRIBUTE_REFERENCE, ItemType::Enumeration),
            (ATTRIBUTE_REFERENCE, ItemType::Structure),
        ]
    );
    assert_eq!(
        fields[0].with_value(|value| match value {
            ValueView::TextString(value) => Some(value.as_bytes().to_vec()),
            _ => None,
        }),
        Some(b"object-id-17".to_vec())
    );
    assert_eq!(
        fields[1].with_value(|value| match value {
            ValueView::Enumeration(value) => Some(*value),
            _ => None,
        }),
        Some(0x0042_002F),
        "tag-form Attribute Reference carries the requested KMIP tag"
    );
    let name_fields = fields[2].with_value(|value| match value {
        ValueView::Structure(structure) => Some(
            structure
                .children()
                .iter()
                .map(|field| {
                    (
                        field.tag().raw(),
                        field.item_type(),
                        field.with_value(|value| match value {
                            ValueView::TextString(value) => Some(value.to_owned()),
                            _ => None,
                        }),
                    )
                })
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert_eq!(
        name_fields,
        Some(vec![
            (
                VENDOR_IDENTIFICATION,
                ItemType::TextString,
                Some("KMIPKit.TestVendor_1".to_owned()),
            ),
            (
                ATTRIBUTE_NAME,
                ItemType::TextString,
                Some("Opaque.Attribute".to_owned()),
            ),
        ]),
        "name-form Attribute Reference contains vendor identification and attribute name"
    );
}

#[test]
fn request_preserves_optional_identifier_and_zero_or_more_reference_cardinality() {
    let request_without_identifier =
        GetAttributesRequest::try_new(None, [AttributeReference::tag(0x0042_002F)])
            .expect("Unique Identifier is optional in Table 223");
    let request_fields = request_without_identifier
        .to_ttlv_payload()
        .expect("the request payload is valid")
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect::<Vec<_>>();
    assert_eq!(request_fields, [0x0042_013B]);

    let request_without_identifier_or_references =
        GetAttributesRequest::try_new(None, []).expect("Table 223 permits zero references");
    let payload = request_without_identifier_or_references
        .to_ttlv_payload()
        .expect("an omitted identifier and zero references remain omitted");
    assert!(payload.view().children().is_empty());
}

#[test]
fn request_rejects_a_repeated_identical_attribute_reference() {
    assert!(
        GetAttributesRequest::try_new(
            None,
            [
                AttributeReference::tag(0x0042_002F),
                AttributeReference::tag(0x0042_002F),
            ],
        )
        .is_err(),
        "§6.1.20 forbids the same Attribute Reference more than once"
    );
}

#[test]
fn request_rejects_a_reserved_tag_form_reference_before_encoding() {
    // OASIS KMIP v2.1 §11.56 and ADR-0010 require outbound raw tags to pass
    // the tag-allocation gate; an unknown Enumeration remains representable,
    // but a reserved Raw Tag cannot be emitted as an Item.
    let request = GetAttributesRequest::try_new(None, [AttributeReference::tag(0x0042_0009)])
        .expect("the raw tag is retained while constructing the reference");

    assert!(
        request.to_ttlv_payload().is_err(),
        "a reserved §11.56 tag cannot be encoded as an Attribute Reference"
    );
}

#[test]
fn response_preserves_direct_attribute_instances_and_omits_missing_values() {
    let request = GetAttributesRequest::try_new(
        Some(OBJECT_IDENTIFIER.to_owned()),
        [
            AttributeReference::tag(0x0042_002F),
            AttributeReference::tag(0x0042_0030),
        ],
    )
    .expect("the selected Attribute References are distinct");
    assert_eq!(
        request
            .to_ttlv_payload()
            .expect("valid request")
            .view()
            .children()
            .len(),
        3
    );

    let message = response_message(
        0,
        None,
        None,
        Some(successful_payload([
            item(0x0042_002F, Value::integer(7)),
            item(0x0042_002F, Value::integer(19)),
        ])),
    );
    let actual = GetAttributesResponse::try_from_response_item(response_item(&message))
        .expect("Table 224 response has the required Unique Identifier and Attributes");

    assert_eq!(actual.result().status(), ResultStatus::from_raw(0));
    assert_eq!(actual.result().reason(), None);
    let attributes = actual
        .attributes()
        .expect("successful result has Attributes");

    assert_eq!(attributes.as_items().len(), 2);
    assert_eq!(
        attributes
            .as_items()
            .iter()
            .map(|attribute| (attribute.tag().raw(), attribute_integer(attribute)))
            .collect::<Vec<_>>(),
        [(0x0042_002F, Some(7)), (0x0042_002F, Some(19))]
    );
}

#[test]
fn response_without_requested_values_keeps_required_identifier_and_empty_attributes() {
    let _request = GetAttributesRequest::try_new(
        Some(OBJECT_IDENTIFIER.to_owned()),
        [AttributeReference::tag(0x0042_002F)],
    )
    .expect("the request has one Attribute Reference");
    let message = response_message(0, None, None, Some(successful_payload([])));
    let actual = GetAttributesResponse::try_from_response_item(response_item(&message))
        .expect("Table 224 requires the Unique Identifier and Attributes fields");

    assert_eq!(actual.unique_identifier(), Some("object-id-17"));
    assert!(
        actual
            .attributes()
            .expect("successful result has the required Attributes field")
            .as_items()
            .is_empty(),
        "the server returns no fabricated value when the requested attribute is absent"
    );
}

#[test]
fn response_without_references_preserves_the_full_direct_attribute_set_in_order() {
    let request = GetAttributesRequest::try_new(None, [])
        .expect("Table 223 allows no Attribute Reference to request all attributes");
    assert!(
        request
            .to_ttlv_payload()
            .expect("valid request")
            .view()
            .children()
            .is_empty()
    );

    let message = response_message(
        0,
        None,
        None,
        Some(successful_payload([
            item(0x0042_0057, Value::enumeration(3)),
            item(0x0042_002F, Value::integer(7)),
            item(0x0042_0057, Value::enumeration(4)),
        ])),
    );
    let actual = GetAttributesResponse::try_from_response_item(response_item(&message))
        .expect("all direct attributes are valid Table 224 response members");
    let attributes = actual
        .attributes()
        .expect("successful result has Attributes");

    assert_eq!(
        attributes
            .as_items()
            .iter()
            .map(|attribute| attribute.tag().raw())
            .collect::<Vec<_>>(),
        [0x0042_0057, 0x0042_002F, 0x0042_0057]
    );
}

#[test]
fn response_preserves_every_table_225_failure_reason_status_and_message() {
    // Table 225: Invalid Attribute, Object Not Found, Attestation Failed,
    // Attestation Required, Feature Not Supported, Invalid Field, Invalid
    // Message, Operation Not Supported, Permission Denied, Response Too Large.
    let table_225_reasons = [0x2C, 0x37, 0x15, 0x14, 0x08, 0x07, 0x04, 0x05, 0x0C, 0x02];

    for expected_reason in table_225_reasons {
        let message = response_message(
            1,
            Some(expected_reason),
            Some("Get Attributes rejected by server"),
            None,
        );
        let actual = GetAttributesResponse::try_from_response_item(response_item(&message))
            .expect("Table 225 operation failures retain their common KMIP result");

        assert_eq!(actual.result().status(), ResultStatus::from_raw(1));
        assert_eq!(
            actual.result().reason(),
            Some(ResultReason::from_raw(expected_reason))
        );
        assert_eq!(
            actual.result().message().map(|message| message.as_str()),
            Some("Get Attributes rejected by server")
        );
        assert!(actual.attributes().is_none());
    }
}

#[test]
fn response_rejects_payloads_missing_required_fields_or_violating_table_224_shape() {
    let malformed_payloads = [
        Structure::new(),
        structure([item(ATTRIBUTES, Value::structure(Structure::new()))]),
        structure([item(
            UNIQUE_IDENTIFIER,
            Value::text_string(OBJECT_IDENTIFIER.to_owned()),
        )]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::integer(17)),
            item(ATTRIBUTES, Value::structure(Structure::new())),
        ]),
        structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            ),
            item(ATTRIBUTES, Value::integer(17)),
        ]),
        structure([
            item(ATTRIBUTES, Value::structure(Structure::new())),
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            ),
        ]),
        structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            ),
            item(ATTRIBUTES, Value::structure(Structure::new())),
            item(ATTRIBUTES, Value::structure(Structure::new())),
        ]),
        structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            ),
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            ),
            item(ATTRIBUTES, Value::structure(Structure::new())),
        ]),
    ];
    for payload in malformed_payloads {
        let message = response_message(0, None, None, Some(payload));
        assert!(
            GetAttributesResponse::try_from_response_item(response_item(&message)).is_err(),
            "Table 224 requires one Unique Identifier followed by one Attributes Structure"
        );
    }
}

#[test]
fn response_rejects_vendor_attribute_fields_outside_table_150_order() {
    // OASIS KMIP v2.1 §4.60, Table 150 lists Vendor Identification, Attribute
    // Name, then Attribute Value. Section 10.1 requires Structure fields to
    // appear in the order in which they are defined.
    let out_of_order_vendor_attribute = item(
        VENDOR_ATTRIBUTE,
        Value::structure(structure([
            item(
                ATTRIBUTE_NAME,
                Value::text_string("Opaque.Attribute".to_owned()),
            ),
            item(
                VENDOR_IDENTIFICATION,
                Value::text_string("KMIPKit.TestVendor_1".to_owned()),
            ),
            item(ATTRIBUTE_VALUE, Value::byte_string(vec![0, 0x80, 0xff])),
        ])),
    );
    let message = response_message(
        0,
        None,
        None,
        Some(successful_payload([out_of_order_vendor_attribute])),
    );

    assert!(
        GetAttributesResponse::try_from_response_item(response_item(&message)).is_err(),
        "a successful response cannot accept Table 150 members out of order"
    );
}
