use super::*;

#[test]
fn c_selection_copies_flat_profile_id_spans_in_caller_order() {
    let bytes = b"baseline_clientconditional_https";
    let offsets = [0_u64, 15_u64];
    let lengths = [15_u64, 17_u64];
    let selection = kmipkit_profile_validation_selection_v1_t {
        struct_size: u32::try_from(size_of::<kmipkit_profile_validation_selection_v1_t>())
            .expect("C ABI selection size fits u32"),
        abi_version: 1,
        profile_id_data: bytes.as_ptr(),
        profile_id_data_length: bytes.len() as u64,
        profile_id_offsets: offsets.as_ptr(),
        profile_id_lengths: lengths.as_ptr(),
        profile_id_count: 2,
    };

    let parsed = profile_selection_from_v1(&raw const selection).expect("valid C spans parse");

    assert_eq!(
        parsed.profile_ids(),
        ["baseline_client", "conditional_https"]
    );
}

#[test]
fn c_context_preserves_tri_state_and_exact_caller_state_for_one_exchange() {
    let identifier = b"caller-object-id";
    let entry = kmipkit_profile_object_state_v1_t {
        struct_size: u32::try_from(size_of::<kmipkit_profile_object_state_v1_t>())
            .expect("C ABI object state size fits u32"),
        abi_version: 1,
        batch_item_index: 3,
        unique_identifier_data: identifier.as_ptr(),
        unique_identifier_length: identifier.len() as u64,
        known_fields: PROFILE_STATE_ORIGINAL_CREATION_DATE_PRESENT
            | PROFILE_STATE_USAGE_LIMITS_PRESENT
            | PROFILE_STATE_USAGE_LIMITS_TOTAL
            | PROFILE_STATE_USAGE_LIMITS_COUNT,
        boolean_values: PROFILE_STATE_USAGE_LIMITS_PRESENT,
        usage_limits_total: 100,
        usage_limits_count: 7,
    };
    let entries = [entry];
    let context = kmipkit_profile_validation_context_v1_t {
        struct_size: u32::try_from(size_of::<kmipkit_profile_validation_context_v1_t>())
            .expect("C ABI context size fits u32"),
        abi_version: 1,
        object_states: entries.as_ptr(),
        object_state_count: 1,
    };

    let parsed = profile_context_from_v1(&raw const context).expect("valid C context parses");
    let expected = ProfileValidationContext::new().with_object_state(
        3,
        ProfileObjectState::new("caller-object-id")
            .with_original_creation_date_present(Some(false))
            .with_usage_limits_present(Some(true))
            .with_usage_limits_total(Some(100))
            .with_usage_limits_count(Some(7)),
    );

    assert_eq!(parsed, expected);
}

#[test]
fn c_selection_rejects_offsets_outside_the_flat_buffer() {
    let bytes = b"baseline_client";
    let offsets = [0_u64];
    let lengths = [bytes.len() as u64 + 1];
    let selection = kmipkit_profile_validation_selection_v1_t {
        struct_size: u32::try_from(size_of::<kmipkit_profile_validation_selection_v1_t>())
            .expect("C ABI selection size fits u32"),
        abi_version: 1,
        profile_id_data: bytes.as_ptr(),
        profile_id_data_length: bytes.len() as u64,
        profile_id_offsets: offsets.as_ptr(),
        profile_id_lengths: lengths.as_ptr(),
        profile_id_count: 1,
    };

    assert_eq!(
        profile_selection_from_v1(&raw const selection),
        Err(ERROR_INVALID_INPUT)
    );
}
