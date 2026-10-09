use kmipkit_ttlv::{Item, ItemType, ValueView};

use crate::lifecycle_fixtures::{
    request_payload, success_payload_duplicate_identifier, success_payload_missing_identifier,
    success_payload_wrong_identifier_type, successful_response_payload,
};
use crate::UniqueIdentifier;

const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;

fn identifier_forms() -> [UniqueIdentifier; 3] {
    [
        UniqueIdentifier::TextString("object-identifier".to_owned()),
        UniqueIdentifier::Enumeration(0xA1B2_C3D4),
        UniqueIdentifier::Integer(-12_345),
    ]
}

fn assert_identifier(item: &Item, expected: &UniqueIdentifier) {
    assert_eq!(item.tag().raw(), UNIQUE_IDENTIFIER_TAG);

    let (expected_type, value_matches) = item.with_value(|value| match (expected, value) {
        (UniqueIdentifier::TextString(expected), ValueView::TextString(actual)) => {
            (ItemType::TextString, actual == expected)
        }
        (UniqueIdentifier::Enumeration(expected), ValueView::Enumeration(actual)) => {
            (ItemType::Enumeration, actual == expected)
        }
        (UniqueIdentifier::Integer(expected), ValueView::Integer(actual)) => {
            (ItemType::Integer, actual == expected)
        }
        _ => (item.item_type(), false),
    });

    assert_eq!(item.item_type(), expected_type);
    assert!(value_matches, "identifier wire value must be preserved");
}

#[test]
fn request_payload_omits_an_unsupplied_optional_identifier() {
    let payload = request_payload(None);

    assert!(payload.view().children().is_empty());
}

#[test]
fn request_payload_preserves_each_identifier_wire_representation() {
    for identifier in identifier_forms() {
        let payload = request_payload(Some(identifier.clone()));
        let fields = payload.view().children();

        assert_eq!(fields.len(), 1);
        assert_identifier(&fields[0], &identifier);
    }
}

#[test]
fn successful_response_payload_preserves_each_identifier_wire_representation() {
    for identifier in identifier_forms() {
        let payload = successful_response_payload(identifier.clone());
        let fields = payload.view().children();

        assert_eq!(fields.len(), 1);
        assert_identifier(&fields[0], &identifier);
    }
}

#[test]
fn malformed_success_payload_can_omit_the_required_identifier() {
    let payload = success_payload_missing_identifier();

    assert!(payload.view().children().is_empty());
}

#[test]
fn malformed_success_payload_can_use_a_wrong_identifier_item_type() {
    let payload = success_payload_wrong_identifier_type();
    let fields = payload.view().children();

    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), UNIQUE_IDENTIFIER_TAG);
    assert_eq!(fields[0].item_type(), ItemType::DateTime);
}

#[test]
fn malformed_success_payload_can_duplicate_the_identifier() {
    let identifier = UniqueIdentifier::TextString("object-identifier".to_owned());
    let payload = success_payload_duplicate_identifier(identifier.clone());
    let fields = payload.view().children();

    assert_eq!(fields.len(), 2);
    assert_identifier(&fields[0], &identifier);
    assert_identifier(&fields[1], &identifier);
}
