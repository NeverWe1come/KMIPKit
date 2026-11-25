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

#[test]
fn all_legacy_kmip1x_reserved_tags_decode_only_with_allow_reserved() {
    // Tags reserved in KMIP 2.0+ but required by KMIP 1.0-1.4:
    // Template-Attribute (0x420091), Template (0x420090), Common Template-Attribute (0x42001F),
    // Private Key Template-Attribute (0x420065), Public Key Template-Attribute (0x42006E),
    // Operation Policy Name (0x42005D), Certificate Identifier 1.0 (0x420014),
    // Certificate Issuer 1.0 (0x420015), Certificate Subject 1.0 (0x42001A)
    let legacy_reserved_tags: [u32; 9] = [
        0x0042_0091,
        0x0042_0090,
        0x0042_001F,
        0x0042_0065,
        0x0042_006E,
        0x0042_005D,
        0x0042_0014,
        0x0042_0015,
        0x0042_001A,
    ];

    for tag in legacy_reserved_tags {
        let [_, b1, b2, b3] = tag.to_be_bytes();
        // Encode an empty Structure (0x01) of length 0
        let wire = [b1, b2, b3, 0x01, 0, 0, 0, 0];
        assert!(
            decode(&wire).is_err(),
            "strict decode must reject reserved tag 0x{tag:06X}"
        );
        let item = decode_allow_reserved(&wire)
            .unwrap_or_else(|e| panic!("allow_reserved failed for 0x{tag:06X}: {e:?}"));
        assert_eq!(item.tag().raw(), tag);
    }
}
