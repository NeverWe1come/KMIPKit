//! Locate request vectors derived from OASIS KMIP Specification v2.1
//! §6.1.28, Table 247. Search criteria are generic §4 attributes; the vectors
//! include §4.1/Table 29 dates, §4.17/Table 61 Cryptographic Usage Mask,
//! §4.31/Tables 90–92 Link, §4.59/Tables 148–149 Usage Limits, and
//! §4.60/Table 150 Vendor Attribute. Storage status bits are defined in §12.3;
//! Object Group Member values are defined in §11.33. These are local protocol
//! vectors, not official OASIS Test Cases.
//!
//! Traceability: `KMIPKIT-ELEM-OP-C2S-LOCATE`; `KMIPKIT-0017-FR-006`, FR-007,
//! FR-009; `KMIPKIT-REQ-SPEC-6.1.28-001`, -002, -007, -008-001, -008-002,
//! -011, -012, -013-001, and -013-002. The Object Group Member vector checks
//! Table 247 value preservation only; server matching remains server behavior.

use crate::{AttributeSet, AttributeSetError, LocateRequest, ObjectGroupMember, StorageStatusMask};
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
