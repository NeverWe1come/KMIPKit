use kmipkit::protocol::named_values_generated::values::bitmask::cryptographic_usage_mask;
use kmipkit::protocol::named_values_generated::values::enumeration::cryptographic_algorithm;

#[test]
// OASIS KMIP Specification v2.1 §§11.12 and 12.1; KMIPKIT-0047-FR-001–004.
fn named_values_preserve_unknown_raw_values_and_bits() {
    assert_eq!(cryptographic_algorithm::AES, 3);
    assert_eq!(
        cryptographic_algorithm::name(cryptographic_algorithm::AES),
        Some("AES")
    );
    assert_eq!(cryptographic_algorithm::name(u32::MAX), None);

    let raw = cryptographic_usage_mask::ENCRYPT | 0x8000_0000;
    assert_eq!(
        cryptographic_usage_mask::known_bits(raw),
        cryptographic_usage_mask::ENCRYPT
    );
    assert_eq!(cryptographic_usage_mask::unknown_bits(raw), 0x8000_0000);
    assert_eq!(
        cryptographic_usage_mask::names(raw).collect::<Vec<_>>(),
        ["Encrypt"]
    );
}
