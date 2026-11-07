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
