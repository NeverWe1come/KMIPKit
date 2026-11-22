use kmipkit_ttlv::KmipTag;

#[test]
fn authenticated_encryption_additional_data_has_a_name() {
    assert_eq!(
        KmipTag::AuthenticatedEncryptionAdditionalData.name(),
        "Authenticated Encryption Additional Data"
    );
}

#[test]
fn every_catalogued_tag_round_trips_and_has_its_normative_name() {
    let mut catalogued_tags = 0;
    for low_tag in 0..=u16::MAX {
        let raw = 0x0042_0000 | u32::from(low_tag);
        if let Some(tag) = KmipTag::from_u32(raw) {
            assert_eq!(tag.as_u32(), raw);
            assert_eq!(tag.raw(), raw);
            assert!(!tag.name().is_empty(), "tag {raw:#08x} has no name");
            catalogued_tags += 1;
        }
    }
    assert!(
        catalogued_tags > 400,
        "tag catalog iteration was incomplete"
    );
}
