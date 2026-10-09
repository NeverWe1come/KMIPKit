#![cfg(test)]

//! Derived Add Attribute request vectors for OASIS KMIP v2.1 §6.1.2,
//! Tables 167–169, using the New Attribute structure in §5.7, Table 163.
//! Traceability: KMIPKIT-0016-FR-003/FR-010 and SC-002/SC-003.
//! These structural vectors do not claim that an official OASIS case passed.

use crate::{AddAttributeRequest, NewAttribute};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const NEW_ATTRIBUTE: u32 = 0x0042_013D;
const COMMENT: u32 = 0x0042_00FD;
const VENDOR_ATTRIBUTE: u32 = 0x0042_0008;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const ATTRIBUTE_NAME: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE: u32 = 0x0042_000B;
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
    let fields = payload.view().children();

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
