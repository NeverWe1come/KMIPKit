use crate::codec::encode_ttlv_item;
use crate::{Item, ItemType, RawTag, Value, ValueView};

#[test]
fn encoded_item_round_trips_through_the_bounded_decoder() {
    let item = Item::new(
        RawTag::new(0x0042_0173).unwrap().try_checked().unwrap(),
        Value::text_string("kmip".to_owned()),
    )
    .unwrap();
    let mut encoded = Vec::new();

    encode_ttlv_item(&item, &mut encoded);

    let decoded = crate::codec::decode(&encoded).unwrap();
    assert_eq!(decoded.item_type(), ItemType::TextString);
    assert!(
        decoded.with_value(|value| matches!(value, ValueView::TextString(text) if text == "kmip"))
    );
}

#[test]
fn string_like_kmip30_values_encode_with_their_distinct_item_types() {
    for (value, expected_type, expected_item_type) in [
        (
            Value::identifier("object-id".to_owned()),
            0x0C,
            ItemType::Identifier,
        ),
        (
            Value::reference("object-ref".to_owned()),
            0x0D,
            ItemType::Reference,
        ),
        (
            Value::name_reference("attribute-name".to_owned()),
            0x0E,
            ItemType::NameReference,
        ),
    ] {
        let item = Item::new(
            RawTag::new(0x0042_0173).unwrap().try_checked().unwrap(),
            value,
        )
        .unwrap();
        let mut encoded = Vec::new();
        encode_ttlv_item(&item, &mut encoded);
        assert_eq!(encoded[3], expected_type);
        let decoded = crate::codec::decode(&encoded).unwrap();
        assert_eq!(decoded.item_type(), expected_item_type);
    }
}

#[test]
fn integer_big_integer_and_boolean_values_encode_with_their_ttlv_types() {
    for (value, expected_type, expected_length) in [
        (Value::long_integer(i64::MIN), 0x03, 8),
        (Value::big_integer(vec![0x01, 0x02, 0x03]), 0x04, 3),
        (Value::boolean(true), 0x06, 8),
    ] {
        let item = Item::new(
            RawTag::new(0x0042_0173).unwrap().try_checked().unwrap(),
            value,
        )
        .unwrap();
        let mut encoded = Vec::new();

        encode_ttlv_item(&item, &mut encoded);

        assert_eq!(encoded[3], expected_type);
        assert_eq!(
            u32::from_be_bytes(encoded[4..8].try_into().unwrap()),
            expected_length
        );
    }
}
