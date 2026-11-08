from kmipkit import protocol_values


# OASIS KMIP Specification v2.1 §§11.12 and 12.1; KMIPKIT-0047-FR-001–004.
def test_named_values_preserve_unknown_raw_values_and_bits() -> None:
    algorithm = protocol_values.CryptographicAlgorithm
    assert algorithm.AES == 3
    assert algorithm.name(algorithm.AES) == "AES"
    assert algorithm.name(0xFFFF_FFFF) is None

    usage = protocol_values.CryptographicUsageMask
    raw = usage.ENCRYPT | 0x8000_0000
    assert usage.known_bits(raw) == usage.ENCRYPT
    assert usage.unknown_bits(raw) == 0x8000_0000
    assert usage.names(raw) == ("Encrypt",)


# KMIP 2.1 §11.36 Table 470; KMIP 3.0 §§11.34–11.37 Tables 581–583.
def test_versioned_named_values_match_exact_edition_tables() -> None:
    assert protocol_values.V2_1_OPERATION_SET_DEFAULTS == 0x36
    assert protocol_values.v2_1_operation_name(protocol_values.V2_1_OPERATION_PING) == "Ping"
    assert protocol_values.V3_0_OPERATION_OBLITERATE == 0x3D
    assert (
        protocol_values.v3_0_operation_name(protocol_values.V3_0_OPERATION_CREATE_CREDENTIAL)
        == "Create Credential"
    )
    assert protocol_values.v3_0_object_class_name(1) == "User"
    assert protocol_values.v3_0_object_class_name(3) is None
    assert protocol_values.v3_0_object_type_name(0x0D) == "Password Credential"
    assert protocol_values.v3_0_object_type_name(6) is None
    assert protocol_values.v3_0_operation_name(0x43) is None
