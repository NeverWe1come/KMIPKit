use super::*;

#[link(name = "kmipkit_credential_type_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_credential_type_c_consumer_round_trip(raw: u32) -> u32;
}

#[test]
fn compiled_c_consumer_preserves_known_extension_and_unknown_raw_values() {
    for raw in [1, 2, 3, 4, 5, 6, 0x8000_0009, 7, u32::MAX] {
        // SAFETY: The C consumer accepts a fixed-width scalar and returns that exact raw value.
        let actual = unsafe { kmipkit_credential_type_c_consumer_round_trip(raw) };
        assert_eq!(actual, kmipkit_protocol::CredentialType::from_raw(raw).raw());
    }
}
