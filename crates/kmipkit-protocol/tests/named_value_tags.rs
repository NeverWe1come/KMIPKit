use kmipkit_protocol::named_values_generated::values::{
    enumeration::tag,
    versions::v3_0::enumeration::tag as v3_tag,
};

#[test]
fn assigned_tag_lookups_are_version_scoped_and_preserve_unknown_values() {
    assert_eq!(tag::OBJECT_GROUPS, 0x0042_0166);
    assert_eq!(tag::name(tag::OBJECT_GROUPS), Some("Object Groups"));
    assert_eq!(tag::name(0x0042_01ff), None);

    assert_eq!(v3_tag::CERTIFICATE_LINK, 0x0042_0190);
    assert_eq!(v3_tag::name(v3_tag::CERTIFICATE_LINK), Some("Certificate Link"));
    assert_eq!(v3_tag::name(0x0042_018f), None);
}
