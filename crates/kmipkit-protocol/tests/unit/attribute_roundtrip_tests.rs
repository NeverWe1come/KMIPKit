#![cfg(test)]

//! Derived attribute preservation vectors for OASIS KMIP v2.1 §§4, 5.1,
//! 5.5–5.7, §6.1.2 Tables 167–169, §6.1.20 Table 224, §11.1 Table 429, and
//! §11.56. Traceability: KMIPKIT-0016-FR-011 and SC-005. These tests are
//! derived structural checks and do not claim official OASIS conformance.

use crate::{
    AddAttributeRequest, AdjustAttributeRequest, AdjustmentType, AttributeReference, AttributeSet,
    CurrentAttribute, GetAttributesResponse, ModifyAttributeRequest, NewAttribute,
    ResponseBatchItemView, ResponseMessage, ResultStatus,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

const GET_ATTRIBUTES_OPERATION: u32 = 0x0000_000B;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ATTRIBUTES: u32 = 0x0042_0125;
const CURRENT_ATTRIBUTE: u32 = 0x0042_013C;
const NEW_ATTRIBUTE: u32 = 0x0042_013D;
const ADJUSTMENT_TYPE: u32 = 0x0042_0158;
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
const COMMENT: u32 = 0x0042_00FD;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const EXTENSION_TAG: u32 = 0x0054_1234;
const RESERVED_TAG: u32 = 0x0042_0009;
const OBJECT_IDENTIFIER: &str = "object-id-attribute-roundtrip";

#[derive(Debug, Eq, PartialEq)]
enum CapturedValue {
    Integer(i32),
    Enumeration(u32),
    TextString(String),
    ByteString(Vec<u8>),
    Other,
}

fn tag(raw_tag: u32) -> Tag {
    RawTag::new(raw_tag)
        .expect("fixture tag fits the 24-bit KMIP field")
        .try_checked()
        .expect("fixture tag uses an assigned or accepted extension allocation")
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("fixture uses a valid checked tag and TTLV value")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut result = Structure::new();
    for child in items {
        result
            .try_push(child)
            .expect("fixture structure stays within the model depth limit");
    }
    result
}

fn capture(item: &Item) -> CapturedValue {
    item.with_value(|value| match value {
        ValueView::Integer(value) => CapturedValue::Integer(*value),
        ValueView::Enumeration(value) => CapturedValue::Enumeration(*value),
        ValueView::TextString(value) => CapturedValue::TextString(value.to_owned()),
        ValueView::ByteString(value) => CapturedValue::ByteString(value.to_vec()),
        _ => CapturedValue::Other,
    })
}

fn successful_get_attributes_payload(attributes: impl IntoIterator<Item = Item>) -> Structure {
    structure([
        item(
            UNIQUE_IDENTIFIER,
            Value::text_string(OBJECT_IDENTIFIER.to_owned()),
        ),
        item(ATTRIBUTES, Value::structure(structure(attributes))),
    ])
}

fn get_attributes_response_message(payload: Structure) -> ResponseMessage {
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
        item(OPERATION, Value::enumeration(GET_ATTRIBUTES_OPERATION)),
        item(RESULT_STATUS, Value::enumeration(0)),
        item(RESPONSE_PAYLOAD, Value::structure(payload)),
    ]);
    ResponseMessage::try_from_ttlv(structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(batch)),
    ]))
    .expect("fixture forms a valid successful KMIP 2.1 response")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one response batch item")
}

#[test]
fn name_form_reference_roundtrips_an_unknown_attribute_name_verbatim() {
    // OASIS KMIP v2.1 §5.5, Table 161 represents the name form with exact
    // Vendor Identification and Attribute Name Text Strings. KMIPKit must not
    // normalize or synthesize a tag for an unrecognized name.
    let original_name = "Future Attribute/Case-μ  ";
    let reference = AttributeReference::name("KMIPKit.TestVendor_1", original_name);

    let encoded = reference
        .to_ttlv_item()
        .expect("the name-form reference is structurally valid");
    let decoded = AttributeReference::try_from_ttlv_item(&encoded)
        .expect("the generated Table 161 item parses as a name-form reference");

    assert_eq!(
        decoded.name_parts(),
        Some(("KMIPKit.TestVendor_1", original_name)),
        "unknown Attribute Name text, including case, Unicode, and trailing spaces, is exact"
    );
}

#[test]
fn add_attribute_preserves_assigned_and_extension_tagged_new_values() {
    // OASIS KMIP v2.1 §5.7, Table 163 defines New Attribute as one direct
    // generic Item. §6.1.2, Table 167 carries it in Add Attribute. Assigned
    // tags and §11.56 Table 487 extension Tags keep their complete tag and value under
    // the accepted KMIPKIT-0004/ADR-0010 allocation policy.
    let cases = [
        (
            COMMENT,
            Value::text_string("assigned-value".to_owned()),
            CapturedValue::TextString("assigned-value".to_owned()),
        ),
        (
            EXTENSION_TAG,
            Value::byte_string(vec![0x00, 0x7F, 0xFF]),
            CapturedValue::ByteString(vec![0x00, 0x7F, 0xFF]),
        ),
    ];

    for (raw_tag, value, expected_value) in cases {
        let request = AddAttributeRequest::new(None, NewAttribute::new(item(raw_tag, value)));
        let payload = request
            .to_ttlv_payload()
            .expect("the caller-supplied New Attribute remains a valid generic Item");
        let payload_view = payload.view();
        let fields = payload_view.children();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].tag().raw(), NEW_ATTRIBUTE);

        let inner = fields[0].with_value(|value| match value {
            ValueView::Structure(attribute) => attribute
                .children()
                .first()
                .map(|attribute| (attribute.tag().raw(), capture(attribute))),
            _ => None,
        });
        assert_eq!(
            inner,
            Some((raw_tag, expected_value)),
            "the direct New Attribute Item must retain its tag and raw value"
        );
    }
}

#[test]
fn modify_attribute_preserves_current_and_new_items_with_allocated_tags() {
    // OASIS KMIP v2.1 §§5.6–5.7, Tables 162–163 define Current and New
    // Attribute as wrappers around one direct generic Item; §6.1.34 Table
    // 265 carries both. The assigned Comment tag and accepted §11.56 extension
    // tag retain their caller-supplied values independently.
    let request = ModifyAttributeRequest::new(
        None,
        Some(CurrentAttribute::new(item(
            COMMENT,
            Value::text_string("previous".to_owned()),
        ))),
        NewAttribute::new(item(EXTENSION_TAG, Value::byte_string(vec![0xCA, 0xFE]))),
    );
    let payload = request
        .to_ttlv_payload()
        .expect("both direct Items use assigned or accepted extension tags");

    let preserved = payload
        .view()
        .children()
        .iter()
        .map(|wrapper| {
            let direct = wrapper.with_value(|value| match value {
                ValueView::Structure(attribute) => attribute
                    .children()
                    .first()
                    .map(|item| (item.tag().raw(), capture(item))),
                _ => None,
            });
            (wrapper.tag().raw(), direct)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        preserved,
        [
            (
                CURRENT_ATTRIBUTE,
                Some((COMMENT, CapturedValue::TextString("previous".to_owned())))
            ),
            (
                NEW_ATTRIBUTE,
                Some((EXTENSION_TAG, CapturedValue::ByteString(vec![0xCA, 0xFE])))
            ),
        ]
    );
}

#[test]
fn get_attributes_response_preserves_repeated_instances_in_wire_order() {
    // OASIS KMIP v2.1 §6.1.20, Table 224 carries the Attributes Structure;
    // §§5.1–5.4, Tables 157–160 define its direct attribute Items. Repeated
    // values remain distinct and keep their response order in the typed model.
    let message = get_attributes_response_message(successful_get_attributes_payload([
        item(COMMENT, Value::text_string("first".to_owned())),
        item(EXTENSION_TAG, Value::byte_string(vec![0xA1, 0xB2])),
        item(COMMENT, Value::text_string("second".to_owned())),
    ]));
    let response = GetAttributesResponse::try_from_response_item(response_item(&message))
        .expect("successful Table 224 response contains its identifier and attributes");
    assert_eq!(response.result().status(), ResultStatus::from_raw(0));

    let attributes = response
        .attributes()
        .expect("successful response exposes its Attributes field");
    assert_eq!(
        attributes
            .as_items()
            .iter()
            .map(|attribute| (attribute.tag().raw(), capture(attribute)))
            .collect::<Vec<_>>(),
        [
            (COMMENT, CapturedValue::TextString("first".to_owned())),
            (EXTENSION_TAG, CapturedValue::ByteString(vec![0xA1, 0xB2])),
            (COMMENT, CapturedValue::TextString("second".to_owned())),
        ]
    );
}

#[test]
fn generic_attribute_ttlv_retains_unknown_enumeration_values() {
    // OASIS KMIP v2.1 §4.13 Table 53 declares Cryptographic Algorithm as an
    // Enumeration attribute; §11.1 Table 429 describes assigned and
    // extension codes. FR-011 keeps all generic Enumeration values lossless,
    // including codes a typed outbound Adjustment Type must reject.
    let unknown_value = 0x7FFF_FFFF;
    let attributes = AttributeSet::try_new([item(
        CRYPTOGRAPHIC_ALGORITHM,
        Value::enumeration(unknown_value),
    )])
    .expect("the generic model accepts the catalogued Enumeration TTLV type");

    assert_eq!(
        capture(&attributes.as_items()[0]),
        CapturedValue::Enumeration(unknown_value)
    );
}

#[test]
fn adjust_attribute_preserves_valid_extension_enumeration_values() {
    // OASIS KMIP v2.1 §6.1.3, Table 170 encodes Adjustment Type as an
    // Enumeration; §11.1, Table 429 assigns 1–3 and the 0x80000000–0x8FFFFFFF
    // extension range. The request preserves each supplied extension code.
    for extension_value in [0x8000_0000, 0x8ABC_DEF0, 0x8FFF_FFFF] {
        let request = AdjustAttributeRequest::new(
            None,
            AttributeReference::tag(CRYPTOGRAPHIC_LENGTH),
            AdjustmentType::from_raw(extension_value),
            None,
        );
        let payload = request
            .to_ttlv_payload()
            .expect("assigned or §11.1 extension Adjustment Type is encodable");
        let payload_view = payload.view();
        let adjustment = payload_view
            .children()
            .iter()
            .find(|field| field.tag().raw() == ADJUSTMENT_TYPE)
            .expect("Table 170 requires Adjustment Type");

        assert_eq!(
            adjustment.with_value(|value| match value {
                ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            }),
            Some(extension_value)
        );
    }
}

#[test]
fn adjust_attribute_rejects_reserved_attribute_reference_tags() {
    // OASIS KMIP v2.1 §11.56 Table 487 classifies the KMIP Tag values. The outbound
    // rejection of Reserved or unallocated raw tags follows KMIPKIT-0004 and
    // accepted ADR-0010 allocation policy; this is not an OASIS conformance
    // assertion about a receiver's behavior.
    let request = AdjustAttributeRequest::new(
        None,
        AttributeReference::tag(RESERVED_TAG),
        AdjustmentType::INCREMENT,
        None,
    );

    assert!(
        request.to_ttlv_payload().is_err(),
        "a Reserved raw Tag cannot be emitted in an Attribute Reference"
    );
}

#[test]
fn adjust_attribute_rejects_reserved_adjustment_enumerations() {
    // OASIS KMIP v2.1 §11.1, Table 429 leaves unlisted values outside its
    // extension range Reserved for future versions. FR-011 preserves such
    // codes generically but prohibits emitting them as typed Adjustment Type.
    for reserved_value in [0x0000_0000, 0x0000_0004, 0x7FFF_FFFF, 0x9000_0000] {
        let request = AdjustAttributeRequest::new(
            None,
            AttributeReference::tag(CRYPTOGRAPHIC_LENGTH),
            AdjustmentType::from_raw(reserved_value),
            None,
        );

        assert!(
            request.to_ttlv_payload().is_err(),
            "Reserved Adjustment Type {reserved_value:#010X} cannot be emitted"
        );
    }
}
