#![cfg(test)]

//! Derived Delete Attribute request/response vectors for OASIS KMIP v2.1
//! §6.1.13, Tables 202–204, and §5.6, Table 162. Traceability:
//! KMIPKIT-0016-FR-005/FR-010 and SC-002/SC-003.
//! Operation-specific failures from Table 204 are exercised with the client
//! result-preservation tests in T032; these structural vectors do not claim
//! that an official OASIS case passed.

use crate::{
    AttributeReference, CurrentAttribute, DeleteAttributeRequest, DeleteAttributeResponse,
    ResponseBatchItemView, ResponseMessage, ResultStatus,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const DELETE_ATTRIBUTE_OPERATION: u32 = 0x0000_000F;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CURRENT_ATTRIBUTE: u32 = 0x0042_013C;
const ATTRIBUTE_REFERENCE: u32 = 0x0042_013B;
const COMMENT: u32 = 0x0042_00FD;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const ATTRIBUTE_NAME: u32 = 0x0042_000A;
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

fn successful_response_message(payload: Structure) -> ResponseMessage {
    let version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(version)),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch = structure([
        item(OPERATION, Value::enumeration(DELETE_ATTRIBUTE_OPERATION)),
        item(RESULT_STATUS, Value::enumeration(0)),
        item(RESPONSE_PAYLOAD, Value::structure(payload)),
    ]);
    let message = structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(batch)),
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
fn request_preserves_current_attribute_as_one_direct_item_in_table_202_order() {
    // Table 202 orders optional Unique Identifier, Current Attribute, and
    // Attribute Reference. §5.6 Table 162 defines Current Attribute as one
    // direct object-attribute Item inside its Structure.
    let request = DeleteAttributeRequest::new(
        Some(OBJECT_IDENTIFIER.to_owned()),
        Some(CurrentAttribute::new(item(
            COMMENT,
            Value::text_string("selected value".to_owned()),
        ))),
        Some(AttributeReference::tag(COMMENT)),
    );
    let payload = request
        .to_ttlv_payload()
        .expect("Table 202 request fields use allocated KMIP tags");
    let payload_view = payload.view();
    let fields = payload_view.children();

    assert_eq!(
        fields
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [
            (UNIQUE_IDENTIFIER, ItemType::TextString),
            (CURRENT_ATTRIBUTE, ItemType::Structure),
            (ATTRIBUTE_REFERENCE, ItemType::Enumeration),
        ],
        "the request contains only the ordered Table 202 fields"
    );
    assert_eq!(
        fields[0].with_value(|value| match value {
            ValueView::TextString(value) => Some(value.to_owned()),
            _ => None,
        }),
        Some(OBJECT_IDENTIFIER.to_owned())
    );
    let current_attribute = fields[1].with_value(|value| match value {
        ValueView::Structure(current) => Some(
            current
                .children()
                .iter()
                .map(|attribute| (attribute.tag().raw(), attribute.item_type()))
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert_eq!(
        current_attribute,
        Some(vec![(COMMENT, ItemType::TextString)]),
        "Current Attribute wraps exactly one direct attribute Item"
    );
    assert_eq!(
        fields[1].with_value(|value| match value {
            ValueView::Structure(current) => current.children().first().and_then(|attribute| {
                attribute.with_value(|value| match value {
                    ValueView::TextString(value) => Some(value.to_owned()),
                    _ => None,
                })
            }),
            _ => None,
        }),
        Some("selected value".to_owned()),
        "the selected attribute value remains unchanged"
    );
    assert_eq!(
        fields[2].with_value(|value| match value {
            ValueView::Enumeration(value) => Some(*value),
            _ => None,
        }),
        Some(COMMENT),
        "the supplied Attribute Reference remains unchanged"
    );
}

#[test]
fn request_omits_current_attribute_and_preserves_name_reference() {
    // Table 202 makes Current Attribute optional. §6.1.13 specifies that when
    // it is omitted and Attribute Reference is present, the server deletes
    // all instances of the referenced attribute; this vector asserts only the
    // unchanged request shape and does not model server state.
    let request = DeleteAttributeRequest::new(
        None,
        None,
        Some(AttributeReference::name(
            "KMIPKit.TestVendor_1",
            "Opaque.Attribute",
        )),
    );
    let payload = request
        .to_ttlv_payload()
        .expect("name-form Attribute Reference is valid");
    let payload_view = payload.view();
    let fields = payload_view.children();

    assert_eq!(
        fields
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [(ATTRIBUTE_REFERENCE, ItemType::Structure)],
        "the optional identifier and Current Attribute stay absent"
    );
    let reference_members = fields[0].with_value(|value| match value {
        ValueView::Structure(reference) => Some(
            reference
                .children()
                .iter()
                .map(|field| {
                    let text = field.with_value(|value| match value {
                        ValueView::TextString(value) => Some(value.to_owned()),
                        _ => None,
                    });
                    (field.tag().raw(), field.item_type(), text)
                })
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert_eq!(
        reference_members,
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
        "the name-form selector is preserved exactly and in Table 161 order"
    );
}

#[test]
fn request_preserves_both_optional_selectors_as_absent() {
    // Table 202 marks both selectors optional. This checks serialization only;
    // it assigns no deletion semantics when neither selector is supplied.
    let request = DeleteAttributeRequest::new(Some(OBJECT_IDENTIFIER.to_owned()), None, None);
    let payload = request
        .to_ttlv_payload()
        .expect("the optional selectors may be omitted");

    assert_eq!(
        payload
            .view()
            .children()
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [(UNIQUE_IDENTIFIER, ItemType::TextString)],
        "omitted selectors are not synthesized or reordered"
    );
}

#[test]
fn successful_response_contains_the_required_unique_identifier() {
    // Table 203 defines Unique Identifier as the sole required response field.
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
    let message = successful_response_message(payload);
    let response = DeleteAttributeResponse::try_from_response_item(response_item(&message))
        .expect("Table 203 response has its required Unique Identifier");

    assert_eq!(response.result().status(), ResultStatus::from_raw(0));
    assert_eq!(response.unique_identifier(), Some(OBJECT_IDENTIFIER));
}
