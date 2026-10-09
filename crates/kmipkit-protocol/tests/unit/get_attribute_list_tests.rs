#![cfg(test)]

//! Derived request/response vectors for OASIS KMIP v2.1 §6.1.21, Tables
//! 226–228, with Attribute Reference encoding from §5.5, Table 161.
//! Traceability: KMIPKIT-0016-FR-007/FR-010 and SC-002/SC-004. These are
//! derived structural tests; they do not claim an official OASIS case passed.

use crate::{
    AttributeReference, GetAttributeListRequest, GetAttributeListResponse, ResponseBatchItemView,
    ResponseMessage, ResultReason, ResultStatus,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const GET_ATTRIBUTE_LIST_OPERATION: u32 = 0x0000_000C;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ATTRIBUTE_REFERENCE: u32 = 0x0042_013B;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const ATTRIBUTE_NAME: u32 = 0x0042_000A;
const TAG_FORM_REFERENCE: u32 = 0x0042_002F;
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
const VENDOR: &str = "KMIPKit_TestVendor";

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

fn name_reference(attribute_name: &str) -> Item {
    item(
        ATTRIBUTE_REFERENCE,
        Value::structure(structure([
            item(VENDOR_IDENTIFICATION, Value::text_string(VENDOR.to_owned())),
            item(
                ATTRIBUTE_NAME,
                Value::text_string(attribute_name.to_owned()),
            ),
        ])),
    )
}

fn tag_reference(attribute_tag: u32) -> Item {
    item(ATTRIBUTE_REFERENCE, Value::enumeration(attribute_tag))
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
        item(OPERATION, Value::enumeration(GET_ATTRIBUTE_LIST_OPERATION)),
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

fn successful_payload(references: impl IntoIterator<Item = Item>) -> Structure {
    structure(
        std::iter::once(item(
            UNIQUE_IDENTIFIER,
            Value::text_string(OBJECT_IDENTIFIER.to_owned()),
        ))
        .chain(references),
    )
}

#[test]
fn request_encodes_only_the_optional_unique_identifier_from_table_226() {
    let request = GetAttributeListRequest::try_new(Some(OBJECT_IDENTIFIER.to_owned()))
        .expect("Table 226 permits the Unique Identifier to be supplied");
    let payload = request
        .to_ttlv_payload()
        .expect("the Table 226 request field uses an allocated tag");
    let view = payload.view();
    let fields = view.children();

    assert_eq!(
        fields
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [(UNIQUE_IDENTIFIER, ItemType::TextString)]
    );
    assert_eq!(
        fields[0].with_value(|value| match value {
            ValueView::TextString(value) => Some(value.as_bytes().to_vec()),
            _ => None,
        }),
        Some(OBJECT_IDENTIFIER.as_bytes().to_vec())
    );

    let request_without_identifier =
        GetAttributeListRequest::try_new(None).expect("Table 226 makes Unique Identifier optional");
    assert!(
        request_without_identifier
            .to_ttlv_payload()
            .expect("an omitted identifier remains omitted")
            .view()
            .children()
            .is_empty()
    );
}

#[test]
fn response_returns_the_full_name_list_and_preserves_repeated_names_in_wire_order() {
    // Table 226 has no Attribute Reference request selector. The §6.1.21
    // prose therefore requires this request shape to return all attributes.
    let message = response_message(
        0,
        None,
        None,
        Some(successful_payload([
            name_reference("Opaque.Alpha"),
            name_reference("Opaque.Beta"),
            name_reference("Opaque.Alpha"),
        ])),
    );
    let actual = GetAttributeListResponse::try_from_response_item(response_item(&message))
        .expect("Table 227 permits one or more returned Attribute References");

    assert_eq!(actual.unique_identifier(), Some(OBJECT_IDENTIFIER));
    assert_eq!(
        actual.attribute_references(),
        Some(
            &[
                AttributeReference::name(VENDOR, "Opaque.Alpha"),
                AttributeReference::name(VENDOR, "Opaque.Beta"),
                AttributeReference::name(VENDOR, "Opaque.Alpha"),
            ][..]
        ),
        "the full response list retains exact names, repetitions, and wire order"
    );
}

#[test]
fn response_preserves_tag_form_attribute_references() {
    // §5.5 Table 161 allows Attribute Reference to use Enumeration (Tag).
    let message = response_message(
        0,
        None,
        None,
        Some(successful_payload([tag_reference(TAG_FORM_REFERENCE)])),
    );
    let actual = GetAttributeListResponse::try_from_response_item(response_item(&message))
        .expect("Table 227 accepts a valid tag-form Attribute Reference");

    assert_eq!(
        actual.attribute_references(),
        Some(&[AttributeReference::tag(TAG_FORM_REFERENCE)][..]),
        "the Attribute Reference Enumeration retains the exact tag value"
    );
}

#[test]
fn response_preserves_every_table_228_failure_reason_status_and_message() {
    // Table 228: Object Not Found, Attestation Failed, Attestation Required,
    // Feature Not Supported, Invalid Field, Invalid Message, Operation Not
    // Supported, Permission Denied, and Response Too Large.
    let table_228_reasons = [0x37, 0x15, 0x14, 0x08, 0x07, 0x04, 0x05, 0x0C, 0x02];

    for expected_reason in table_228_reasons {
        let message = response_message(
            1,
            Some(expected_reason),
            Some("Get Attribute List rejected by server"),
            None,
        );
        let actual = GetAttributeListResponse::try_from_response_item(response_item(&message))
            .expect("Table 228 failures use the common KMIP operation result");

        assert_eq!(actual.result().status(), ResultStatus::from_raw(1));
        assert_eq!(
            actual.result().reason(),
            Some(ResultReason::from_raw(expected_reason))
        );
        assert_eq!(
            actual.result().message().map(|message| message.as_str()),
            Some("Get Attribute List rejected by server")
        );
        assert!(actual.attribute_references().is_none());
    }
}

#[test]
fn successful_response_requires_one_identifier_then_one_or_more_references() {
    let malformed_payloads = [
        Structure::new(),
        structure([name_reference("Opaque.Alpha")]),
        structure([item(
            UNIQUE_IDENTIFIER,
            Value::text_string(OBJECT_IDENTIFIER.to_owned()),
        )]),
        structure([
            item(UNIQUE_IDENTIFIER, Value::integer(17)),
            name_reference("Opaque.Alpha"),
        ]),
        structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            ),
            item(ATTRIBUTE_REFERENCE, Value::integer(0x0042_000A)),
        ]),
        structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            ),
            item(ATTRIBUTE_REFERENCE, Value::structure(Structure::new())),
        ]),
        structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            ),
            item(
                ATTRIBUTE_REFERENCE,
                Value::structure(structure([item(
                    ATTRIBUTE_NAME,
                    Value::text_string("Opaque.Alpha".to_owned()),
                )])),
            ),
        ]),
        structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            ),
            item(
                ATTRIBUTE_REFERENCE,
                Value::structure(structure([item(
                    VENDOR_IDENTIFICATION,
                    Value::text_string(VENDOR.to_owned()),
                )])),
            ),
        ]),
        structure([
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            ),
            name_reference("Opaque.Alpha"),
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            ),
        ]),
        structure([
            name_reference("Opaque.Alpha"),
            item(
                UNIQUE_IDENTIFIER,
                Value::text_string(OBJECT_IDENTIFIER.to_owned()),
            ),
        ]),
    ];

    for payload in malformed_payloads {
        let message = response_message(0, None, None, Some(payload));
        assert!(
            GetAttributeListResponse::try_from_response_item(response_item(&message)).is_err(),
            "Table 227 requires a Unique Identifier followed by at least one valid Attribute Reference"
        );
    }
}

#[test]
fn successful_response_rejects_name_reference_with_reversed_table_161_members() {
    // OASIS §5.5 Table 161 defines the name-form members as Vendor
    // Identification followed by Attribute Name; §10.1 requires Structure
    // fields to use their order in the structure description.
    let reversed_reference = item(
        ATTRIBUTE_REFERENCE,
        Value::structure(structure([
            item(
                ATTRIBUTE_NAME,
                Value::text_string("Opaque.Alpha".to_owned()),
            ),
            item(VENDOR_IDENTIFICATION, Value::text_string(VENDOR.to_owned())),
        ])),
    );
    let message = response_message(
        0,
        None,
        None,
        Some(successful_payload([reversed_reference])),
    );

    assert!(
        GetAttributeListResponse::try_from_response_item(response_item(&message)).is_err(),
        "a name-form Attribute Reference with Attribute Name before Vendor Identification is malformed"
    );
}
