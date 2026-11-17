"""Cross-version assigned KMIP tag name contracts."""

from kmipkit import protocol_values


def test_assigned_tags_are_version_scoped_and_unknown_values_stay_unassigned() -> None:
    assert protocol_values.Tag.OBJECT_GROUPS == 0x00420166
    assert protocol_values.Tag.name(protocol_values.Tag.OBJECT_GROUPS) == "Object Groups"
    assert protocol_values.Tag.name(0x004201FF) is None

    assert protocol_values.V3_0_TAG_CERTIFICATE_LINK == 0x00420190
    assert protocol_values.v3_0_tag_name(0x00420190) == "Certificate Link"
    assert protocol_values.v3_0_tag_name(0x0042018F) is None
