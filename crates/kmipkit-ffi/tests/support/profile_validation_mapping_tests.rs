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

#[test]
fn profile_validation_details_remain_owned_after_last_error_changes() {
    let validation_error = ProfileValidationSelection::new(["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        .validate()
        .expect_err("incomplete Baseline evidence fails closed");
    let error = ClientError::profile_validation(validation_error, RequestDeliveryState::NotSent);
    assert_eq!(record_client_error(&error), ERROR_INVALID_INPUT);

    let mut handle = ptr::null_mut();
    assert_eq!(
        kmipkit_last_error_profile_validation_error_get_v1(&mut handle),
        SUCCESS
    );
    assert!(!handle.is_null());
    assert_eq!(record_ffi_error(ERROR_INVALID_INPUT), ERROR_INVALID_INPUT);

    let mut view = kmipkit_profile_validation_error_view_v1_t {
        struct_size: u32::try_from(size_of::<kmipkit_profile_validation_error_view_v1_t>())
            .expect("profile error view size fits u32"),
        abi_version: 1,
        source_kind: 0,
        batch_item_index_present: 0,
        batch_item_index: 0,
        error_code_data: ptr::null(),
        error_code_length: 0,
        profile_id_data: ptr::null(),
        profile_id_length: 0,
        requirement_id_data: ptr::null(),
        requirement_id_length: 0,
        source_id_data: ptr::null(),
        source_id_length: 0,
        source_locator_data: ptr::null(),
        source_locator_length: 0,
    };
    assert_eq!(
        kmipkit_profile_validation_error_view_get_v1(handle, &mut view),
        SUCCESS
    );
    assert_eq!(view.source_kind, PROFILE_ERROR_SOURCE_CLAUSE);
    assert_eq!(view.batch_item_index_present, 0);
    assert_eq!(
        span_text(view.error_code_data, view.error_code_length),
        "profile_unavailable"
    );
    assert_eq!(
        span_text(view.profile_id_data, view.profile_id_length),
        "KMIPKIT-PROFILE-BASELINE-CLIENT"
    );
    assert_eq!(
        span_text(view.requirement_id_data, view.requirement_id_length),
        "KMIPKIT-REQ-SPEC-14.1-001"
    );
    assert_eq!(
        span_text(view.source_id_data, view.source_id_length),
        "KMIPKIT-CLAUSE-SPEC-14.1-001"
    );
    assert_eq!(view.source_locator_length, 0);
    kmipkit_profile_validation_error_release(handle);
}

#[test]
fn absent_profile_error_snapshot_does_not_clobber_client_error_metadata() {
    let error = ClientError::Transport(kmipkit_transport::TransportError::new(
        RequestDeliveryState::PossiblySent,
        TransportCauseCategory::Timeout,
        std::io::Error::other("redacted timeout source"),
    ));
    assert_eq!(record_client_error(&error), ERROR_TRANSPORT);

    let mut handle = ptr::null_mut();
    let detail_status = kmipkit_last_error_profile_validation_error_get_v1(&mut handle);
    let mut status = SUCCESS;
    let mut category = ERROR_CATEGORY_NONE;
    let mut cause = ERROR_CAUSE_NONE;
    let mut delivery = DELIVERY_STATE_NONE;
    assert_eq!(
        kmipkit_last_error_get(&mut status, &mut category, &mut cause, &mut delivery),
        SUCCESS
    );

    assert!(handle.is_null());
    assert_eq!(
        (
            detail_status,
            status,
            category,
            cause,
            delivery,
        ),
        (
            SUCCESS,
            ERROR_TRANSPORT,
            ERROR_CATEGORY_TRANSPORT,
            ERROR_CAUSE_TRANSPORT_TIMEOUT,
            DELIVERY_STATE_POSSIBLY_SENT,
        )
    );
}

fn span_text(data: *const u8, length: u64) -> String {
    assert!(!data.is_null());
    let Ok(length) = usize::try_from(length) else {
        panic!("profile error text length fits usize");
    };
    // SAFETY: the FFI view points into the live profile-error handle and this test keeps that
    // handle open until after the span has been copied.
    let bytes = unsafe { std::slice::from_raw_parts(data, length) };
    String::from_utf8(bytes.to_vec()).expect("profile error IDs are UTF-8")
}
