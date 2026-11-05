use super::*;
use crate::kmipkit_byte_span_v1_t;
use kmipkit_protocol::ProfileValidationErrorCode;

#[cfg(feature = "coverage-c-consumer")]
#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_profile_validation_result_exercise(
        result: *mut kmipkit_profile_validation_result_t,
        request: *mut kmipkit_ttlv_structure_t,
    ) -> i32;
}

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
fn c_profile_selection_resolution_fails_closed_with_redacted_profile_details() {
    const BASELINE: &[u8] = b"KMIPKIT-PROFILE-BASELINE-CLIENT";
    let offsets = [0_u64];
    let lengths = [BASELINE.len() as u64];
    let selection = kmipkit_profile_validation_selection_v1_t {
        struct_size: u32::try_from(size_of::<kmipkit_profile_validation_selection_v1_t>())
            .expect("C ABI selection size fits u32"),
        abi_version: 1,
        profile_id_data: BASELINE.as_ptr(),
        profile_id_data_length: BASELINE.len() as u64,
        profile_id_offsets: offsets.as_ptr(),
        profile_id_lengths: lengths.as_ptr(),
        profile_id_count: 1,
    };
    let mut result = ptr::dangling_mut::<kmipkit_profile_validation_result_t>();

    assert_eq!(
        unsafe {
            kmipkit_profile_validation_selection_validate_v1(&raw const selection, &raw mut result)
        },
        ERROR_INVALID_INPUT
    );
    assert!(
        result.is_null(),
        "a failed resolution must clear its output"
    );

    let (error, view) = last_profile_validation_view();
    assert_eq!(
        span_text(view.error_code_data, view.error_code_length),
        "profile_unavailable"
    );
    assert_eq!(
        span_text(view.profile_id_data, view.profile_id_length),
        "KMIPKIT-PROFILE-BASELINE-CLIENT"
    );
    assert_eq!(view.source_kind, PROFILE_ERROR_SOURCE_CLAUSE);
    assert_eq!(
        span_text(view.requirement_id_data, view.requirement_id_length),
        "KMIPKIT-REQ-SPEC-14.1-001"
    );
    kmipkit_profile_validation_error_release(error);
}

#[test]
fn c_profile_result_accessors_reject_null_handles_and_clear_scalar_outputs() {
    let mut profile_count = u64::MAX;
    assert_eq!(
        unsafe {
            kmipkit_profile_validation_result_profile_id_count_v1(
                ptr::null_mut(),
                &raw mut profile_count,
            )
        },
        ERROR_INVALID_INPUT
    );
    assert_eq!(profile_count, 0);

    let mut requirement_count = u64::MAX;
    assert_eq!(
        unsafe {
            kmipkit_profile_validation_result_requirement_id_count_v1(
                ptr::null_mut(),
                &raw mut requirement_count,
            )
        },
        ERROR_INVALID_INPUT
    );
    assert_eq!(requirement_count, 0);
}

#[test]
fn c_profile_result_accessors_preserve_ordered_dependency_closed_ids() {
    let resolved = ProfileValidationSelection::for_test(["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        .validate()
        .expect("the test-only selector resolves the catalogued baseline rules");
    let expected_profiles = resolved.profile_ids().to_vec();
    let expected_requirements = resolved.requirement_ids().to_vec();
    let handle = try_make_handle(
        Kind::ProfileValidationResult,
        HandleValue::ProfileValidationResult(resolved),
    )
    .expect("a resolved result receives an owned C handle");

    let mut profile_count = 0;
    assert_eq!(
        unsafe {
            kmipkit_profile_validation_result_profile_id_count_v1(handle, &raw mut profile_count)
        },
        SUCCESS
    );
    assert_eq!(
        profile_count,
        u64::try_from(expected_profiles.len()).expect("profile count fits the C ABI")
    );
    for (index, expected) in expected_profiles.iter().enumerate() {
        let mut span = kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: 0,
        };
        assert_eq!(
            unsafe {
                kmipkit_profile_validation_result_profile_id_at_v1(
                    handle,
                    index as u64,
                    &raw mut span,
                )
            },
            SUCCESS
        );
        assert_eq!(span_text(span.data, span.length), *expected);
    }

    let mut requirement_count = 0;
    assert_eq!(
        unsafe {
            kmipkit_profile_validation_result_requirement_id_count_v1(
                handle,
                &raw mut requirement_count,
            )
        },
        SUCCESS
    );
    assert_eq!(
        requirement_count,
        u64::try_from(expected_requirements.len()).expect("requirement count fits the C ABI")
    );
    for (index, expected) in expected_requirements.iter().enumerate() {
        let mut span = kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: 0,
        };
        assert_eq!(
            unsafe {
                kmipkit_profile_validation_result_requirement_id_at_v1(
                    handle,
                    index as u64,
                    &raw mut span,
                )
            },
            SUCCESS
        );
        assert_eq!(span_text(span.data, span.length), *expected);
    }
    kmipkit_profile_validation_result_release(handle);
}

#[test]
fn c_profile_result_release_invalidates_result_access_and_borrowed_id_spans() {
    let resolved = ProfileValidationSelection::for_test(["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        .validate()
        .expect("the test-only selector resolves the catalogued baseline rules");
    let handle = make_handle(
        Kind::ProfileValidationResult,
        HandleValue::ProfileValidationResult(resolved),
    );
    let mut span = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        unsafe { kmipkit_profile_validation_result_profile_id_at_v1(handle, 0, &raw mut span) },
        SUCCESS
    );
    assert!(!span.data.is_null());
    kmipkit_profile_validation_result_release(handle);

    let mut count = u64::MAX;
    assert_eq!(
        unsafe { kmipkit_profile_validation_result_profile_id_count_v1(handle, &raw mut count) },
        ERROR_INVALID_INPUT
    );
    assert_eq!(count, 0);
    // The borrowed span must not be read after release; only its owner handle is checked here.
}

#[test]
fn c_profile_result_validators_reject_null_stale_and_invalid_transport_inputs() {
    let mut result = ptr::null_mut::<kmipkit_profile_validation_result_t>();
    let mut count = u64::MAX;
    assert_eq!(
        unsafe { kmipkit_profile_validation_result_validate_transport_v1(result, 1) },
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        unsafe { kmipkit_profile_validation_result_validate_request_v1(result, ptr::null_mut()) },
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        unsafe {
            kmipkit_profile_validation_result_validate_request_with_context_v1(
                result,
                ptr::null_mut(),
                ptr::null(),
            )
        },
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        unsafe {
            kmipkit_profile_validation_result_validate_response_v1(
                result,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        },
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        unsafe {
            kmipkit_profile_validation_result_validate_response_with_context_v1(
                result,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            )
        },
        ERROR_INVALID_INPUT
    );

    let resolved = ProfileValidationSelection::for_test(["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        .validate()
        .expect("the test-only selector resolves the catalogued baseline rules");
    result = make_handle(
        Kind::ProfileValidationResult,
        HandleValue::ProfileValidationResult(resolved),
    );
    assert_eq!(
        unsafe { kmipkit_profile_validation_result_validate_transport_v1(result, 99) },
        ERROR_INVALID_INPUT,
        "transport discriminants outside the versioned ABI are rejected"
    );
    let mut error = ptr::null_mut();
    assert_eq!(
        kmipkit_last_error_profile_validation_error_get_v1(&raw mut error),
        SUCCESS,
        "malformed ABI inputs do not create a profile-rule detail snapshot"
    );
    assert!(error.is_null());
    kmipkit_profile_validation_result_release(result);

    assert_eq!(
        unsafe { kmipkit_profile_validation_result_validate_transport_v1(result, 1) },
        ERROR_INVALID_INPUT,
        "a released result handle is stale"
    );
    assert_eq!(
        unsafe { kmipkit_profile_validation_result_profile_id_count_v1(result, &raw mut count) },
        ERROR_INVALID_INPUT
    );
    assert_eq!(count, 0);
}

#[test]
fn c_profile_result_validators_match_the_canonical_rust_result_methods() {
    let resolved = ProfileValidationSelection::for_test(["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        .validate()
        .expect("the test-only selector resolves the catalogued baseline rules");
    let result = make_handle(
        Kind::ProfileValidationResult,
        HandleValue::ProfileValidationResult(resolved.clone()),
    );
    let request = profile_request([(0x0000_001e, Structure::new())]);
    let request_tree = request
        .with_ttlv(|tree| clone_structure_view(&tree))
        .expect("the request tree can be cloned for a borrowed ABI input");
    let request_handle = make_handle(Kind::Structure, HandleValue::Structure(request_tree));
    let response = profile_response(0x0000_001e);
    let response_tree = response
        .with_ttlv(|tree| clone_structure_view(&tree))
        .expect("the response tree can be cloned for a borrowed ABI input");
    let response_handle = make_handle(Kind::Structure, HandleValue::Structure(response_tree));

    let expected = resolved.validate_transport(kmipkit_protocol::ProfileTransport::Https);
    assert_profile_validation_outcome(
        unsafe { kmipkit_profile_validation_result_validate_transport_v1(result, 2) },
        expected,
    );

    let expected = resolved.validate_request(&request);
    assert_profile_validation_outcome(
        unsafe { kmipkit_profile_validation_result_validate_request_v1(result, request_handle) },
        expected,
    );

    let context = ProfileValidationContext::new();
    let expected = resolved.validate_request_with_context(&request, &context);
    assert_profile_validation_outcome(
        unsafe {
            kmipkit_profile_validation_result_validate_request_with_context_v1(
                result,
                request_handle,
                ptr::null(),
            )
        },
        expected,
    );

    let expected = resolved.validate_response(&request, &response);
    assert_profile_validation_outcome(
        unsafe {
            kmipkit_profile_validation_result_validate_response_v1(
                result,
                request_handle,
                response_handle,
            )
        },
        expected,
    );

    let expected = resolved.validate_response_with_context(&request, &response, &context);
    assert_profile_validation_outcome(
        unsafe {
            kmipkit_profile_validation_result_validate_response_with_context_v1(
                result,
                request_handle,
                response_handle,
                ptr::null(),
            )
        },
        expected,
    );

    kmipkit_profile_validation_result_release(result);
    kmipkit_ttlv_structure_release(request_handle);
    kmipkit_ttlv_structure_release(response_handle);
}

#[test]
fn c_profile_validators_reject_bad_context_size_and_version_without_profile_details() {
    let resolved = ProfileValidationSelection::for_test(["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        .validate()
        .expect("the test-only selector resolves the catalogued baseline rules");
    let result = make_handle(
        Kind::ProfileValidationResult,
        HandleValue::ProfileValidationResult(resolved),
    );
    let request = profile_request([(0x0000_001e, Structure::new())]);
    let request_tree = request
        .with_ttlv(|tree| clone_structure_view(&tree))
        .expect("the request tree can be cloned for a borrowed ABI input");
    let request_handle = make_handle(Kind::Structure, HandleValue::Structure(request_tree));
    let invalid_context = kmipkit_profile_validation_context_v1_t {
        struct_size: 0,
        abi_version: 99,
        object_states: ptr::null(),
        object_state_count: 0,
    };

    assert_eq!(
        unsafe {
            kmipkit_profile_validation_result_validate_request_with_context_v1(
                result,
                request_handle,
                &raw const invalid_context,
            )
        },
        ERROR_INVALID_INPUT
    );
    assert!(last_profile_validation_error_is_absent());

    let invalid_context_version = kmipkit_profile_validation_context_v1_t {
        struct_size: u32::try_from(size_of::<kmipkit_profile_validation_context_v1_t>())
            .expect("C profile context size fits u32"),
        abi_version: 99,
        object_states: ptr::null(),
        object_state_count: 0,
    };
    assert_eq!(
        unsafe {
            kmipkit_profile_validation_result_validate_request_with_context_v1(
                result,
                request_handle,
                &raw const invalid_context_version,
            )
        },
        ERROR_INVALID_INPUT
    );
    assert!(last_profile_validation_error_is_absent());
    kmipkit_profile_validation_result_release(result);
    kmipkit_ttlv_structure_release(request_handle);
}

#[test]
fn c_profile_result_message_validators_reject_malformed_ttlv_without_rule_details() {
    let resolved = ProfileValidationSelection::for_test(["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        .validate()
        .expect("the test-only selector resolves the catalogued baseline rules");
    let result = make_handle(
        Kind::ProfileValidationResult,
        HandleValue::ProfileValidationResult(resolved),
    );
    let malformed_request = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let malformed_response = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));

    assert_eq!(
        unsafe { kmipkit_profile_validation_result_validate_request_v1(result, malformed_request) },
        ERROR_PROTOCOL
    );
    assert!(last_profile_validation_error_is_absent());
    let mut status = SUCCESS;
    let mut category = ERROR_CATEGORY_NONE;
    let mut cause = ERROR_CAUSE_NONE;
    let mut delivery = DELIVERY_STATE_NONE;
    assert_eq!(
        kmipkit_last_error_get(
            &raw mut status,
            &raw mut category,
            &raw mut cause,
            &raw mut delivery,
        ),
        SUCCESS
    );
    assert_eq!(
        (status, category, cause, delivery),
        (
            ERROR_PROTOCOL,
            ERROR_CATEGORY_PROTOCOL,
            ERROR_CAUSE_PROTOCOL_OTHER,
            DELIVERY_STATE_NONE,
        )
    );
    let valid_request = profile_request([(0x0000_001e, Structure::new())]);
    let valid_request_tree = valid_request
        .with_ttlv(|tree| clone_structure_view(&tree))
        .expect("the request tree can be cloned for a borrowed ABI input");
    let valid_request_handle =
        make_handle(Kind::Structure, HandleValue::Structure(valid_request_tree));
    assert_eq!(
        unsafe {
            kmipkit_profile_validation_result_validate_response_v1(
                result,
                valid_request_handle,
                malformed_response,
            )
        },
        ERROR_PROTOCOL
    );
    assert!(last_profile_validation_error_is_absent());
    status = SUCCESS;
    category = ERROR_CATEGORY_NONE;
    cause = ERROR_CAUSE_NONE;
    delivery = DELIVERY_STATE_NONE;
    assert_eq!(
        kmipkit_last_error_get(
            &raw mut status,
            &raw mut category,
            &raw mut cause,
            &raw mut delivery,
        ),
        SUCCESS
    );
    assert_eq!(
        (status, category, cause, delivery),
        (
            ERROR_PROTOCOL,
            ERROR_CATEGORY_PROTOCOL,
            ERROR_CAUSE_PROTOCOL_OTHER,
            DELIVERY_STATE_NONE,
        )
    );
    kmipkit_profile_validation_result_release(result);
    kmipkit_ttlv_structure_release(malformed_request);
    kmipkit_ttlv_structure_release(malformed_response);
    kmipkit_ttlv_structure_release(valid_request_handle);
}

#[test]
fn c_profile_context_validation_redacts_request_identifiers_and_context_values() {
    const REQUEST_IDENTIFIER: &str = "request-id-should-never-be-diagnostic";
    const CONTEXT_IDENTIFIER: &str = "context-id-should-never-be-diagnostic";
    const RESPONSE_MARKER: &str = "raw-response-must-never-be-diagnostic";
    let request = profile_request([
        (0x0000_001e, Structure::new()),
        (
            0x0000_0031,
            profile_structure([
                profile_item(
                    0x0042_0094,
                    Value::text_string(REQUEST_IDENTIFIER.to_owned()),
                ),
                profile_item(
                    0x0042_013d,
                    Value::structure(profile_structure([profile_item(
                        0x0042_00bc,
                        Value::date_time(1_700_000_001),
                    )])),
                ),
            ]),
        ),
    ]);
    let request_tree = request
        .with_ttlv(|tree| clone_structure_view(&tree))
        .expect("the existing request tree can be cloned for a borrowed ABI input");
    let request_handle = make_handle(Kind::Structure, HandleValue::Structure(request_tree));
    let resolved = ProfileValidationSelection::for_test(["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        .validate()
        .expect("the test-only selector resolves the catalogued baseline rules");
    let result = make_handle(
        Kind::ProfileValidationResult,
        HandleValue::ProfileValidationResult(resolved),
    );
    let context_identifier = CONTEXT_IDENTIFIER.as_bytes();
    let object_state = kmipkit_profile_object_state_v1_t {
        struct_size: u32::try_from(size_of::<kmipkit_profile_object_state_v1_t>())
            .expect("C object state size fits u32"),
        abi_version: 1,
        batch_item_index: 1,
        unique_identifier_data: context_identifier.as_ptr(),
        unique_identifier_length: context_identifier.len() as u64,
        known_fields: PROFILE_STATE_ORIGINAL_CREATION_DATE_PRESENT,
        boolean_values: PROFILE_STATE_ORIGINAL_CREATION_DATE_PRESENT,
        usage_limits_total: 0,
        usage_limits_count: 0,
    };
    let object_states = [object_state];
    let context = kmipkit_profile_validation_context_v1_t {
        struct_size: u32::try_from(size_of::<kmipkit_profile_validation_context_v1_t>())
            .expect("C profile context size fits u32"),
        abi_version: 1,
        object_states: object_states.as_ptr(),
        object_state_count: 1,
    };

    assert_eq!(
        unsafe {
            kmipkit_profile_validation_result_validate_request_with_context_v1(
                result,
                request_handle,
                &raw const context,
            )
        },
        ERROR_INVALID_INPUT
    );
    assert_last_profile_validation_error_redacts(&[REQUEST_IDENTIFIER, CONTEXT_IDENTIFIER]);

    let response = profile_response_with_marker(0x0000_0031, RESPONSE_MARKER);
    let response_tree = response
        .with_ttlv(|tree| clone_structure_view(&tree))
        .expect("the response tree can be cloned for a borrowed ABI input");
    let response_handle = make_handle(Kind::Structure, HandleValue::Structure(response_tree));
    assert_eq!(
        unsafe {
            kmipkit_profile_validation_result_validate_response_with_context_v1(
                result,
                request_handle,
                response_handle,
                &raw const context,
            )
        },
        ERROR_INVALID_INPUT
    );
    assert_last_profile_validation_error_redacts(&[
        REQUEST_IDENTIFIER,
        CONTEXT_IDENTIFIER,
        RESPONSE_MARKER,
    ]);
    kmipkit_profile_validation_result_release(result);
    kmipkit_ttlv_structure_release(request_handle);
    kmipkit_ttlv_structure_release(response_handle);
}

fn assert_last_profile_validation_error_redacts(markers: &[&str]) {
    let (error, view) = last_profile_validation_view();
    let diagnostics = [
        span_text(view.error_code_data, view.error_code_length),
        span_text(view.profile_id_data, view.profile_id_length),
        span_text(view.requirement_id_data, view.requirement_id_length),
        span_text(view.source_id_data, view.source_id_length),
        span_text(view.source_locator_data, view.source_locator_length),
    ]
    .join(" ");
    for marker in markers {
        assert!(!diagnostics.contains(marker));
    }
    kmipkit_profile_validation_error_release(error);
}

#[cfg(feature = "coverage-c-consumer")]
#[test]
fn compiled_c_consumer_exercises_ordered_ids_transport_and_redacted_rule_failure() {
    const REQUEST_IDENTIFIER: &str = "request-id-must-not-leak";
    let request = profile_request([(
        0x0000_0031,
        profile_structure([
            profile_item(
                0x0042_0094,
                Value::text_string(REQUEST_IDENTIFIER.to_owned()),
            ),
            profile_item(
                0x0042_013d,
                Value::structure(profile_structure([profile_item(
                    0x0042_00bc,
                    Value::date_time(1_700_000_001),
                )])),
            ),
        ]),
    )]);
    let request_tree = request
        .with_ttlv(|tree| clone_structure_view(&tree))
        .expect("the request tree can be cloned for a borrowed C consumer input");
    let request_handle = make_handle(Kind::Structure, HandleValue::Structure(request_tree));
    let resolved = ProfileValidationSelection::for_test(["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        .validate()
        .expect("the test-only selector resolves the catalogued baseline rules");
    let result = make_handle(
        Kind::ProfileValidationResult,
        HandleValue::ProfileValidationResult(resolved),
    );

    // SAFETY: both handles are live and have their declared kinds for this C consumer call.
    assert_eq!(
        unsafe {
            kmipkit_client_abi_c_consumer_profile_validation_result_exercise(result, request_handle)
        },
        1
    );
    kmipkit_profile_validation_result_release(result);
    kmipkit_ttlv_structure_release(request_handle);
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
        kmipkit_last_error_profile_validation_error_get_v1(&raw mut handle),
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
    // SAFETY: `handle` is the live detail snapshot returned above. `view` is aligned and
    // initialized with the exact v1 size/version; this call has unique writable access to it.
    assert_eq!(
        unsafe { kmipkit_profile_validation_error_view_get_v1(handle, &raw mut view) },
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
    let detail_status = kmipkit_last_error_profile_validation_error_get_v1(&raw mut handle);
    let mut status = SUCCESS;
    let mut category = ERROR_CATEGORY_NONE;
    let mut cause = ERROR_CAUSE_NONE;
    let mut delivery = DELIVERY_STATE_NONE;
    assert_eq!(
        kmipkit_last_error_get(
            &raw mut status,
            &raw mut category,
            &raw mut cause,
            &raw mut delivery,
        ),
        SUCCESS
    );

    assert!(handle.is_null());
    assert_eq!(
        (detail_status, status, category, cause, delivery,),
        (
            SUCCESS,
            ERROR_TRANSPORT,
            ERROR_CATEGORY_TRANSPORT,
            ERROR_CAUSE_TRANSPORT_TIMEOUT,
            DELIVERY_STATE_POSSIBLY_SENT,
        )
    );
}

#[test]
fn source_gap_validation_reaches_c_view_with_catalogued_locator() {
    const SOURCE_GAP: &str = "KMIPKIT-SOURCE-GAP-SPEC-2.3-OPAQUE-DATA-TYPE-REQUIRED";
    const LOCATOR: &str =
        "Section 2.3, table object_definition, row Opaque Data Type, HTML lines 6927-6945";
    let validation = ProfileValidationSelection::for_test(["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        .validate()
        .expect("the existing test-support selector resolves catalogued rule mappings");
    let request = profile_request([(
        0x0000_0003,
        profile_structure([
            profile_item(0x0042_0057, Value::enumeration(8)),
            profile_item(0x0042_0125, Value::structure(Structure::new())),
            profile_item(
                0x0042_005b,
                Value::structure(profile_structure([profile_item(
                    0x0042_005a,
                    Value::byte_string(b"opaque payload marker".to_vec()),
                )])),
            ),
        ]),
    )]);
    let profile_error = validation
        .validate_request(&request)
        .expect_err("the missing Opaque Data Type violates its table-row requirement");
    assert_eq!(profile_error.source_clause_id(), None);
    assert_eq!(profile_error.source_gap_id(), Some(SOURCE_GAP));
    assert_eq!(profile_error.source_gap_locator(), Some(LOCATOR));

    let error = ClientError::profile_validation(profile_error, RequestDeliveryState::NotSent);
    assert_eq!(record_client_error(&error), ERROR_INVALID_INPUT);
    let (handle, view) = last_profile_validation_view();
    assert_eq!(view.source_kind, PROFILE_ERROR_SOURCE_GAP);
    assert_eq!(
        span_text(view.source_id_data, view.source_id_length),
        SOURCE_GAP
    );
    assert_eq!(
        span_text(view.source_locator_data, view.source_locator_length),
        LOCATOR
    );
    kmipkit_profile_validation_error_release(handle);
}

#[test]
fn contextual_validation_exposes_second_batch_item_index_in_c_view() {
    let validation = ProfileValidationSelection::for_test(["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        .validate()
        .expect("the existing test-support selector resolves catalogued rule mappings");
    let changed_attribute =
        profile_structure([profile_item(0x0042_00bc, Value::date_time(1_700_000_001))]);
    let request = profile_request([
        (0x0000_001e, Structure::new()),
        (
            0x0000_0031,
            profile_structure([
                profile_item(0x0042_0094, Value::text_string("context-target".to_owned())),
                profile_item(0x0042_013d, Value::structure(changed_attribute)),
            ]),
        ),
    ]);
    let context = ProfileValidationContext::new().with_object_state(
        1,
        ProfileObjectState::new("context-target").with_original_creation_date_present(Some(true)),
    );
    let profile_error = validation
        .validate_request_with_context(&request, &context)
        .expect_err("the second batch item attempts to modify known creation-date state");
    assert_eq!(profile_error.batch_item_index(), Some(1));

    let error = ClientError::profile_validation(profile_error, RequestDeliveryState::NotSent);
    assert_eq!(record_client_error(&error), ERROR_INVALID_INPUT);
    let (handle, view) = last_profile_validation_view();
    assert_eq!(view.batch_item_index_present, 1);
    assert_eq!(view.batch_item_index, 1);
    kmipkit_profile_validation_error_release(handle);
}

#[test]
fn unknown_profile_selector_keeps_profile_id_absent_in_c_view() {
    let unknown_id = "unpublished-profile-selector-sentinel";
    let profile_error = ProfileValidationSelection::new([unknown_id])
        .validate()
        .expect_err("an unknown selector cannot resolve to catalog metadata");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::UnknownProfile
    );
    assert_eq!(profile_error.profile_id(), None);
    assert!(!profile_error.to_string().contains(unknown_id));

    let error = ClientError::profile_validation(profile_error, RequestDeliveryState::NotSent);
    assert_eq!(record_client_error(&error), ERROR_INVALID_INPUT);
    let (handle, view) = last_profile_validation_view();
    assert!(view.profile_id_data.is_null());
    assert_eq!(view.profile_id_length, 0);
    assert_eq!(view.source_kind, PROFILE_ERROR_SOURCE_NONE);
    assert_eq!(
        span_text(view.error_code_data, view.error_code_length),
        "unknown_profile"
    );
    kmipkit_profile_validation_error_release(handle);
}

fn profile_item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag)
        .expect("profile fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("profile fixture tag is allocated");
    Item::new(tag, value).expect("profile fixture item is valid")
}

fn profile_structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for item in items {
        structure
            .try_push(item)
            .expect("profile fixture stays within the TTLV item limit");
    }
    structure
}

fn profile_request(items: impl IntoIterator<Item = (u32, Structure)>) -> RequestMessage {
    let items: Vec<_> = items.into_iter().collect();
    let item_count = items.len();
    let version = profile_structure([
        profile_item(0x0042_006a, Value::integer(2)),
        profile_item(0x0042_006b, Value::integer(1)),
    ]);
    let header = profile_structure([
        profile_item(0x0042_0069, Value::structure(version)),
        profile_item(
            0x0042_000d,
            Value::integer(i32::try_from(item_count).expect("fixture batch count fits i32")),
        ),
    ]);
    let mut root = vec![profile_item(0x0042_0077, Value::structure(header))];
    for (index, (operation, payload)) in items.into_iter().enumerate() {
        let mut batch_item = vec![profile_item(0x0042_005c, Value::enumeration(operation))];
        if item_count > 1 {
            batch_item.push(profile_item(
                0x0042_0093,
                Value::byte_string(format!("profile-test-item-{index}").into_bytes()),
            ));
        }
        batch_item.push(profile_item(0x0042_0079, Value::structure(payload)));
        root.push(profile_item(
            0x0042_000f,
            Value::structure(profile_structure(batch_item)),
        ));
    }
    RequestMessage::try_from_ttlv(profile_structure(root))
        .expect("profile test Request Message has a valid generic TTLV shape")
}

fn profile_response(operation: u32) -> ResponseMessage {
    ResponseMessage::try_from_ttlv(profile_structure([
        profile_item(
            0x0042_007a,
            Value::structure(profile_structure([
                profile_item(
                    0x0042_0069,
                    Value::structure(profile_structure([
                        profile_item(0x0042_006a, Value::integer(2)),
                        profile_item(0x0042_006b, Value::integer(1)),
                    ])),
                ),
                profile_item(0x0042_0092, Value::date_time(1_700_000_000)),
                profile_item(0x0042_000d, Value::integer(1)),
            ])),
        ),
        profile_item(
            0x0042_000f,
            Value::structure(profile_structure([
                profile_item(0x0042_005c, Value::enumeration(operation)),
                profile_item(0x0042_007f, Value::enumeration(0)),
                profile_item(0x0042_007c, Value::structure(Structure::new())),
            ])),
        ),
    ]))
    .expect("profile test Response Message has a valid generic TTLV shape")
}

fn profile_response_with_marker(operation: u32, marker: &str) -> ResponseMessage {
    ResponseMessage::try_from_ttlv(profile_structure([
        profile_item(
            0x0042_007a,
            Value::structure(profile_structure([
                profile_item(
                    0x0042_0069,
                    Value::structure(profile_structure([
                        profile_item(0x0042_006a, Value::integer(2)),
                        profile_item(0x0042_006b, Value::integer(1)),
                    ])),
                ),
                profile_item(0x0042_0092, Value::date_time(1_700_000_000)),
                profile_item(0x0042_000d, Value::integer(1)),
            ])),
        ),
        profile_item(
            0x0042_000f,
            Value::structure(profile_structure([
                profile_item(0x0042_005c, Value::enumeration(operation)),
                profile_item(0x0042_007f, Value::enumeration(0)),
                profile_item(
                    0x0042_007c,
                    Value::structure(profile_structure([profile_item(
                        0x0042_0094,
                        Value::text_string(marker.to_owned()),
                    )])),
                ),
            ])),
        ),
    ]))
    .expect("profile test Response Message with a marker has valid generic TTLV shape")
}

fn assert_profile_validation_outcome(
    status: i32,
    expected: Result<(), kmipkit_protocol::ProfileValidationError>,
) {
    match expected {
        Ok(()) => {
            assert_eq!(status, SUCCESS);
            assert!(last_profile_validation_error_is_absent());
        }
        Err(expected_error) => {
            assert_eq!(status, ERROR_INVALID_INPUT);
            let (error, view) = last_profile_validation_view();
            assert_eq!(
                span_text(view.error_code_data, view.error_code_length),
                expected_error.code().as_str()
            );
            kmipkit_profile_validation_error_release(error);
        }
    }
}

fn empty_profile_validation_view() -> kmipkit_profile_validation_error_view_v1_t {
    kmipkit_profile_validation_error_view_v1_t {
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
    }
}

fn last_profile_validation_view() -> (
    *mut kmipkit_profile_validation_error_t,
    kmipkit_profile_validation_error_view_v1_t,
) {
    let mut handle = ptr::null_mut();
    assert_eq!(
        kmipkit_last_error_profile_validation_error_get_v1(&raw mut handle),
        SUCCESS
    );
    assert!(!handle.is_null());
    let mut view = empty_profile_validation_view();
    // SAFETY: `handle` is a live detail snapshot returned by the getter. `view` is aligned and
    // initialized with the exact v1 size/version; this call has unique writable access to it.
    assert_eq!(
        unsafe { kmipkit_profile_validation_error_view_get_v1(handle, &raw mut view) },
        SUCCESS
    );
    (handle, view)
}

fn last_profile_validation_error_is_absent() -> bool {
    let mut handle = ptr::null_mut();
    kmipkit_last_error_profile_validation_error_get_v1(&raw mut handle) == SUCCESS
        && handle.is_null()
}

fn span_text(data: *const u8, length: u64) -> String {
    let Ok(length) = usize::try_from(length) else {
        panic!("profile error text length fits usize");
    };
    if length == 0 {
        return String::new();
    }
    assert!(!data.is_null());
    // SAFETY: the FFI view points into the live profile-error handle and this test keeps that
    // handle open until after the span has been copied.
    let bytes = unsafe { std::slice::from_raw_parts(data, length) };
    String::from_utf8(bytes.to_vec()).expect("profile error IDs are UTF-8")
}
