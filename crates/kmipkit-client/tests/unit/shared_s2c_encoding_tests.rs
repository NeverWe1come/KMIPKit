//! Regression vectors for the shared S2C encoder and native callback wrappers.

use kmipkit_protocol::server_to_client::{PutFunction, ServerNotifyRequest, ServerPutRequest};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

use crate::{binding_support, s2c_encoding};

fn item(value: Value) -> Item {
    let tag = RawTag::new(0x0054_1234).unwrap().try_checked().unwrap();
    Item::new(tag, value).unwrap()
}

fn put(value: Value, attributes: Option<Structure>) -> ServerPutRequest {
    ServerPutRequest::new(
        PutFunction::NEW,
        "object".into(),
        None,
        item(value),
        attributes,
    )
}

#[test]
fn response_and_binding_encoders_match_fixed_vectors_for_every_item_type() {
    let mut structure = Structure::new();
    structure.try_push(item(Value::integer(-1))).unwrap();
    // Values include sign bits, unknown enums, non-ASCII text, KMIP 3.0 types,
    // and a nested Structure. The expected bytes are independent wire vectors.
    let cases = [
        (
            Value::structure(structure),
            1,
            vec![
                0x54, 0x12, 0x34, 2, 0, 0, 0, 4, 255, 255, 255, 255, 0, 0, 0, 0,
            ],
        ),
        (Value::integer(-1), 2, vec![255; 4]),
        (
            Value::long_integer(i64::MIN),
            3,
            vec![128, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            Value::big_integer(vec![0, 0, 0, 0, 0, 0, 0, 1]),
            4,
            vec![0, 0, 0, 0, 0, 0, 0, 1],
        ),
        (Value::enumeration(0x8123_4567), 5, vec![129, 35, 69, 103]),
        (Value::boolean(true), 6, vec![0, 0, 0, 0, 0, 0, 0, 1]),
        (Value::text_string("é".into()), 7, vec![195, 169]),
        (Value::byte_string(vec![0, 255, 1]), 8, vec![0, 255, 1]),
        (Value::date_time(-1), 9, vec![255; 8]),
        (Value::interval(u32::MAX), 10, vec![255; 4]),
        (
            Value::date_time_extended(i64::MIN),
            11,
            vec![128, 0, 0, 0, 0, 0, 0, 0],
        ),
        (Value::identifier("id".into()), 12, b"id".to_vec()),
        (Value::reference("ref".into()), 13, b"ref".to_vec()),
        (Value::name_reference("name".into()), 14, b"name".to_vec()),
    ];
    for (value, kind, payload) in cases {
        let request = put(value, None);
        let mut expected = vec![0x54, 0x12, 0x34, kind];
        expected.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_be_bytes());
        expected.extend_from_slice(&payload);
        expected.resize(expected.len().next_multiple_of(8), 0);
        assert_eq!(
            binding_support::put_object_ttlv(&request).as_slice(),
            expected
        );

        let mut response = Structure::new();
        response
            .try_push(request.object_item().try_clone().unwrap())
            .unwrap();
        let mut bytes = Vec::new();
        s2c_encoding::encode_structure_view(0x0042_007B, &response.view(), &mut bytes);
        assert_eq!(&bytes[8..], expected);
        let decoded = kmipkit_ttlv::codec::decode(&bytes).unwrap();
        assert_eq!(decoded.as_structure().unwrap().children().len(), 1);
    }
}

#[test]
fn callback_attributes_preserve_absence_empty_presence_and_padding_boundaries() {
    let absent_put = put(Value::integer(1), None);
    let absent_notify = ServerNotifyRequest::new("object".into(), None, vec![]);
    assert!(binding_support::put_attributes_ttlv(&absent_put).is_none());
    assert!(binding_support::notify_attributes_ttlv(&absent_notify).is_none());

    let empty_put = put(Value::integer(1), Some(Structure::new()));
    assert_eq!(
        binding_support::put_attributes_ttlv(&empty_put)
            .unwrap()
            .as_slice(),
        [0x42, 0x01, 0x25, 1, 0, 0, 0, 0],
    );
    for length in 0..=17 {
        let mut attributes = Structure::new();
        // A repeated unknown tag must retain order and exact payload bytes.
        attributes
            .try_push(item(Value::byte_string(vec![0xAA; length])))
            .unwrap();
        attributes.try_push(item(Value::boolean(false))).unwrap();
        let notify = ServerNotifyRequest::new("object".into(), Some(attributes), vec![]);
        let put = put(
            Value::integer(1),
            Some(notify.attributes().unwrap().view().try_clone().unwrap()),
        );
        let bytes = binding_support::notify_attributes_ttlv(&notify).unwrap();
        assert_eq!(bytes, binding_support::put_attributes_ttlv(&put).unwrap());
        let decoded = kmipkit_ttlv::codec::decode(&bytes).unwrap();
        let view = decoded.as_structure().unwrap();
        assert_eq!(view.children().len(), 2);
        view.children()[0].with_value(|value| match value {
            kmipkit_ttlv::ValueView::ByteString(bytes) => assert_eq!(bytes, vec![0xAA; length]),
            _ => panic!("expected ByteString"),
        });
    }
}

#[test]
fn deleted_attribute_callback_keeps_wrapper_order_and_empty_case() {
    let empty = ServerNotifyRequest::new("object".into(), None, vec![]);
    assert_eq!(
        binding_support::notify_deleted_attributes_ttlv(&empty).as_slice(),
        [0x42, 0, 0x79, 1, 0, 0, 0, 0],
    );
    let references = [1, 2].map(|value| {
        let mut reference = Structure::new();
        reference.try_push(item(Value::integer(value))).unwrap();
        reference
    });
    let notify = ServerNotifyRequest::new("object".into(), None, Vec::from(references));
    let bytes = binding_support::notify_deleted_attributes_ttlv(&notify);
    assert_eq!(&bytes[..8], [0x42, 0, 0x79, 1, 0, 0, 0, 48]);
    for (index, offset) in [8, 32].into_iter().enumerate() {
        assert_eq!(&bytes[offset..offset + 8], [0x42, 1, 0x3B, 1, 0, 0, 0, 16]);
        assert_eq!(bytes[offset + 19], u8::try_from(index + 1).unwrap());
    }
    kmipkit_ttlv::codec::decode(&bytes).unwrap();
}
