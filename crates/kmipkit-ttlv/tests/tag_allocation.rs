use kmipkit_ttlv::{RawTag, Tag};

fn raw_tag(raw: u32) -> RawTag {
    RawTag::new(raw).expect("the test value must fit in 24 bits")
}

fn checked_tag(raw: u32) -> Tag {
    let raw_tag = raw_tag(raw);
    RawTag::try_checked(&raw_tag)
        .expect("the test value must be an allocated KMIP tag")
}

#[test]
fn raw_tag_accepts_24_bit_boundaries_and_rejects_wider_values() {
    for raw in [0x000000, 0xFFFFFF] {
        let tag = RawTag::new(raw).expect("the 24-bit boundary must be representable");
        assert_eq!(tag.raw(), raw);
    }

    assert!(RawTag::new(0x1000000).is_err());
}

#[test]
fn exact_project_tags_take_precedence_over_the_aggregate_reserved_range() {
    for raw in [0x420173, 0x420174, 0x420175, 0x420176] {
        let tag = checked_tag(raw);
        assert_eq!(tag.raw(), raw);
    }
}

#[test]
fn individually_and_residually_reserved_tags_are_rejected() {
    for raw in [0x420000, 0x420009, 0x420177, 0x42FFFF] {
        let raw_tag = raw_tag(raw);
        assert!(RawTag::try_checked(&raw_tag).is_err());
    }
}

#[test]
fn extension_range_endpoints_and_middle_value_are_accepted() {
    for raw in [0x540000, 0x542BCD, 0x54FFFF] {
        let tag = checked_tag(raw);
        assert_eq!(tag.raw(), raw);
    }
}

#[test]
fn unused_values_are_rejected_while_remaining_raw_and_failed_checks_borrow() {
    for raw in [0x000000, 0x430000, 0x550000, 0xFFFFFF] {
        let tag = raw_tag(raw);
        assert!(RawTag::try_checked(&tag).is_err());
        assert_eq!(tag.raw(), raw);
    }

    let reserved = raw_tag(0x420009);
    assert!(RawTag::try_checked(&reserved).is_err());
    assert_eq!(reserved.raw(), 0x420009);
}
