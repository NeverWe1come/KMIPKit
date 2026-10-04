use kmipkit_ttlv::{RawTag, Tag};

fn raw_tag(raw: u32) -> RawTag {
    RawTag::new(raw).expect("the test value must fit in 24 bits")
}

fn checked_tag(raw: u32) -> Tag {
    let raw_tag = raw_tag(raw);
    RawTag::try_checked(&raw_tag).expect("the test value must be an allocated KMIP tag")
}

#[test]
fn raw_tag_accepts_24_bit_boundaries_and_rejects_wider_values() {
    for raw in [0x0000_0000, 0x00FF_FFFF] {
        let tag = RawTag::new(raw).expect("the 24-bit boundary must be representable");
        assert_eq!(tag.raw(), raw);
    }

    assert!(RawTag::new(0x0100_0000).is_err());
}

#[test]
fn exact_project_tags_take_precedence_over_the_aggregate_reserved_range() {
    for raw in [0x0042_0173, 0x0042_0174, 0x0042_0175, 0x0042_0176] {
        let tag = checked_tag(raw);
        assert_eq!(tag.raw(), raw);
    }
}

#[test]
fn individually_and_residually_reserved_tags_are_rejected() {
    for raw in [0x0042_0000, 0x0042_0009, 0x0042_0177, 0x0042_FFFF] {
        let raw_tag = raw_tag(raw);
        assert!(RawTag::try_checked(&raw_tag).is_err());
    }
}

#[test]
fn extension_range_endpoints_and_middle_value_are_accepted() {
    for raw in [0x0054_0000, 0x0054_2BCD, 0x0054_FFFF] {
        let tag = checked_tag(raw);
        assert_eq!(tag.raw(), raw);
    }
}

#[test]
fn unused_values_are_rejected_while_remaining_raw_and_failed_checks_borrow() {
    for raw in [0x0000_0000, 0x0043_0000, 0x0055_0000, 0x00FF_FFFF] {
        let tag = raw_tag(raw);
        assert!(RawTag::try_checked(&tag).is_err());
        assert_eq!(tag.raw(), raw);
    }

    let reserved = raw_tag(0x0042_0009);
    assert!(RawTag::try_checked(&reserved).is_err());
    assert_eq!(reserved.raw(), 0x0042_0009);
}
