//! Locate request vectors derived from OASIS KMIP Specification v2.1
//! §6.1.28, Table 247. Search criteria are generic §4 attributes; the vectors
//! include §4.1/Table 29 dates, §4.17/Table 61 Cryptographic Usage Mask,
//! §4.31/Tables 90–92 Link, §4.59/Tables 148–149 Usage Limits, and
//! §4.60/Table 150 Vendor Attribute. Storage status bits are defined in §12.3;
//! Object Group Member values are defined in §11.33. These are local protocol
//! vectors, not official OASIS Test Cases.
//!
//! Traceability: `KMIPKIT-ELEM-OP-C2S-LOCATE`; `KMIPKIT-0017-FR-006`–FR-009,
//! FR-011; `KMIPKIT-REQ-SPEC-6.1.28-001`, -002, -007, -008-001, -008-002,
//! -011, -012, -013-001, and -013-002. The Object Group Member vector checks
//! Table 247 value preservation only; server matching remains server behavior.

use crate::{
    AttributeSet, AttributeSetError, LocateError, LocateRequest, LocateResponse, ObjectGroupMember,
    ResponseBatchItemView, ResponseMessage, ResultReason, ResultStatus, StorageStatusMask,
    UniqueIdentifier,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const MAXIMUM_ITEMS_TAG: u32 = 0x0042_004F;
const OFFSET_ITEMS_TAG: u32 = 0x0042_00D4;
const STORAGE_STATUS_MASK_TAG: u32 = 0x0042_008E;
const OBJECT_GROUP_MEMBER_TAG: u32 = 0x0042_00AC;
const ATTRIBUTES_TAG: u32 = 0x0042_0125;

const VENDOR_ATTRIBUTE_TAG: u32 = 0x0042_0008;
const ACTIVATION_DATE_TAG: u32 = 0x0042_0001;
const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE_TAG: u32 = 0x0042_000B;
const CRYPTOGRAPHIC_USAGE_MASK_TAG: u32 = 0x0042_002C;
const LINK_TAG: u32 = 0x0042_004A;
const LINKED_OBJECT_IDENTIFIER_TAG: u32 = 0x0042_004C;
const USAGE_LIMITS_TAG: u32 = 0x0042_0095;
const USAGE_LIMITS_COUNT_TAG: u32 = 0x0042_0096;
const USAGE_LIMITS_TOTAL_TAG: u32 = 0x0042_0097;
const USAGE_LIMITS_UNIT_TAG: u32 = 0x0042_0098;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;

const LOCATE_OPERATION: u32 = 0x0000_0008;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const LOCATED_ITEMS_TAG: u32 = 0x0042_00D5;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const RESPONSE_HEADER_TAG: u32 = 0x0042_007A;
const PROTOCOL_VERSION_TAG: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR_TAG: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR_TAG: u32 = 0x0042_006B;
const TIME_STAMP_TAG: u32 = 0x0042_0092;
const BATCH_COUNT_TAG: u32 = 0x0042_000D;
const BATCH_ITEM_TAG: u32 = 0x0042_000F;
const OPERATION_TAG: u32 = 0x0042_005C;
const RESULT_STATUS_TAG: u32 = 0x0042_007F;
const RESULT_REASON_TAG: u32 = 0x0042_007E;
const RESPONSE_PAYLOAD_TAG: u32 = 0x0042_007C;

#[derive(Debug, Eq, PartialEq)]
enum ValueSnapshot {
    Structure(Vec<FieldSnapshot>),
    Integer(i32),
    LongInteger(i64),
    Enumeration(u32),
    TextString(String),
    DateTime(i64),
}

type FieldSnapshot = (u32, ItemType, ValueSnapshot);

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

fn field_snapshots(structure: &Structure) -> Vec<FieldSnapshot> {
    structure
        .view()
        .children()
        .iter()
        .map(item_snapshot)
        .collect()
}

fn item_snapshot(item: &Item) -> FieldSnapshot {
    (
        item.tag().raw(),
        item.item_type(),
        item.with_value(value_snapshot),
    )
}

fn value_snapshot(value: ValueView<'_>) -> ValueSnapshot {
    match value {
        ValueView::Structure(structure) => {
            ValueSnapshot::Structure(structure.children().iter().map(item_snapshot).collect())
        }
        ValueView::Integer(value) => ValueSnapshot::Integer(*value),
        ValueView::LongInteger(value) => ValueSnapshot::LongInteger(*value),
        ValueView::Enumeration(value) => ValueSnapshot::Enumeration(*value),
        ValueView::TextString(value) => ValueSnapshot::TextString(value.to_owned()),
        ValueView::DateTime(value) => ValueSnapshot::DateTime(*value),
        _ => panic!("Locate request fixture contains only asserted TTLV values"),
    }
}

fn payload(request: LocateRequest) -> Structure {
    request
        .to_ttlv_payload()
        .expect("the supplied Table 247 request is representable")
}

fn vendor_attribute(fields: impl IntoIterator<Item = Item>) -> Item {
    item(VENDOR_ATTRIBUTE_TAG, Value::structure(structure(fields)))
}

fn response_message(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> ResponseMessage {
    let protocol_version = structure([
        item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    let response_header = structure([
        item(PROTOCOL_VERSION_TAG, Value::structure(protocol_version)),
        item(TIME_STAMP_TAG, Value::date_time(1)),
        item(BATCH_COUNT_TAG, Value::integer(1)),
    ]);
    let mut batch_fields = vec![
        item(OPERATION_TAG, Value::enumeration(LOCATE_OPERATION)),
        item(RESULT_STATUS_TAG, Value::enumeration(status)),
    ];
    if let Some(reason) = reason {
        batch_fields.push(item(RESULT_REASON_TAG, Value::enumeration(reason)));
    }
    if let Some(payload) = payload {
        batch_fields.push(item(RESPONSE_PAYLOAD_TAG, Value::structure(payload)));
    }

    ResponseMessage::try_from_ttlv(structure([
        item(RESPONSE_HEADER_TAG, Value::structure(response_header)),
        item(BATCH_ITEM_TAG, Value::structure(structure(batch_fields))),
    ]))
    .expect("fixture is a valid KMIP 2.1 Locate response message")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one validated Locate response batch item")
}

fn decode_locate(message: &ResponseMessage) -> Result<LocateResponse, LocateError> {
    LocateResponse::try_from_response_item(response_item(message))
}

fn locate_response_payload(
    located_items: Option<i32>,
    identifiers: impl IntoIterator<Item = Value>,
) -> Structure {
    let mut fields = Vec::new();
    if let Some(located_items) = located_items {
        fields.push(item(LOCATED_ITEMS_TAG, Value::integer(located_items)));
    }
    fields.extend(
        identifiers
            .into_iter()
            .map(|identifier| item(UNIQUE_IDENTIFIER_TAG, identifier)),
    );
    structure(fields)
}

#[test]
fn request_always_encodes_the_required_empty_attributes_structure() {
    let payload = payload(LocateRequest::new(AttributeSet::new()));

    assert_eq!(
        field_snapshots(&payload),
        [(
            ATTRIBUTES_TAG,
            ItemType::Structure,
            ValueSnapshot::Structure(vec![]),
        )]
    );
}

#[test]
fn request_emits_supplied_optional_fields_in_table_247_order() {
    let attributes = AttributeSet::try_new([item(ACTIVATION_DATE_TAG, Value::date_time(100))])
        .expect("Activation Date is a catalogued Date-Time attribute");
    let payload = payload(
        LocateRequest::new(attributes)
            .with_maximum_items(3)
            .with_offset_items(0)
            .with_storage_status_mask(StorageStatusMask::from_raw(0x0000_0003))
            .with_object_group_member(ObjectGroupMember::from_raw(1)),
    );

    assert_eq!(
        field_snapshots(&payload),
        [
            (
                MAXIMUM_ITEMS_TAG,
                ItemType::Integer,
                ValueSnapshot::Integer(3),
            ),
            (
                OFFSET_ITEMS_TAG,
                ItemType::Integer,
                ValueSnapshot::Integer(0),
            ),
            (
                STORAGE_STATUS_MASK_TAG,
                ItemType::Integer,
                ValueSnapshot::Integer(3),
            ),
            (
                OBJECT_GROUP_MEMBER_TAG,
                ItemType::Enumeration,
                ValueSnapshot::Enumeration(1),
            ),
            (
                ATTRIBUTES_TAG,
                ItemType::Structure,
                ValueSnapshot::Structure(vec![(
                    ACTIVATION_DATE_TAG,
                    ItemType::DateTime,
                    ValueSnapshot::DateTime(100),
                )]),
            ),
        ]
    );
}

#[test]
fn storage_status_mask_preserves_assigned_and_unknown_bits() {
    for raw_mask in [0x0000_0001, 0x0000_0002, 0x0000_0004, 0x8000_0000] {
        let payload = payload(
            LocateRequest::new(AttributeSet::new())
                .with_storage_status_mask(StorageStatusMask::from_raw(raw_mask)),
        );

        assert_eq!(
            field_snapshots(&payload),
            [
                (
                    STORAGE_STATUS_MASK_TAG,
                    ItemType::Integer,
                    ValueSnapshot::Integer(raw_mask as i32),
                ),
                (
                    ATTRIBUTES_TAG,
                    ItemType::Structure,
                    ValueSnapshot::Structure(vec![]),
                ),
            ]
        );
    }
}

#[test]
fn object_group_member_preserves_assigned_and_unknown_enumerations() {
    for raw_value in [1, 2, u32::MAX] {
        let payload = payload(
            LocateRequest::new(AttributeSet::new())
                .with_object_group_member(ObjectGroupMember::from_raw(raw_value)),
        );

        assert_eq!(
            field_snapshots(&payload),
            [
                (
                    OBJECT_GROUP_MEMBER_TAG,
                    ItemType::Enumeration,
                    ValueSnapshot::Enumeration(raw_value),
                ),
                (
                    ATTRIBUTES_TAG,
                    ItemType::Structure,
                    ValueSnapshot::Structure(vec![]),
                ),
            ]
        );
    }
}

#[test]
fn attribute_set_rejects_catalogued_type_mismatch_in_try_new_and_try_push() {
    let wrong_type = || item(ACTIVATION_DATE_TAG, Value::integer(100));

    assert_eq!(
        AttributeSet::try_new([wrong_type()]).expect_err("Date-Time is required by the catalog"),
        AttributeSetError::AttributeTtlvTypeMismatch,
    );

    let mut attributes = AttributeSet::new();
    assert_eq!(
        attributes
            .try_push(wrong_type())
            .expect_err("try_push applies the catalogued Date-Time type"),
        AttributeSetError::AttributeTtlvTypeMismatch,
    );
}

#[test]
fn attribute_set_rejects_vendor_attribute_shape_and_table_150_order() {
    assert_eq!(
        AttributeSet::try_new([item(VENDOR_ATTRIBUTE_TAG, Value::integer(7))])
            .expect_err("Vendor Attribute is a Table 150 Structure"),
        AttributeSetError::VendorAttributeMustBeStructure,
    );

    let missing_value = vendor_attribute([
        item(
            VENDOR_IDENTIFICATION_TAG,
            Value::text_string("KMIPKit.TestVendor".to_owned()),
        ),
        item(
            ATTRIBUTE_NAME_TAG,
            Value::text_string("Opaque.Attribute".to_owned()),
        ),
    ]);
    assert_eq!(
        AttributeSet::try_new([missing_value])
            .expect_err("Table 150 requires a Vendor Attribute Value"),
        AttributeSetError::MissingAttributeValue,
    );

    let wrong_order = vendor_attribute([
        item(
            ATTRIBUTE_NAME_TAG,
            Value::text_string("Opaque.Attribute".to_owned()),
        ),
        item(
            VENDOR_IDENTIFICATION_TAG,
            Value::text_string("KMIPKit.TestVendor".to_owned()),
        ),
        item(ATTRIBUTE_VALUE_TAG, Value::byte_string(vec![0x80])),
    ]);
    let mut attributes = AttributeSet::new();
    assert_eq!(
        attributes
            .try_push(wrong_order)
            .expect_err("Table 150 members must retain their source order"),
        AttributeSetError::VendorAttributeFieldOrder,
    );
}

#[test]
fn request_preserves_structured_repeated_and_usage_criteria_for_server_matching() {
    let partial_link = item(
        LINK_TAG,
        Value::structure(structure([item(
            LINKED_OBJECT_IDENTIFIER_TAG,
            Value::text_string("linked-object-7".to_owned()),
        )])),
    );
    let mut attributes = AttributeSet::try_new([
        partial_link,
        item(ACTIVATION_DATE_TAG, Value::date_time(100)),
    ])
    .expect("Link may specify Linked Object Identifier without Link Type");
    attributes
        .try_push(item(ACTIVATION_DATE_TAG, Value::date_time(200)))
        .expect("AttributeSet preserves a second Date value as a range criterion");
    attributes
        .try_push(item(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::integer(5)))
        .expect("Cryptographic Usage Mask uses its catalogued Integer encoding");
    attributes
        .try_push(item(
            USAGE_LIMITS_TAG,
            Value::structure(structure([
                item(USAGE_LIMITS_TOTAL_TAG, Value::long_integer(100)),
                item(USAGE_LIMITS_COUNT_TAG, Value::long_integer(73)),
                item(USAGE_LIMITS_UNIT_TAG, Value::enumeration(1)),
            ])),
        ))
        .expect("Usage Limits retains the Table 392 structure values");

    let payload = payload(LocateRequest::new(attributes));

    assert_eq!(
        field_snapshots(&payload),
        [(
            ATTRIBUTES_TAG,
            ItemType::Structure,
            ValueSnapshot::Structure(vec![
                (
                    LINK_TAG,
                    ItemType::Structure,
                    ValueSnapshot::Structure(vec![(
                        LINKED_OBJECT_IDENTIFIER_TAG,
                        ItemType::TextString,
                        ValueSnapshot::TextString("linked-object-7".to_owned()),
                    )]),
                ),
                (
                    ACTIVATION_DATE_TAG,
                    ItemType::DateTime,
                    ValueSnapshot::DateTime(100),
                ),
                (
                    ACTIVATION_DATE_TAG,
                    ItemType::DateTime,
                    ValueSnapshot::DateTime(200),
                ),
                (
                    CRYPTOGRAPHIC_USAGE_MASK_TAG,
                    ItemType::Integer,
                    ValueSnapshot::Integer(5),
                ),
                (
                    USAGE_LIMITS_TAG,
                    ItemType::Structure,
                    ValueSnapshot::Structure(vec![
                        (
                            USAGE_LIMITS_TOTAL_TAG,
                            ItemType::LongInteger,
                            ValueSnapshot::LongInteger(100),
                        ),
                        (
                            USAGE_LIMITS_COUNT_TAG,
                            ItemType::LongInteger,
                            ValueSnapshot::LongInteger(73),
                        ),
                        (
                            USAGE_LIMITS_UNIT_TAG,
                            ItemType::Enumeration,
                            ValueSnapshot::Enumeration(1),
                        ),
                    ]),
                ),
            ]),
        )]
    );
}

#[test]
fn successful_table_248_response_preserves_located_items_presence_separately_from_identifiers() {
    // Table 248 makes Located Items optional and permits zero Unique Identifier items.
    // The total count is independent of how many identifiers this response carries.
    let cases = [(None, None), (Some(0), Some(0)), (Some(37), Some(37))];

    for (wire_count, expected_count) in cases {
        let payload = locate_response_payload(wire_count, []);
        let message = response_message(SUCCESS, None, Some(payload));
        let response = decode_locate(&message)
            .expect("Table 248 accepts an empty identifier list and optional Located Items");

        assert_eq!(response.located_items(), expected_count);
        assert!(response.unique_identifiers().is_empty());
    }
}

#[test]
fn successful_table_248_response_preserves_a_single_identifier_without_located_items() {
    let payload = locate_response_payload(
        None,
        [Value::text_string("single-object-identifier".to_owned())],
    );
    let message = response_message(SUCCESS, None, Some(payload));
    let response = decode_locate(&message)
        .expect("Table 248 permits an identifier when Located Items is omitted");
    let expected_identifiers = [UniqueIdentifier::TextString(
        "single-object-identifier".to_owned(),
    )];

    assert_eq!(response.located_items(), None);
    assert_eq!(
        response.unique_identifiers(),
        expected_identifiers.as_slice()
    );
}

#[test]
fn successful_table_248_response_preserves_identifier_forms_duplicates_and_wire_order() {
    // §4.58 Tables 145–146 permits these three raw identifier forms. The Enumeration
    // value is deliberately unknown to this release and must remain uninterpreted.
    let payload = locate_response_payload(
        Some(37),
        [
            Value::text_string("z-last-on-purpose".to_owned()),
            Value::enumeration(0xF123_4567),
            Value::integer(-12_345),
            Value::text_string("repeated-identifier".to_owned()),
            Value::text_string("repeated-identifier".to_owned()),
        ],
    );
    let message = response_message(SUCCESS, None, Some(payload));
    let response =
        decode_locate(&message).expect("Table 248 repetitions retain their wire values and order");
    let expected_identifiers = [
        UniqueIdentifier::TextString("z-last-on-purpose".to_owned()),
        UniqueIdentifier::Enumeration(0xF123_4567),
        UniqueIdentifier::Integer(-12_345),
        UniqueIdentifier::TextString("repeated-identifier".to_owned()),
        UniqueIdentifier::TextString("repeated-identifier".to_owned()),
    ];

    assert_eq!(response.located_items(), Some(37));
    assert_eq!(
        response.unique_identifiers(),
        expected_identifiers.as_slice()
    );
}

#[test]
fn failed_table_249_response_preserves_each_locate_result_reason() {
    // OASIS KMIP 2.1 §6.1.28.1, Table 249 lists these Locate Operation Failed reasons.
    let table_249_reasons = [
        ("Invalid Attribute", 0x0000_002C),
        ("Attestation Failed", 0x0000_0015),
        ("Attestation Required", 0x0000_0014),
        ("Feature Not Supported", 0x0000_0008),
        ("Invalid Field", 0x0000_0007),
        ("Invalid Message", 0x0000_0004),
        ("Operation Not Supported", 0x0000_0005),
        ("Permission Denied", 0x0000_000C),
        ("Response Too Large", 0x0000_0002),
    ];

    for (reason_name, reason_raw) in table_249_reasons {
        let message = response_message(OPERATION_FAILED, Some(reason_raw), None);
        let response =
            decode_locate(&message).expect("a Table 249 Locate error remains an operation result");

        assert_eq!(
            response.result().status(),
            ResultStatus::from_raw(OPERATION_FAILED),
            "{reason_name}"
        );
        assert_eq!(
            response.result().reason(),
            Some(ResultReason::from_raw(reason_raw)),
            "{reason_name}"
        );
        assert_eq!(response.located_items(), None, "{reason_name}");
        assert!(response.unique_identifiers().is_empty(), "{reason_name}");
    }
}

#[test]
fn failed_locate_response_preserves_an_unknown_result_reason_raw_value() {
    const UNKNOWN_RESULT_REASON: u32 = 0xF123_4569;

    let message = response_message(OPERATION_FAILED, Some(UNKNOWN_RESULT_REASON), None);
    let response = decode_locate(&message)
        .expect("the public result contract retains unknown Result Reason values");

    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(UNKNOWN_RESULT_REASON)
    );
    assert_eq!(response.located_items(), None);
    assert!(response.unique_identifiers().is_empty());
}
