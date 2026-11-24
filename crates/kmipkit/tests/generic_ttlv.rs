use kmipkit::generic_ttlv::{GenericTtlvEncodingError, with_encoded_generic_ttlv};
use kmipkit::ttlv::{Item, RawTag, Value, codec::CodecLimits};

#[test]
fn encodes_native_generic_item_with_ttlv_padding() {
    let tag = RawTag::new(0x0042_00c2).unwrap().try_checked().unwrap();
    let item = Item::new(tag, Value::byte_string(b"abc".to_vec())).unwrap();
    let expected = [
        0x42, 0x00, 0xc2, 0x08, 0, 0, 0, 3, b'a', b'b', b'c', 0, 0, 0, 0, 0,
    ];

    with_encoded_generic_ttlv(&item, &CodecLimits::defaults(), |bytes| {
        assert_eq!(bytes, expected);
    })
    .unwrap();

    let limits = CodecLimits::new(expected.len() - 1, 64, 1).unwrap();
    let result = with_encoded_generic_ttlv(&item, &limits, |_| {
        panic!("the callback must only receive a complete encoded item");
    });
    assert_eq!(result, Err(GenericTtlvEncodingError));
}
