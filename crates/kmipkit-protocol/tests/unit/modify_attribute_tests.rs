#![cfg(test)]

//! Derived Modify Attribute request/response vectors for OASIS KMIP v2.1
//! §6.1.34, Tables 265–266; §§4.2, 5.6–5.7, Tables 31–32 and 162–163;
//! §11.2 Table 430; §§11.36 and 11.56. Traceability: KMIPKIT-0016-FR-008/
//! FR-010 and SC-002/SC-003.
//! Table 267 Result Reasons and Operation Failed are checked against §§11.46–11.47,
//! Tables 479–480. T032 covers transport propagation; these vectors do not claim that
//! an official OASIS case passed.

use crate::{
    CurrentAttribute, ModifyAttributeRequest, ModifyAttributeResponse, NewAttribute,
    ResponseBatchItemView, ResponseMessage, ResultMessage, ResultReason, ResultStatus,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const MODIFY_ATTRIBUTE_OPERATION: u32 = 0x0000_000E;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CURRENT_ATTRIBUTE: u32 = 0x0042_013C;
const NEW_ATTRIBUTE: u32 = 0x0042_013D;
const ALTERNATIVE_NAME: u32 = 0x0042_00BF;
const ALTERNATIVE_NAME_VALUE: u32 = 0x0042_00C0;
const ALTERNATIVE_NAME_TYPE: u32 = 0x0042_00C1;
const UNINTERPRETED_TEXT_STRING: u32 = 0x0000_0001;
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
const MODIFY_ATTRIBUTE_ERROR_MESSAGE: &str = "Modify Attribute rejected by server";
const TABLE_267_REASONS: [(&str, u32); 14] = [
    ("Attribute Instance Not Found", 0x0000_0020),
    ("Attribute Not Found", 0x0000_0021),
    ("Attribute Read Only", 0x0000_0022),
    ("Non Unique Name Attribute", 0x0000_0035),
    ("Object Not Found", 0x0000_0037),
    ("Attestation Failed", 0x0000_0015),
    ("Attestation Required", 0x0000_0014),
    ("Feature Not Supported", 0x0000_0008),
    ("Invalid Field", 0x0000_0007),
    ("Invalid Message", 0x0000_0004),
    ("Operation Not Supported", 0x0000_0005),
    ("Permission Denied", 0x0000_000C),
    ("Response Too Large", 0x0000_0002),
    ("Wrong Key Lifecycle State", 0x0000_0043),
];

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

fn alternative_name(value: &str) -> Item {
    item(
        ALTERNATIVE_NAME,
        Value::structure(structure([
            item(ALTERNATIVE_NAME_VALUE, Value::text_string(value.to_owned())),
            item(
                ALTERNATIVE_NAME_TYPE,
                Value::enumeration(UNINTERPRETED_TEXT_STRING),
            ),
        ])),
    )
}

fn request(
    unique_identifier: Option<&str>,
    current_value: Option<&str>,
    new_value: &str,
) -> ModifyAttributeRequest {
    ModifyAttributeRequest::new(
        unique_identifier.map(str::to_owned),
        current_value.map(|value| CurrentAttribute::new(alternative_name(value))),
        NewAttribute::new(alternative_name(new_value)),
    )
}

fn wrapped_attribute_items(field: &Item) -> Option<Vec<(u32, ItemType)>> {
    field.with_value(|value| match value {
        ValueView::Structure(wrapper) => Some(
            wrapper
                .children()
                .iter()
                .map(|attribute| (attribute.tag().raw(), attribute.item_type()))
                .collect(),
        ),
        _ => None,
    })
}

fn alternative_name_members(
    field: &Item,
) -> Option<Vec<(u32, ItemType, Option<String>, Option<u32>)>> {
    field.with_value(|value| match value {
        ValueView::Structure(wrapper) => wrapper.children().first().and_then(|attribute| {
            attribute.with_value(|value| match value {
                ValueView::Structure(alternative_name) => Some(
                    alternative_name
                        .children()
                        .iter()
                        .map(|member| {
                            let text = member.with_value(|value| match value {
                                ValueView::TextString(value) => Some(value.to_owned()),
                                _ => None,
                            });
                            let enumeration = member.with_value(|value| match value {
                                ValueView::Enumeration(value) => Some(*value),
                                _ => None,
                            });
                            (member.tag().raw(), member.item_type(), text, enumeration)
                        })
                        .collect(),
                ),
                _ => None,
            })
        }),
        _ => None,
    })
}

fn assert_alternative_name_wrapper(field: &Item, expected_value: &str) {
    // Table 163/162 contains one direct attribute Item; Table 31 defines the
    // Alternative Name member order, encodings, and required fields.
    assert_eq!(
        wrapped_attribute_items(field),
        Some(vec![(ALTERNATIVE_NAME, ItemType::Structure)])
    );
    assert_eq!(
        alternative_name_members(field),
        Some(vec![
            (
                ALTERNATIVE_NAME_VALUE,
                ItemType::TextString,
                Some(expected_value.to_owned()),
                None,
            ),
            (
                ALTERNATIVE_NAME_TYPE,
                ItemType::Enumeration,
                None,
                Some(UNINTERPRETED_TEXT_STRING),
            ),
        ]),
        "the Alternative Name members retain their exact Table 31 values and order"
    );
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
        item(OPERATION, Value::enumeration(MODIFY_ATTRIBUTE_OPERATION)),
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

fn successful_response_message(payload: Structure) -> ResponseMessage {
    response_message(0, None, None, Some(payload))
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one response batch item")
}

#[test]
fn request_preserves_exact_current_and_new_items_in_table_265_order() {
    // Table 265 orders optional Unique Identifier, optional Current Attribute,
    // then required New Attribute. Alternative Name is client-modifiable and
    // allows multiple instances (§4.2 Table 32); this vector checks only the
    // exact request values and does not assert remote object state or success.
    let request = request(
        Some(OBJECT_IDENTIFIER),
        Some("selected current alternative name"),
        "caller replacement alternative name",
    );
    let payload = request
        .to_ttlv_payload()
        .expect("Table 265 request fields use allocated KMIP tags");
    let fields = payload.view().children();

    assert_eq!(
        fields
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [
            (UNIQUE_IDENTIFIER, ItemType::TextString),
            (CURRENT_ATTRIBUTE, ItemType::Structure),
            (NEW_ATTRIBUTE, ItemType::Structure),
        ],
        "request fields follow Table 265 order and cardinality"
    );
    assert_eq!(
        fields[0].with_value(|value| match value {
            ValueView::TextString(value) => Some(value.to_owned()),
            _ => None,
        }),
        Some(OBJECT_IDENTIFIER.to_owned())
    );
    assert_alternative_name_wrapper(&fields[1], "selected current alternative name");
    assert_alternative_name_wrapper(&fields[2], "caller replacement alternative name");
}

#[test]
fn request_omits_current_attribute_without_selecting_an_instance_locally() {
    // §6.1.34 leaves selection to the server when Current Attribute is absent;
    // if multiple instances exist the server returns an error. This test proves
    // omission is preserved and does not simulate object state or assert success.
    let request = request(None, None, "replacement with no selector");
    let payload = request
        .to_ttlv_payload()
        .expect("optional Unique Identifier and Current Attribute may be omitted");
    let fields = payload.view().children();

    assert_eq!(
        fields
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [(NEW_ATTRIBUTE, ItemType::Structure)],
        "omitted fields stay absent while required New Attribute remains"
    );
    assert_alternative_name_wrapper(&fields[0], "replacement with no selector");
}

#[test]
fn request_preserves_each_permitted_alternative_name_instance_selector() {
    // Alternative Name allows multiple values (§4.2 Table 32). Separate
    // vectors preserve each supplied instance selector one at a time; they do
    // not claim that both values coexist on a particular remote object.
    for current_value in ["candidate instance one", "candidate instance two"] {
        let request = request(
            Some(OBJECT_IDENTIFIER),
            Some(current_value),
            "caller replacement",
        );
        let payload = request
            .to_ttlv_payload()
            .expect("each supplied Current Attribute uses the Table 162 shape");
        let fields = payload.view().children();

        assert_eq!(
            fields
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>(),
            [UNIQUE_IDENTIFIER, CURRENT_ATTRIBUTE, NEW_ATTRIBUTE],
            "one selected instance and one replacement are represented per request"
        );
        assert_alternative_name_wrapper(&fields[1], current_value);
        assert_alternative_name_wrapper(&fields[2], "caller replacement");
    }
}

#[test]
fn successful_response_contains_the_required_unique_identifier() {
    // Table 266 defines Unique Identifier as the sole required response field.
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
    let response = ModifyAttributeResponse::try_from_response_item(response_item(&message))
        .expect("Table 266 response has its required Unique Identifier");

    assert_eq!(response.result().status(), ResultStatus::from_raw(0));
    assert_eq!(response.unique_identifier(), Some(OBJECT_IDENTIFIER));
}

#[test]
fn successful_response_without_unique_identifier_is_rejected() {
    // Table 266 requires Unique Identifier even when Result Status is Success.
    let message = successful_response_message(Structure::new());
    let response = ModifyAttributeResponse::try_from_response_item(response_item(&message));

    assert!(
        response.is_err(),
        "a successful Table 266 response without Unique Identifier is malformed"
    );
}

#[test]
fn response_preserves_all_table_267_operation_failed_reasons_and_message() {
    // Table 267 assigns Operation Failed to these 14 reasons. Their raw values
    // come from §11.46 Table 479; Operation Failed is 0x00000001 in §11.47
    // Table 480. The server's exact Result Message remains available unchanged.
    for (reason_name, raw_reason) in TABLE_267_REASONS {
        let message = response_message(
            0x0000_0001,
            Some(raw_reason),
            Some(MODIFY_ATTRIBUTE_ERROR_MESSAGE),
            None,
        );
        let response = ModifyAttributeResponse::try_from_response_item(response_item(&message))
            .expect("Table 267 failure preserves the common operation result");
        let result = response.result();

        assert_eq!(
            result.status(),
            ResultStatus::from_raw(0x0000_0001),
            "{reason_name} retains Operation Failed"
        );
        assert_eq!(
            result.reason(),
            Some(ResultReason::from_raw(raw_reason)),
            "{reason_name} retains its §11.46 Table 479 raw value"
        );
        assert_eq!(
            result.message().map(ResultMessage::as_str),
            Some(MODIFY_ATTRIBUTE_ERROR_MESSAGE),
            "{reason_name} retains the server's exact Result Message"
        );
        assert!(response.unique_identifier().is_none());
    }
}
