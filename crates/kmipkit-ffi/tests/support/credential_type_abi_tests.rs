use super::*;

#[link(name = "kmipkit_credential_type_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_credential_type_c_consumer_round_trip(raw: u32, out_raw: *mut u32) -> i32;
}

#[test]
fn compiled_c_consumer_preserves_known_extension_and_unknown_raw_values() {
    for raw in [1, 2, 3, 4, 5, 6, 0x8000_0009, 7, u32::MAX] {
        let mut actual = u32::MAX;
        // SAFETY: `actual` is writable storage for the C consumer's fixed-width output parameter.
        let status = unsafe { kmipkit_credential_type_c_consumer_round_trip(raw, &raw mut actual) };
        assert_eq!(status, 0);
        assert_eq!(
            actual,
            kmipkit_protocol::CredentialType::from_raw(raw).raw()
        );
    }
}

#[test]
fn credential_type_abi_rejects_a_null_output_pointer() {
    assert_eq!(
        kmipkit_credential_type_from_raw(1, std::ptr::null_mut()),
        ERROR_INVALID_INPUT
    );
}
