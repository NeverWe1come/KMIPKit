#![cfg(test)]

//! Derived Adjust Attribute vectors for OASIS KMIP v2.1 §6.1.3, Tables
//! 170–172, and §11.1, Tables 428–429. Traceability:
//! KMIPKIT-0016-FR-004/FR-010/FR-011 and SC-002/SC-003/SC-005.
//! These are derived structural tests; they do not claim an official OASIS
//! conformance case passed.

use crate::{
    AdjustAttributeRequest, AdjustAttributeResponse, AdjustmentType, AttributeReference,
    ResponseBatchItemView, ResponseMessage, ResultReason, ResultStatus,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const ADJUST_ATTRIBUTE_OPERATION: u32 = 0x0000_0030;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ATTRIBUTE_REFERENCE: u32 = 0x0042_013B;
const ADJUSTMENT_TYPE: u32 = 0x0042_0158;
const ADJUSTMENT_VALUE: u32 = 0x0042_0162;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const ATTRIBUTE_NAME: u32 = 0x0042_000A;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const ACTIVATION_DATE: u32 = 0x0042_0001;
const PROTECTION_PERIOD: u32 = 0x0042_0146;
const QUANTUM_SAFE: u32 = 0x0042_0147;
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

fn request(
    unique_identifier: Option<&str>,
    reference: AttributeReference,
    adjustment_type: u32,
    adjustment_value: Option<Value>,
) -> AdjustAttributeRequest {
    AdjustAttributeRequest::new(
        unique_identifier.map(str::to_owned),
        reference,
        AdjustmentType::from_raw(adjustment_type),
        adjustment_value,
    )
}

fn enumeration_value(item: &Item) -> Option<u32> {
    item.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    })
}

#[test]
fn request_follows_table_170_field_order_and_preserves_tag_reference_and_parameter() {
    // §6.1.3 Table 170 orders the optional Unique Identifier before the
    // required Attribute Reference and Adjustment Type, then optional Value.
    // §5.5 Table 161 defines tag form as an Enumeration; Table 429 assigns
    // Increment=1. Cryptographic Length is an Integer Attribute (§4.15 Table
    // 57; §11.56 Table 487), matching the supplied Integer Adjustment Value.
    // It is read-only and always required (§4.15 Table 58), so this vector
    // checks only request wire shape and preservation; it does not assert the
    // request is applicable or permitted for an object.
    let request = request(
        Some(OBJECT_IDENTIFIER),
        AttributeReference::tag(CRYPTOGRAPHIC_LENGTH),
        1,
        Some(Value::integer(7)),
    );
    let payload = request
        .to_ttlv_payload()
        .expect("the Table 170 request uses assigned tags and enum values");
    let fields = payload.view().children();

    assert_eq!(
        fields
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [
            (UNIQUE_IDENTIFIER, ItemType::TextString),
            (ATTRIBUTE_REFERENCE, ItemType::Enumeration),
            (ADJUSTMENT_TYPE, ItemType::Enumeration),
            (ADJUSTMENT_VALUE, ItemType::Integer),
        ],
        "the request contains exactly the Table 170 fields in order"
    );
    assert_eq!(
        fields[0].with_value(|value| match value {
            ValueView::TextString(value) => Some(value.to_owned()),
            _ => None,
        }),
        Some(OBJECT_IDENTIFIER.to_owned())
    );
    assert_eq!(enumeration_value(&fields[1]), Some(CRYPTOGRAPHIC_LENGTH));
    assert_eq!(enumeration_value(&fields[2]), Some(1));
    assert_eq!(
        fields[3].with_value(|value| match value {
            ValueView::Integer(value) => Some(*value),
            _ => None,
        }),
        Some(7),
        "the client preserves the supplied Adjustment Value without applying it"
    );
}

#[test]
fn request_preserves_name_reference_order_and_omits_identifier_and_adjustment_value() {
    // §5.5 Table 161 orders name-form Vendor Identification before Attribute
    // Name. Table 170 permits omitting Unique Identifier and Adjustment Value.
    let request = request(
        None,
        AttributeReference::name("KMIPKit.TestVendor_1", "Opaque.Counter"),
        2,
        None,
    );
    let payload = request
        .to_ttlv_payload()
        .expect("name-form Attribute Reference is valid");
    let fields = payload.view().children();

    assert_eq!(
        fields
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect::<Vec<_>>(),
        [
            (ATTRIBUTE_REFERENCE, ItemType::Structure),
            (ADJUSTMENT_TYPE, ItemType::Enumeration),
        ],
        "both optional fields remain absent and required fields remain ordered"
    );
    assert_eq!(enumeration_value(&fields[1]), Some(2));

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
                Some("Opaque.Counter".to_owned()),
            ),
        ])
    );
}

#[test]
fn omitted_adjustment_value_stays_absent_for_standard_attribute_type_categories() {
    // §6.1.3 Table 170 makes Adjustment Value optional. §11.1 Table 428 lists
    // Integer, Interval, and Date Time among the supported Increment/Decrement
    // categories. Its text defines omitted parameter values for numeric types,
    // Date Time, and Date Time Extended; it does not state a default for
    // Interval. This vector checks only that omission stays omitted and does
    // not materialize or calculate a default. The reference types are defined
    // by §4.15 Table 57 (Cryptographic Length, Integer; §4.15 Table 58 makes
    // it required/read-only), §4.43 Table 115 (Protection Period, Interval),
    // §4.1 Table 29 (Activation Date, Date Time), §4.45 Table 119 (Quantum
    // Safe, Boolean), and §4.9 Table 45 (Comment, Text String). Their assigned
    // tags are from §11.56. No §4 attribute has a Date Time Extended type, so
    // this test does not invent a standard tag for that parameter category.
    // Boolean/Negate covers a Table 428 adjustment with no parameter default;
    // Text String/Increment has no Table 428 applicability. Table
    // 30 constrains Activation Date modification to Pre-Active state; this
    // serializer vector does not assert that the target object is in that state.
    let vectors = [
        ("numeric", CRYPTOGRAPHIC_LENGTH, 1),
        ("interval", PROTECTION_PERIOD, 2),
        ("date time", ACTIVATION_DATE, 1),
        ("boolean", QUANTUM_SAFE, 3),
        ("other", COMMENT, 1),
    ];

    for (target_kind, target_tag, raw_adjustment_type) in vectors {
        let request = request(
            None,
            AttributeReference::tag(target_tag),
            raw_adjustment_type,
            None,
        );
        let payload = request
            .to_ttlv_payload()
            .expect("assigned Adjustment Type is encodable");
        assert_eq!(
            payload
                .view()
                .children()
                .iter()
                .map(|field| (field.tag().raw(), field.item_type()))
                .collect::<Vec<_>>(),
            [
                (ATTRIBUTE_REFERENCE, ItemType::Enumeration),
                (ADJUSTMENT_TYPE, ItemType::Enumeration),
            ],
            "omission for the {target_kind} target does not add an Adjustment Value field"
        );
        let fields = payload.view().children();
        assert_eq!(enumeration_value(&fields[0]), Some(target_tag));
        assert_eq!(enumeration_value(&fields[1]), Some(raw_adjustment_type));
    }
}

#[derive(Clone, Copy)]
enum AdjustmentParameterVector {
    Integer(i32),
    LongInteger(i64),
    BigInteger(&'static [u8]),
    Interval(u32),
    DateTime(i64),
    DateTimeExtended(i64),
    Boolean(bool),
}

impl AdjustmentParameterVector {
    fn into_value(self) -> Value {
        match self {
            Self::Integer(value) => Value::integer(value),
            Self::LongInteger(value) => Value::long_integer(value),
            Self::BigInteger(value) => Value::big_integer(value.to_vec()),
            Self::Interval(value) => Value::interval(value),
            Self::DateTime(value) => Value::date_time(value),
            Self::DateTimeExtended(value) => Value::date_time_extended(value),
            Self::Boolean(value) => Value::boolean(value),
        }
    }

    fn matches(self, item: &Item) -> bool {
        item.with_value(|actual| match (self, actual) {
            (Self::Integer(expected), ValueView::Integer(actual)) => expected == *actual,
            (Self::LongInteger(expected), ValueView::LongInteger(actual)) => expected == *actual,
            (Self::BigInteger(expected), ValueView::BigInteger(actual)) => expected == actual,
            (Self::Interval(expected), ValueView::Interval(actual)) => expected == *actual,
            (Self::DateTime(expected), ValueView::DateTime(actual)) => expected == *actual,
            (Self::DateTimeExtended(expected), ValueView::DateTimeExtended(actual)) => {
                expected == *actual
            }
            (Self::Boolean(expected), ValueView::Boolean(actual)) => expected == *actual,
            _ => false,
        })
    }
}

#[test]
fn table_428_parameters_and_table_170_optional_values_are_transmitted_unchanged() {
    // §11.1 Table 428 lists Adjustment Parameter Item Types for Increment and
    // Decrement: Integer, Long Integer, Big Integer, Interval, Date Time, and
    // Date Time Extended. It specifies no Adjustment Parameter Item Type for
    // Negate. The Negate vectors below supply values only to check generic
    // §6.1.3 Table 170 optional Adjustment Value wire preservation; they do
    // not claim a valid Table 428 Negate parameter pair or server acceptance.
    // No vector asserts server arithmetic or a resulting attribute value.
    // Raw values 1, 2, and 3 are Increment, Decrement, and Negate (§11.1
    // Table 429).
    // Standard type-matched tags are Cryptographic Length (Integer, §4.15
    // Table 57; §11.56 Table 487), Protection Period (Interval, §4.43 Table
    // 115), Activation Date (Date Time, §4.1 Table 29), and Quantum Safe
    // (Boolean, §4.45 Table 119). Cryptographic Length is read-only and always
    // required (§4.15 Table 58), and Activation Date modification is state-
    // constrained (§4.1 Table 30); these vectors assert neither writability
    // nor applicability. §7.40 Table 392 lists Long Integer for Usage Limits
    // Total as a child of the Usage Limits Structure; §4.59 Tables 148–149
    // govern the enclosing Usage Limits Attribute. The child is not used as
    // an independent Attribute Reference. The pinned §4 catalog has no
    // Attribute of type Long Integer, Big Integer, or Date Time Extended, so
    // those vectors use synthetic name-form refs only to verify generic wire
    // preservation, without claiming a known vendor attribute or standard-
    // conformance case.
    let generic_vendor = "KMIPKit.TestVendor_1";
    let vectors = [
        (
            1,
            AttributeReference::tag(CRYPTOGRAPHIC_LENGTH),
            AdjustmentParameterVector::Integer(-7),
        ),
        (
            1,
            AttributeReference::name(generic_vendor, "Opaque.GenericLongIntegerWireVector"),
            AdjustmentParameterVector::LongInteger(8_589_934_597),
        ),
        (
            1,
            AttributeReference::name(generic_vendor, "Opaque.GenericBigIntegerWireVector"),
            AdjustmentParameterVector::BigInteger(&[0x00, 0x80, 0xFF]),
        ),
        (
            1,
            AttributeReference::tag(PROTECTION_PERIOD),
            AdjustmentParameterVector::Interval(37),
        ),
        (
            1,
            AttributeReference::tag(ACTIVATION_DATE),
            AdjustmentParameterVector::DateTime(1_700_000_000),
        ),
        (
            1,
            AttributeReference::name(
                generic_vendor,
                "Opaque.GenericDateTimeExtendedIncrementWireVector",
            ),
            AdjustmentParameterVector::DateTimeExtended(1_700_000_000_234_567),
        ),
        (
            2,
            AttributeReference::name(generic_vendor, "Opaque.GenericDateTimeExtendedWireVector"),
            AdjustmentParameterVector::DateTimeExtended(1_700_000_000_123_456),
        ),
        (
            2,
            AttributeReference::tag(CRYPTOGRAPHIC_LENGTH),
            AdjustmentParameterVector::Integer(19),
        ),
        (
            2,
            AttributeReference::name(
                generic_vendor,
                "Opaque.GenericLongIntegerDecrementWireVector",
            ),
            AdjustmentParameterVector::LongInteger(9_223_372_036),
        ),
        (
            2,
            AttributeReference::name(
                generic_vendor,
                "Opaque.GenericBigIntegerDecrementWireVector",
            ),
            AdjustmentParameterVector::BigInteger(&[0x10, 0x20, 0x30]),
        ),
        (
            2,
            AttributeReference::tag(PROTECTION_PERIOD),
            AdjustmentParameterVector::Interval(43),
        ),
        (
            2,
            AttributeReference::tag(ACTIVATION_DATE),
            AdjustmentParameterVector::DateTime(1_700_000_001),
        ),
        (
            3,
            AttributeReference::tag(CRYPTOGRAPHIC_LENGTH),
            AdjustmentParameterVector::Integer(11),
        ),
        (
            3,
            AttributeReference::name(generic_vendor, "Opaque.GenericLongIntegerNegateWireVector"),
            AdjustmentParameterVector::LongInteger(-12),
        ),
        (
            3,
            AttributeReference::name(generic_vendor, "Opaque.GenericBigIntegerNegateWireVector"),
            AdjustmentParameterVector::BigInteger(&[0x01, 0x02, 0x03]),
        ),
        (
            3,
            AttributeReference::tag(QUANTUM_SAFE),
            AdjustmentParameterVector::Boolean(true),
        ),
    ];

    for (raw_adjustment_type, expected_reference, expected_value) in vectors {
        let request = request(
            None,
            expected_reference.clone(),
            raw_adjustment_type,
            Some(expected_value.into_value()),
        );
        let payload = request
            .to_ttlv_payload()
            .expect("Table 428 Adjustment Value Item Type is valid");
        let fields = payload.view().children();

        assert_eq!(
            fields
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>(),
            [ATTRIBUTE_REFERENCE, ADJUSTMENT_TYPE, ADJUSTMENT_VALUE],
            "request fields preserve §6.1.3 Table 170 order with the optional value present"
        );
        assert_eq!(fields[1].item_type(), ItemType::Enumeration);
        assert_eq!(fields[2].tag().raw(), ADJUSTMENT_VALUE);
        match expected_reference.tag_value() {
            Some(target_tag) => {
                assert_eq!(fields[0].item_type(), ItemType::Enumeration);
                assert_eq!(enumeration_value(&fields[0]), Some(target_tag));
            }
            None => {
                let (expected_vendor, expected_name) = expected_reference
                    .name_parts()
                    .expect("the vector uses a name-form reference");
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
                            Some(expected_vendor.to_owned()),
                        ),
                        (
                            ATTRIBUTE_NAME,
                            ItemType::TextString,
                            Some(expected_name.to_owned()),
                        ),
                    ])
                );
            }
        }
        assert!(
            expected_value.matches(&fields[2]),
            "the supplied Table 428 parameter keeps its exact Item Type and value"
        );
    }
}

#[test]
fn every_assigned_and_extension_boundary_adjustment_type_is_encodable_verbatim() {
    // §11.1 Table 429 assigns 1–3 and the extension range 8XXXXXXX, i.e.
    // 0x80000000–0x8FFFFFFF. No server-side operation is emulated here.
    for raw_adjustment_type in [1, 2, 3, 0x8000_0000, 0x8FFF_FFFF] {
        let adjustment_type = AdjustmentType::from_raw(raw_adjustment_type);
        assert_eq!(adjustment_type.raw(), raw_adjustment_type);

        let request = request(
            None,
            AttributeReference::tag(0x0042_002F),
            raw_adjustment_type,
            None,
        );
        let payload = request
            .to_ttlv_payload()
            .expect("Table 429 assigned and extension values are allowed");
        let fields = payload.view().children();
        assert_eq!(enumeration_value(&fields[1]), Some(raw_adjustment_type));
    }
}

#[test]
fn reserved_adjustment_type_boundaries_are_preserved_but_rejected_for_outbound_requests() {
    // Table 429 assigns only 1–3 and 0x8XXXXXXX. Values immediately outside
    // those sets, including the unsigned upper boundary, are Reserved here.
    for raw_adjustment_type in [0, 4, 0x7FFF_FFFF, 0x9000_0000, u32::MAX] {
        let adjustment_type = AdjustmentType::from_raw(raw_adjustment_type);
        assert_eq!(adjustment_type.raw(), raw_adjustment_type);

        let request = request(
            None,
            AttributeReference::tag(0x0042_002F),
            raw_adjustment_type,
            None,
        );
        assert!(
            request.to_ttlv_payload().is_err(),
            "Reserved Adjustment Type {raw_adjustment_type:#010X} cannot be sent"
        );
    }
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
        item(OPERATION, Value::enumeration(ADJUST_ATTRIBUTE_OPERATION)),
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
fn successful_response_contains_only_the_required_unique_identifier() {
    // §6.1.3 Table 171 defines Unique Identifier as the sole required response
    // field; no adjusted attribute value is returned by this operation.
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
    let response = AdjustAttributeResponse::try_from_response_item(response_item(&message))
        .expect("Table 171 response has its required Unique Identifier");

    assert_eq!(response.result().status(), ResultStatus::from_raw(0));
    assert_eq!(response.result().reason(), None);
    assert_eq!(response.unique_identifier(), Some(OBJECT_IDENTIFIER));
}

#[test]
fn response_preserves_every_table_172_failure_reason_and_message() {
    // Table 172's Operation Failed reasons: Invalid Data Type, Item Not Found,
    // Multi Valued Attribute, Numeric Range, Object Archived, Read Only
    // Attribute, Unsupported Attribute, Attestation Failed, Attestation
    // Required, Feature Not Supported, Invalid Field, Invalid Message,
    // Operation Not Supported, Permission Denied, Response Too Large, and
    // Wrong Key Lifecycle State.
    let table_172_reasons = [
        0x1C, 0x01, 0x1E, 0x1B, 0x0D, 0x1D, 0x1F, 0x15, 0x14, 0x08, 0x07, 0x04, 0x05, 0x0C, 0x02,
        0x43,
    ];

    for expected_reason in table_172_reasons {
        let message = response_message(
            1,
            Some(expected_reason),
            Some("Adjust Attribute rejected by server"),
            None,
        );
        let response = AdjustAttributeResponse::try_from_response_item(response_item(&message))
            .expect("Table 172 operation failures retain the common KMIP result");

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
            Some("Adjust Attribute rejected by server")
        );
        assert!(response.unique_identifier().is_none());
    }
}

#[test]
fn standard_reference_identity_does_not_synthesize_a_local_starting_value() {
    // §6.1.3 prose assigns an absent target value of 0 for numeric types and
    // intervals, false for Boolean, and an error for other types to server
    // processing. The Attribute Reference identifies a standard attribute by
    // its §11.56 tag: Cryptographic Length is Integer (§4.15, Table 57),
    // Protection Period is Interval (§4.43, Table 115), Quantum Safe is
    // Boolean (§4.45, Table 119), and Comment is Text String (§4.9, Table 45).
    // §6.1.3 Table 170 has no Current Attribute field. This protocol-level
    // vector checks only that the request keeps the selected tag and supplied
    // operation; it does not model remote presence or a calculated value.
    // Table 58 marks Cryptographic Length server-set, Read-Only, and always
    // required, so its case is explicitly wire-shape only: it does not claim
    // that absence is a valid object state or that the request is permitted.
    let targets = [
        ("numeric", CRYPTOGRAPHIC_LENGTH, 1),
        ("interval", PROTECTION_PERIOD, 1),
        ("boolean", QUANTUM_SAFE, 3),
        ("other", COMMENT, 1),
    ];

    for (target_kind, target_tag, raw_adjustment_type) in targets {
        let request = request(
            None,
            AttributeReference::tag(target_tag),
            raw_adjustment_type,
            None,
        );
        let payload = request
            .to_ttlv_payload()
            .expect("the typed request preserves its Attribute Reference");
        let fields = payload.view().children();

        assert_eq!(
            fields
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>(),
            [ATTRIBUTE_REFERENCE, ADJUSTMENT_TYPE],
            "the {target_kind} request contains no synthesized current or replacement value"
        );
        assert_eq!(enumeration_value(&fields[0]), Some(target_tag));
        assert_eq!(enumeration_value(&fields[1]), Some(raw_adjustment_type));
    }
}
