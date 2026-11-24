use kmipkit_ttlv::codec::{
    CodecLimits, decode, decode_allow_reserved, decode_with_limits_allow_reserved,
};
use kmipkit_ttlv::{ItemType, ValueView};

// OASIS KMIP Specification v2.1, §§10.1.1–10.1.3 and §11.23; traceability
// Verifies TTLV header structure, value encoding, and exact byte length.
#[test]
fn external_callers_can_decode_a_complete_public_item() {
    let wire = [
        0x42, 0x01, 0x73, 0x02, 0, 0, 0, 4, 0x80, 0, 0, 0, 0, 0, 0, 0,
    ];
    let item = decode(&wire).expect("the external fixture is a valid Integer item");

    assert_eq!(item.tag().raw(), 0x0042_0173);
    assert_eq!(item.item_type(), ItemType::Integer);
    assert!(
        item.with_value(|value| matches!(value, ValueView::Integer(actual) if *actual == i32::MIN))
    );
}

#[test]
fn external_callers_can_decode_reserved_tags_with_allow_reserved() {
    // 0x420091 is Template-Attribute (Reserved in 2.0+)
    let wire = [0x42, 0x00, 0x91, 0x01, 0, 0, 0, 0];
    // Regular decode rejects it
    assert!(decode(&wire).is_err());

    // decode_allow_reserved accepts it
    let item = decode_allow_reserved(&wire).expect("decodes reserved structure item");
    assert_eq!(item.tag().raw(), 0x0042_0091);
    assert_eq!(item.item_type(), ItemType::Structure);

    // decode_with_limits_allow_reserved accepts it with default or explicit limits
    let limits = CodecLimits::defaults();
    let item2 = decode_with_limits_allow_reserved(&wire, &limits).expect("decodes with limits");
    assert_eq!(item2.tag().raw(), 0x0042_0091);

    // Unallocated tag is still rejected when allowing reserved
    let unallocated_wire = [0x00, 0x00, 0x01, 0x01, 0, 0, 0, 0];
    assert!(decode_allow_reserved(&unallocated_wire).is_err());
    assert!(decode_with_limits_allow_reserved(&unallocated_wire, &limits).is_err());
}
