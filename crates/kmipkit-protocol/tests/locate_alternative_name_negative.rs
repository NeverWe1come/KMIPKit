//! Tape Library Alternative Name negative evidence (KMIPKIT-0125-FR-006).
//!
//! OASIS KMIP Profiles v2.1 §5.12.3 (`KMIPKIT-REQ-PROF-5.12.3-003`) forbids a
//! Tape Library client from using the barcode in Alternative Name to locate an
//! object for tape encryption or decryption, because the barcode is not
//! guaranteed unique. `KMIPKit` is a generic client: it sends only the criteria
//! the caller supplies and never derives a Locate criterion itself. These tests
//! show the library injects no Alternative Name into a Locate payload, so the
//! prohibition is the Tape Library application's responsibility. They make no
//! Tape Library profile-conformance claim.

use kmipkit_protocol::{AttributeSet, LocateRequest, ObjectGroupMember, StorageStatusMask};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

const ALTERNATIVE_NAME: u32 = 0x0042_00BF;
const ATTRIBUTES: u32 = 0x0042_0125;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("test tag is in range")
        .try_checked()
        .expect("test tag is allocated")
}

fn contains_tag(items: &[Item], wanted: u32) -> bool {
    items.iter().any(|item| {
        item.tag().raw() == wanted
            || item.with_value(|value| match value {
                ValueView::Structure(structure) => contains_tag(structure.children(), wanted),
                _ => false,
            })
    })
}

fn payload_children(request: LocateRequest) -> Vec<Item> {
    let payload = request.to_ttlv_payload().expect("payload builds");
    let mut children = Vec::new();
    for child in payload.view().children() {
        children.push(child.try_clone().expect("child clones"));
    }
    children
}

#[test]
fn given_empty_criteria_when_locate_payload_is_built_then_no_alternative_name_is_sent() {
    let children = payload_children(LocateRequest::new(AttributeSet::new()));

    assert!(!contains_tag(&children, ALTERNATIVE_NAME));
    assert!(
        contains_tag(&children, ATTRIBUTES),
        "Attributes Structure stays required"
    );
}

#[test]
fn given_every_optional_field_when_locate_payload_is_built_then_no_alternative_name_is_sent() {
    let request = LocateRequest::new(AttributeSet::new())
        .with_maximum_items(8)
        .with_offset_items(0)
        .with_storage_status_mask(StorageStatusMask::from_raw(1))
        .with_object_group_member(ObjectGroupMember::from_raw(2));

    assert!(!contains_tag(&payload_children(request), ALTERNATIVE_NAME));
}

#[test]
fn given_caller_supplied_alternative_name_when_payload_is_built_then_it_is_forwarded_unchanged() {
    let mut attributes = AttributeSet::new();
    attributes
        .try_push(
            Item::new(tag(ALTERNATIVE_NAME), Value::structure(Structure::new()))
                .expect("item builds"),
        )
        .expect("attribute is accepted");

    let children = payload_children(LocateRequest::new(attributes));

    assert!(contains_tag(&children, ALTERNATIVE_NAME));
}
