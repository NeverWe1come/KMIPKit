use super::*;

#[test]
// OASIS KMIP Specification v2.1 §4.38 paragraph 4 and Table 106.
fn original_creation_date_context_allows_only_the_first_addition() {
    const REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.38-003";
    const OBJECT_ID: &str = "original-creation-date-context-test";
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const NEW_ATTRIBUTE: u32 = 0x0042_013D;
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
    const ADD_ATTRIBUTE: u32 = 0x0000_000D;

    let request = request_message_with_operation_payload(
        ADD_ATTRIBUTE,
        structure([
            tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
            tagged(
                NEW_ATTRIBUTE,
                Value::structure(structure([tagged(
                    ORIGINAL_CREATION_DATE,
                    Value::date_time(1_700_000_000),
                )])),
            ),
        ]),
    );
    let selected = table12_profile_result(&[REQUIREMENT]);

    selected
        .validate_request_with_context(
            &request,
            &crate::ProfileValidationContext::new().with_object_state(
                0,
                crate::ProfileObjectState::new(OBJECT_ID)
                    .with_original_creation_date_present(Some(false)),
            ),
        )
        .expect("adding Original Creation Date is permitted when it is absent");

    for context in [
        crate::ProfileValidationContext::new().with_object_state(
            0,
            crate::ProfileObjectState::new(OBJECT_ID)
                .with_original_creation_date_present(Some(true)),
        ),
        crate::ProfileValidationContext::new(),
    ] {
        let error = selected
            .validate_request_with_context(&request, &context)
            .expect_err("existing or unknown object state cannot authorize a first addition");
        assert!(matches!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
                | ProfileValidationErrorCode::ProfileStateContextRequired
        ));
        assert_eq!(error.requirement_id(), Some(REQUIREMENT));
        assert_eq!(error.batch_item_index(), Some(0));
    }
}

#[test]
// OASIS KMIP Specification v2.1 §4.59 Table 148 and Table 149; KMIPKIT-DEC-024.
fn usage_limits_total_context_distinguishes_protection_from_count_allocation() {
    const GENERAL_REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-002";
    const ALLOCATION_REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-003-001";
    const OBJECT_ID: &str = "usage-limits-context-test";
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const NEW_ATTRIBUTE: u32 = 0x0042_013D;
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const UNIT: u32 = 0x0042_0098;
    const SET_ATTRIBUTE: u32 = 0x0000_0031;

    let request = request_message_with_operation_payload(
        SET_ATTRIBUTE,
        structure([
            tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
            tagged(
                NEW_ATTRIBUTE,
                Value::structure(structure([tagged(
                    USAGE_LIMITS,
                    Value::structure(structure([
                        tagged(TOTAL, Value::long_integer(40)),
                        tagged(COUNT, Value::long_integer(3)),
                        tagged(UNIT, Value::enumeration(1)),
                    ])),
                )])),
            ),
        ]),
    );
    let general = table12_profile_result(&[GENERAL_REQUIREMENT]);
    let allocation = table12_profile_result(&[ALLOCATION_REQUIREMENT]);
    let state = |protection, allocation_performed, allocation_changed_count| {
        crate::ProfileObjectState::new(OBJECT_ID)
            .with_usage_limits_present(Some(true))
            .with_protection_use_started(protection)
            .with_get_usage_allocation_performed(allocation_performed)
            .with_get_usage_allocation_changed_count(allocation_changed_count)
            .with_usage_limits_total(Some(50))
            .with_usage_limits_count(Some(3))
    };
    let context = |state| crate::ProfileValidationContext::new().with_object_state(0, state);

    general
        .validate_request_with_context(&request, &context(state(Some(false), Some(false), None)))
        .expect("a Total change before protection or allocation is allowed");

    let protection_error = general
        .validate_request_with_context(&request, &context(state(Some(true), Some(false), None)))
        .expect_err("Total is immutable after protection begins");
    assert_eq!(
        protection_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(protection_error.requirement_id(), Some(GENERAL_REQUIREMENT));

    general
        .validate_request_with_context(
            &request,
            &context(state(Some(false), Some(true), Some(true))),
        )
        .expect("the narrower allocation rule owns the changed-Count case");
    let allocation_error = allocation
        .validate_request_with_context(
            &request,
            &context(state(Some(false), Some(true), Some(true))),
        )
        .expect_err("Total is immutable after allocation changed Count");
    assert_eq!(
        allocation_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        allocation_error.requirement_id(),
        Some(ALLOCATION_REQUIREMENT)
    );

    let allocation_without_count_change = general
        .validate_request_with_context(
            &request,
            &context(state(Some(false), Some(true), Some(false))),
        )
        .expect_err("allocation also prevents Total changes when Count did not change");
    assert_eq!(
        allocation_without_count_change.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
}

#[test]
// OASIS KMIP Specification v2.1 §4.59 paragraph 3; KMIPKIT-DEC-024.
fn usage_limits_count_context_requires_the_existing_count_to_match() {
    const REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-003-002";
    const OBJECT_ID: &str = "usage-limits-count-context-test";
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const NEW_ATTRIBUTE: u32 = 0x0042_013D;
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const UNIT: u32 = 0x0042_0098;
    const SET_ATTRIBUTE: u32 = 0x0000_0031;

    let request = request_message_with_operation_payload(
        SET_ATTRIBUTE,
        structure([
            tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
            tagged(
                NEW_ATTRIBUTE,
                Value::structure(structure([tagged(
                    USAGE_LIMITS,
                    Value::structure(structure([
                        tagged(TOTAL, Value::long_integer(40)),
                        tagged(COUNT, Value::long_integer(3)),
                        tagged(UNIT, Value::enumeration(1)),
                    ])),
                )])),
            ),
        ]),
    );
    let selected = table12_profile_result(&[REQUIREMENT]);
    let context = |present, count| {
        crate::ProfileValidationContext::new().with_object_state(
            0,
            crate::ProfileObjectState::new(OBJECT_ID)
                .with_usage_limits_present(present)
                .with_usage_limits_count(count),
        )
    };

    selected
        .validate_request_with_context(&request, &context(Some(true), Some(3)))
        .expect("setting the already allocated Count value is permitted");

    let mismatch = selected
        .validate_request_with_context(&request, &context(Some(true), Some(2)))
        .expect_err("changing Usage Limits Count is prohibited");
    assert_eq!(
        mismatch.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(mismatch.requirement_id(), Some(REQUIREMENT));

    let unknown = selected
        .validate_request_with_context(&request, &context(None, Some(3)))
        .expect_err("unknown Usage Limits presence cannot authorize the operation");
    assert_eq!(
        unknown.code(),
        ProfileValidationErrorCode::ProfileStateContextRequired
    );
    assert_eq!(unknown.requirement_id(), Some(REQUIREMENT));
}

#[test]
// OASIS KMIP Specification v2.1 §§9.8 and 9.21.
fn client_response_operation_matches_the_operation_it_was_sent_for() {
    const FIRST_OPERATION: u32 = 0x0000_000A;
    const SECOND_OPERATION: u32 = 0x0000_0008;
    let request = request_message_with_two_operation_payloads(
        FIRST_OPERATION,
        Structure::new(),
        SECOND_OPERATION,
        Structure::new(),
    );

    let reordered = response_message_with_success_batch_payloads([
        (SECOND_OPERATION, b"second-item".to_vec(), Structure::new()),
        (FIRST_OPERATION, b"first-item".to_vec(), Structure::new()),
    ]);
    assert!(super::client_understands_response_operations(
        None,
        Some((&request, &reordered))
    ));
    assert!(super::client_understands_response_operations(None, None));

    let single_request = request_message_with_operation_payload(FIRST_OPERATION, Structure::new());
    let single_response = response_message_with_operation_status(
        Some(FIRST_OPERATION),
        0,
        None,
        Some(Structure::new()),
    );
    assert!(super::client_understands_response_operations(
        None,
        Some((&single_request, &single_response))
    ));
    let response_without_multi_item_id = response_message_with_operation_status(
        Some(FIRST_OPERATION),
        0,
        None,
        Some(Structure::new()),
    );
    assert!(!super::client_understands_response_operations(
        None,
        Some((&request, &response_without_multi_item_id))
    ));

    let mismatched_operation = response_message_with_success_batch_payloads([
        (SECOND_OPERATION, b"first-item".to_vec(), Structure::new()),
        (FIRST_OPERATION, b"second-item".to_vec(), Structure::new()),
    ]);
    assert!(!super::client_understands_response_operations(
        None,
        Some((&request, &mismatched_operation))
    ));

    let unknown_identifier = response_message_with_success_batch_payloads([(
        FIRST_OPERATION,
        b"unknown-item".to_vec(),
        Structure::new(),
    )]);
    assert!(!super::client_understands_response_operations(
        None,
        Some((&request, &unknown_identifier))
    ));

    let duplicate_response_identifier = response_message_with_success_batch_payloads([
        (FIRST_OPERATION, b"first-item".to_vec(), Structure::new()),
        (FIRST_OPERATION, b"first-item".to_vec(), Structure::new()),
    ]);
    assert!(!super::client_understands_response_operations(
        None,
        Some((&request, &duplicate_response_identifier))
    ));

    let too_many_response_items = response_message_with_success_batch_payloads([
        (FIRST_OPERATION, b"first-item".to_vec(), Structure::new()),
        (SECOND_OPERATION, b"second-item".to_vec(), Structure::new()),
        (SECOND_OPERATION, b"extra-item".to_vec(), Structure::new()),
    ]);
    assert!(!super::client_understands_response_operations(
        None,
        Some((&request, &too_many_response_items))
    ));
}

#[test]
// OASIS KMIP Specification v2.1 §4.38 paragraph 4 and Table 106.
fn original_creation_date_context_rejects_malformed_targets_and_non_add_changes() {
    const REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.38-003";
    const OBJECT_ID: &str = "original-creation-date-malformed-context-test";
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const NEW_ATTRIBUTE: u32 = 0x0042_013D;
    const CURRENT_ATTRIBUTE: u32 = 0x0042_013C;
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
    const ADD_ATTRIBUTE: u32 = 0x0000_000D;
    const MODIFY_ATTRIBUTE: u32 = 0x0000_000E;

    let selected = table12_profile_result(&[REQUIREMENT]);
    let malformed_target = request_message_with_operation_payload(
        ADD_ATTRIBUTE,
        structure([
            tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
            tagged(
                NEW_ATTRIBUTE,
                Value::text_string("not an Attribute structure".to_owned()),
            ),
        ]),
    );
    let malformed_error = selected
        .validate_request_with_context(&malformed_target, &crate::ProfileValidationContext::new())
        .expect_err("a malformed target wrapper fails closed before state lookup");
    assert_eq!(
        malformed_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );

    let unknown_fact = request_message_with_operation_payload(
        ADD_ATTRIBUTE,
        structure([
            tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
            tagged(
                NEW_ATTRIBUTE,
                Value::structure(structure([tagged(
                    ORIGINAL_CREATION_DATE,
                    Value::date_time(1_700_000_000),
                )])),
            ),
        ]),
    );
    let unknown_error = selected
        .validate_request_with_context(
            &unknown_fact,
            &crate::ProfileValidationContext::new()
                .with_object_state(0, crate::ProfileObjectState::new(OBJECT_ID)),
        )
        .expect_err("an existing state entry with an unknown fact is insufficient");
    assert_eq!(
        unknown_error.code(),
        ProfileValidationErrorCode::ProfileStateContextRequired
    );

    let modify = request_message_with_operation_payload(
        MODIFY_ATTRIBUTE,
        structure([
            tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
            tagged(
                CURRENT_ATTRIBUTE,
                Value::structure(structure([tagged(
                    ORIGINAL_CREATION_DATE,
                    Value::date_time(1_700_000_000),
                )])),
            ),
        ]),
    );
    let modify_error = selected
        .validate_request_with_context(
            &modify,
            &crate::ProfileValidationContext::new().with_object_state(
                0,
                crate::ProfileObjectState::new(OBJECT_ID)
                    .with_original_creation_date_present(Some(false)),
            ),
        )
        .expect_err("Modify Attribute cannot change Original Creation Date");
    assert_eq!(
        modify_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );

    let delete_payload = crate::DeleteAttributeRequest::new(
        Some(OBJECT_ID.to_owned()),
        None,
        Some(crate::AttributeReference::tag(ORIGINAL_CREATION_DATE)),
    )
    .to_ttlv_payload()
    .expect("the standard Original Creation Date tag is a valid Attribute Reference");
    let delete = request_message_with_operation_payload(0x0000_000F, delete_payload);
    let delete_error = selected
        .validate_request_with_context(
            &delete,
            &crate::ProfileValidationContext::new().with_object_state(
                0,
                crate::ProfileObjectState::new(OBJECT_ID)
                    .with_original_creation_date_present(Some(true)),
            ),
        )
        .expect_err("Delete Attribute cannot remove Original Creation Date");
    assert_eq!(
        delete_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
}

#[test]
// OASIS KMIP Specification v2.1 §4.59 Table 148 and Table 149; KMIPKIT-DEC-024.
fn usage_limits_context_requires_known_protection_and_allocation_facts() {
    const GENERAL_REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-002";
    const ALLOCATION_REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-003-001";
    const OBJECT_ID: &str = "usage-limits-unknown-context-test";
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const NEW_ATTRIBUTE: u32 = 0x0042_013D;
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const UNIT: u32 = 0x0042_0098;
    const SET_ATTRIBUTE: u32 = 0x0000_0031;

    let request_with_total = |total| {
        request_message_with_operation_payload(
            SET_ATTRIBUTE,
            structure([
                tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
                tagged(
                    NEW_ATTRIBUTE,
                    Value::structure(structure([tagged(
                        USAGE_LIMITS,
                        Value::structure(structure([
                            tagged(TOTAL, Value::long_integer(total)),
                            tagged(COUNT, Value::long_integer(3)),
                            tagged(UNIT, Value::enumeration(1)),
                        ])),
                    )])),
                ),
            ]),
        )
    };
    let selected_general = table12_profile_result(&[GENERAL_REQUIREMENT]);
    let selected_allocation = table12_profile_result(&[ALLOCATION_REQUIREMENT]);
    let state = |protection, allocation, changed_count, total| {
        crate::ProfileObjectState::new(OBJECT_ID)
            .with_usage_limits_present(Some(true))
            .with_protection_use_started(protection)
            .with_get_usage_allocation_performed(allocation)
            .with_get_usage_allocation_changed_count(changed_count)
            .with_usage_limits_total(total)
            .with_usage_limits_count(Some(3))
    };
    let context = |state| crate::ProfileValidationContext::new().with_object_state(0, state);

    for state in [
        state(None, Some(false), None, Some(50)),
        state(Some(false), None, None, Some(50)),
    ] {
        let error = selected_general
            .validate_request_with_context(&request_with_total(40), &context(state))
            .expect_err("an unknown protection or allocation fact fails closed");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::ProfileStateContextRequired
        );
    }

    selected_general
        .validate_request_with_context(
            &request_with_total(50),
            &context(state(None, None, None, Some(50))),
        )
        .expect("an unchanged Total does not require protection history");

    let missing_total = selected_general
        .validate_request_with_context(
            &request_with_total(40),
            &context(state(Some(false), Some(false), None, None)),
        )
        .expect_err("a changed Total requires its current value");
    assert_eq!(
        missing_total.code(),
        ProfileValidationErrorCode::ProfileStateContextRequired
    );

    let allocation_mismatch = selected_allocation
        .validate_request_with_context(
            &request_with_total(40),
            &context(state(Some(false), Some(false), Some(true), Some(50))),
        )
        .expect_err("Count cannot be reported changed when allocation was not performed");
    assert_eq!(
        allocation_mismatch.code(),
        ProfileValidationErrorCode::ProfileStateBindingMismatch
    );

    selected_allocation
        .validate_request_with_context(
            &request_with_total(40),
            &context(state(Some(false), Some(true), Some(false), Some(50))),
        )
        .expect("allocation without a changed Count does not trigger the narrower rule");
}

#[test]
// OASIS KMIP Specification v2.1 §4.59 paragraph 3; KMIPKIT-DEC-024.
fn usage_limits_count_context_rejects_malformed_and_unknown_count_values() {
    const REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-003-002";
    const OBJECT_ID: &str = "usage-limits-malformed-count-context-test";
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const NEW_ATTRIBUTE: u32 = 0x0042_013D;
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const UNIT: u32 = 0x0042_0098;
    const SET_ATTRIBUTE: u32 = 0x0000_0031;

    let request_with_count = |count| {
        request_message_with_operation_payload(
            SET_ATTRIBUTE,
            structure([
                tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
                tagged(
                    NEW_ATTRIBUTE,
                    Value::structure(structure([tagged(
                        USAGE_LIMITS,
                        Value::structure(structure([
                            tagged(TOTAL, Value::long_integer(40)),
                            tagged(COUNT, count),
                            tagged(UNIT, Value::enumeration(1)),
                        ])),
                    )])),
                ),
            ]),
        )
    };
    let selected = table12_profile_result(&[REQUIREMENT]);
    let context = |count| {
        crate::ProfileValidationContext::new().with_object_state(
            0,
            crate::ProfileObjectState::new(OBJECT_ID)
                .with_usage_limits_present(Some(true))
                .with_usage_limits_count(count),
        )
    };

    let malformed = selected
        .validate_request_with_context(&request_with_count(Value::integer(3)), &context(Some(3)))
        .expect_err("Usage Limits Count is a Long Integer in the client profile");
    assert_eq!(
        malformed.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );

    let unknown = selected
        .validate_request_with_context(&request_with_count(Value::long_integer(3)), &context(None))
        .expect_err("the present attribute still needs its current Count");
    assert_eq!(
        unknown.code(),
        ProfileValidationErrorCode::ProfileStateContextRequired
    );
}

#[test]
// OASIS KMIP Specification v2.1 §§4.59 and 5.5; KMIPKIT-DEC-024.
// Keep the malformed wrapper and reference branches on the same shared payload fixture.
#[allow(clippy::too_many_lines)]
fn usage_limits_helpers_cover_malformed_wrappers_and_reference_forms() {
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const UNIT: u32 = 0x0042_0098;
    const NEW_ATTRIBUTE: u32 = 0x0042_013D;
    const ATTRIBUTE_REFERENCE: u32 = 0x0042_013B;
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
    const SET_ATTRIBUTE: u32 = 0x0000_0031;
    const ADD_ATTRIBUTE: u32 = 0x0000_000D;
    const MODIFY_ATTRIBUTE: u32 = 0x0000_000E;
    const DELETE_ATTRIBUTE: u32 = 0x0000_000F;
    const ADJUST_ATTRIBUTE: u32 = 0x0000_0030;

    let usage_limits = || {
        Value::structure(structure([
            tagged(TOTAL, Value::long_integer(40)),
            tagged(COUNT, Value::long_integer(3)),
            tagged(UNIT, Value::enumeration(1)),
        ]))
    };
    let valid_payload = structure([tagged(
        NEW_ATTRIBUTE,
        Value::structure(structure([tagged(USAGE_LIMITS, usage_limits())])),
    )]);
    assert!(matches!(
        super::request_usage_total_mutation(&valid_payload.view(), SET_ATTRIBUTE),
        Ok(Some(super::UsageTotalMutation::Set(40)))
    ));

    let empty = Structure::new();
    assert!(matches!(
        super::usage_total_in_wrapper(&empty.view(), NEW_ATTRIBUTE),
        Ok(None)
    ));
    assert!(matches!(
        super::usage_limits_count_in_wrapper(&empty.view(), NEW_ATTRIBUTE),
        Ok(None)
    ));

    let wrong_wrapper = structure([tagged(
        NEW_ATTRIBUTE,
        Value::text_string("not a structure".to_owned()),
    )]);
    assert!(super::usage_total_in_wrapper(&wrong_wrapper.view(), NEW_ATTRIBUTE).is_err());
    assert!(super::usage_limits_count_in_wrapper(&wrong_wrapper.view(), NEW_ATTRIBUTE).is_err());

    let wrong_usage_limits_value = structure([tagged(
        NEW_ATTRIBUTE,
        Value::structure(structure([tagged(
            USAGE_LIMITS,
            Value::text_string("not a Usage Limits structure".to_owned()),
        )])),
    )]);
    assert!(
        super::usage_total_in_wrapper(&wrong_usage_limits_value.view(), NEW_ATTRIBUTE).is_err()
    );
    assert!(
        super::usage_limits_count_in_wrapper(&wrong_usage_limits_value.view(), NEW_ATTRIBUTE)
            .is_err()
    );

    let duplicate_wrapper = structure([
        tagged(NEW_ATTRIBUTE, Value::structure(Structure::new())),
        tagged(NEW_ATTRIBUTE, Value::structure(Structure::new())),
    ]);
    assert!(super::usage_total_in_wrapper(&duplicate_wrapper.view(), NEW_ATTRIBUTE).is_err());
    assert!(
        super::usage_limits_count_in_wrapper(&duplicate_wrapper.view(), NEW_ATTRIBUTE).is_err()
    );

    let no_usage_limits = structure([tagged(
        NEW_ATTRIBUTE,
        Value::structure(structure([tagged(TOTAL, Value::long_integer(40))])),
    )]);
    assert!(matches!(
        super::usage_total_in_wrapper(&no_usage_limits.view(), NEW_ATTRIBUTE),
        Ok(None)
    ));
    assert!(matches!(
        super::usage_limits_count_in_wrapper(&no_usage_limits.view(), NEW_ATTRIBUTE),
        Ok(None)
    ));

    let duplicate_usage_limits = structure([tagged(
        NEW_ATTRIBUTE,
        Value::structure(structure([
            tagged(USAGE_LIMITS, usage_limits()),
            tagged(USAGE_LIMITS, usage_limits()),
        ])),
    )]);
    assert!(super::usage_total_in_wrapper(&duplicate_usage_limits.view(), NEW_ATTRIBUTE).is_err());
    assert!(
        super::usage_limits_count_in_wrapper(&duplicate_usage_limits.view(), NEW_ATTRIBUTE)
            .is_err()
    );

    let missing_total = structure([tagged(
        NEW_ATTRIBUTE,
        Value::structure(structure([tagged(
            USAGE_LIMITS,
            Value::structure(structure([
                tagged(COUNT, Value::long_integer(3)),
                tagged(UNIT, Value::enumeration(1)),
            ])),
        )])),
    )]);
    assert!(super::usage_total_in_wrapper(&missing_total.view(), NEW_ATTRIBUTE).is_err());

    let duplicate_total = structure([tagged(
        NEW_ATTRIBUTE,
        Value::structure(structure([tagged(
            USAGE_LIMITS,
            Value::structure(structure([
                tagged(TOTAL, Value::long_integer(40)),
                tagged(TOTAL, Value::long_integer(41)),
                tagged(COUNT, Value::long_integer(3)),
                tagged(UNIT, Value::enumeration(1)),
            ])),
        )])),
    )]);
    assert!(super::usage_total_in_wrapper(&duplicate_total.view(), NEW_ATTRIBUTE).is_err());

    let malformed_total = structure([tagged(
        NEW_ATTRIBUTE,
        Value::structure(structure([tagged(
            USAGE_LIMITS,
            Value::structure(structure([
                tagged(TOTAL, Value::integer(40)),
                tagged(COUNT, Value::long_integer(3)),
                tagged(UNIT, Value::enumeration(1)),
            ])),
        )])),
    )]);
    assert!(super::usage_total_in_wrapper(&malformed_total.view(), NEW_ATTRIBUTE).is_err());

    let malformed_count = structure([tagged(
        NEW_ATTRIBUTE,
        Value::structure(structure([tagged(
            USAGE_LIMITS,
            Value::structure(structure([
                tagged(TOTAL, Value::long_integer(40)),
                tagged(COUNT, Value::integer(3)),
                tagged(UNIT, Value::enumeration(1)),
            ])),
        )])),
    )]);
    assert!(super::usage_limits_count_in_wrapper(&malformed_count.view(), NEW_ATTRIBUTE).is_err());

    let missing_reference = Structure::new();
    assert!(
        !super::reference_targets_attribute(&missing_reference.view(), USAGE_LIMITS)
            .expect("an omitted reference is not a malformed request")
    );
    let tag_reference = crate::AttributeReference::tag(USAGE_LIMITS)
        .to_ttlv_item()
        .expect("Usage Limits has an assigned tag");
    let tag_reference = structure([tag_reference]);
    assert!(
        super::reference_targets_attribute(&tag_reference.view(), USAGE_LIMITS)
            .expect("the assigned tag identifies Usage Limits")
    );
    assert!(
        !super::reference_targets_attribute(&tag_reference.view(), ORIGINAL_CREATION_DATE)
            .expect("a different assigned tag does not match")
    );

    let name_reference = crate::AttributeReference::name("", "Usage Limits")
        .to_ttlv_item()
        .expect("the standard attribute name can omit vendor identification");
    assert!(
        super::reference_targets_attribute(&structure([name_reference]).view(), USAGE_LIMITS)
            .expect("the built-in name identifies Usage Limits")
    );
    let vendor_reference = crate::AttributeReference::name("example.org", "Usage Limits")
        .to_ttlv_item()
        .expect("the vendor name reference is structurally valid");
    assert!(
        !super::reference_targets_attribute(&structure([vendor_reference]).view(), USAGE_LIMITS)
            .expect("a vendor name does not alias a built-in attribute")
    );
    let malformed_reference = structure([tagged(
        ATTRIBUTE_REFERENCE,
        Value::text_string("invalid reference value".to_owned()),
    )]);
    assert!(super::reference_targets_attribute(&malformed_reference.view(), USAGE_LIMITS).is_err());
    let creation_date_name = crate::AttributeReference::name("", "Original Creation Date")
        .to_ttlv_item()
        .expect("the standard date attribute has a valid name reference");
    assert!(
        super::reference_targets_attribute(
            &structure([creation_date_name]).view(),
            ORIGINAL_CREATION_DATE
        )
        .expect("the built-in name identifies Original Creation Date")
    );

    let duplicate_reference = structure([
        crate::AttributeReference::tag(USAGE_LIMITS)
            .to_ttlv_item()
            .expect("Usage Limits has an assigned tag"),
        crate::AttributeReference::tag(USAGE_LIMITS)
            .to_ttlv_item()
            .expect("Usage Limits has an assigned tag"),
    ]);
    assert!(super::reference_targets_attribute(&duplicate_reference.view(), USAGE_LIMITS).is_err());

    let current_attribute = structure([tagged(
        0x0042_013C,
        Value::structure(structure([tagged(USAGE_LIMITS, usage_limits())])),
    )]);
    assert!(matches!(
        super::request_usage_total_mutation(&current_attribute.view(), MODIFY_ATTRIBUTE),
        Ok(Some(super::UsageTotalMutation::Remove))
    ));
    assert!(matches!(
        super::request_usage_total_mutation(&current_attribute.view(), DELETE_ATTRIBUTE),
        Ok(Some(super::UsageTotalMutation::Remove))
    ));
    assert!(matches!(
        super::request_usage_total_mutation(&tag_reference.view(), DELETE_ATTRIBUTE),
        Ok(Some(super::UsageTotalMutation::Remove))
    ));
    assert!(matches!(
        super::request_usage_total_mutation(&tag_reference.view(), ADJUST_ATTRIBUTE),
        Ok(Some(super::UsageTotalMutation::Indeterminate))
    ));
    assert!(matches!(
        super::request_usage_total_mutation(&valid_payload.view(), ADD_ATTRIBUTE),
        Ok(Some(super::UsageTotalMutation::Set(40)))
    ));
    assert!(matches!(
        super::request_usage_total_mutation(&empty.view(), ADJUST_ATTRIBUTE),
        Ok(None)
    ));
    assert!(matches!(
        super::request_usage_total_mutation(&empty.view(), 0xFFFF_FFFF),
        Ok(None)
    ));
}

#[test]
// OASIS KMIP Specification v2.1 §4.59 Tables 148–149; KMIPKIT-DEC-024.
// Keep the partial state matrix together because each outcome evaluates the same mutation.
#[allow(clippy::too_many_lines)]
fn usage_limits_state_helpers_reject_unbound_and_indeterminate_state() {
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const UNIT: u32 = 0x0042_0098;
    const NEW_ATTRIBUTE: u32 = 0x0042_013D;
    const OBJECT_ID: &str = "usage-limits-state-helper-test";
    const ADD_ATTRIBUTE: u32 = 0x0000_000D;
    const SET_ATTRIBUTE: u32 = 0x0000_0031;

    let payload = structure([tagged(
        UNIQUE_IDENTIFIER,
        Value::text_string(OBJECT_ID.to_owned()),
    )]);
    let matching_state = crate::ProfileObjectState::new(OBJECT_ID);
    let matching_context =
        crate::ProfileValidationContext::new().with_object_state(0, matching_state);
    assert!(
        super::bound_object_state(
            &payload.view(),
            &matching_context,
            "profile-test",
            "KMIPKIT-REQ-SPEC-4.59-002",
            "KMIPKIT-CLAUSE-SPEC-4.59-002",
            0
        )
        .is_ok()
    );

    for malformed_identifier in [
        Structure::new(),
        structure([tagged(UNIQUE_IDENTIFIER, Value::text_string(String::new()))]),
        structure([tagged(UNIQUE_IDENTIFIER, Value::integer(1))]),
        structure([
            tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
            tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
        ]),
    ] {
        let error = super::bound_object_state(
            &malformed_identifier.view(),
            &matching_context,
            "profile-test",
            "KMIPKIT-REQ-SPEC-4.59-002",
            "KMIPKIT-CLAUSE-SPEC-4.59-002",
            0,
        )
        .expect_err("invalid identifiers cannot bind contextual object state");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::ProfileStateBindingMismatch
        );
    }

    let missing_state = super::bound_object_state(
        &payload.view(),
        &crate::ProfileValidationContext::new(),
        "profile-test",
        "KMIPKIT-REQ-SPEC-4.59-002",
        "KMIPKIT-CLAUSE-SPEC-4.59-002",
        0,
    )
    .expect_err("the corresponding object state is required");
    assert_eq!(
        missing_state.code(),
        ProfileValidationErrorCode::ProfileStateContextRequired
    );
    let mismatched_state = super::bound_object_state(
        &payload.view(),
        &crate::ProfileValidationContext::new()
            .with_object_state(0, crate::ProfileObjectState::new("different-object")),
        "profile-test",
        "KMIPKIT-REQ-SPEC-4.59-002",
        "KMIPKIT-CLAUSE-SPEC-4.59-002",
        0,
    )
    .expect_err("state bound to a different identifier cannot authorize the request");
    assert_eq!(
        mismatched_state.code(),
        ProfileValidationErrorCode::ProfileStateBindingMismatch
    );

    let new_total = structure([tagged(
        NEW_ATTRIBUTE,
        Value::structure(structure([tagged(
            USAGE_LIMITS,
            Value::structure(structure([
                tagged(TOTAL, Value::long_integer(40)),
                tagged(COUNT, Value::long_integer(3)),
                tagged(UNIT, Value::enumeration(1)),
            ])),
        )])),
    )]);
    let requested = super::request_usage_total_mutation(&new_total.view(), SET_ATTRIBUTE)
        .expect("the Usage Limits Total is well formed")
        .expect("setting Total is a mutation");
    let absent = crate::ProfileObjectState::new(OBJECT_ID).with_usage_limits_present(Some(false));
    assert_eq!(
        super::usage_total_is_changed(requested, ADD_ATTRIBUTE, &absent),
        Ok(false)
    );
    assert_eq!(
        super::usage_total_is_changed(requested, SET_ATTRIBUTE, &absent),
        Err(ProfileValidationErrorCode::ProfileStateContextRequired)
    );
    let unknown = crate::ProfileObjectState::new(OBJECT_ID);
    assert_eq!(
        super::usage_total_is_changed(requested, SET_ATTRIBUTE, &unknown),
        Err(ProfileValidationErrorCode::ProfileStateContextRequired)
    );
    let missing_total =
        crate::ProfileObjectState::new(OBJECT_ID).with_usage_limits_present(Some(true));
    assert_eq!(
        super::usage_total_is_changed(requested, SET_ATTRIBUTE, &missing_total),
        Err(ProfileValidationErrorCode::ProfileStateContextRequired)
    );
    let unchanged = crate::ProfileObjectState::new(OBJECT_ID)
        .with_usage_limits_present(Some(true))
        .with_usage_limits_total(Some(40));
    assert_eq!(
        super::usage_total_is_changed(requested, SET_ATTRIBUTE, &unchanged),
        Ok(false)
    );
    let changed = crate::ProfileObjectState::new(OBJECT_ID)
        .with_usage_limits_present(Some(true))
        .with_usage_limits_total(Some(50));
    assert_eq!(
        super::usage_total_is_changed(requested, SET_ATTRIBUTE, &changed),
        Ok(true)
    );
    assert_eq!(
        super::usage_total_is_changed(
            super::UsageTotalMutation::Indeterminate,
            SET_ATTRIBUTE,
            &changed
        ),
        Err(ProfileValidationErrorCode::ProfileStateContextRequired)
    );
    assert_eq!(
        super::usage_total_is_changed(
            super::UsageTotalMutation::Remove,
            SET_ATTRIBUTE,
            &crate::ProfileObjectState::new(OBJECT_ID).with_usage_limits_present(Some(true))
        ),
        Ok(true)
    );
    assert_eq!(
        super::usage_total_is_changed(super::UsageTotalMutation::Remove, SET_ATTRIBUTE, &absent),
        Ok(false)
    );
    assert_eq!(
        super::usage_total_is_changed(super::UsageTotalMutation::Remove, SET_ATTRIBUTE, &unknown),
        Err(ProfileValidationErrorCode::ProfileStateContextRequired)
    );
}

#[test]
// OASIS KMIP Specification v2.1 §4.59 Tables 148–149; KMIPKIT-DEC-024.
fn usage_limits_profile_context_rejects_malformed_or_unbound_mutations() {
    const GENERAL_REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-002";
    const ALLOCATION_REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-003-001";
    const COUNT_REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-003-002";
    const OBJECT_ID: &str = "usage-limits-malformed-context-test";
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const NEW_ATTRIBUTE: u32 = 0x0042_013D;
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const UNIT: u32 = 0x0042_0098;
    const SET_ATTRIBUTE: u32 = 0x0000_0031;

    let malformed = request_message_with_operation_payload(
        SET_ATTRIBUTE,
        structure([
            tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
            tagged(
                NEW_ATTRIBUTE,
                Value::text_string("not an Attribute structure".to_owned()),
            ),
        ]),
    );
    let context = crate::ProfileValidationContext::new().with_object_state(
        0,
        crate::ProfileObjectState::new(OBJECT_ID)
            .with_usage_limits_present(Some(true))
            .with_usage_limits_total(Some(50))
            .with_usage_limits_count(Some(3))
            .with_protection_use_started(Some(false))
            .with_get_usage_allocation_performed(Some(false)),
    );
    for requirement in [
        GENERAL_REQUIREMENT,
        ALLOCATION_REQUIREMENT,
        COUNT_REQUIREMENT,
    ] {
        let error = table12_profile_result(&[requirement])
            .validate_request_with_context(&malformed, &context)
            .expect_err("malformed Usage Limits wrappers fail closed for each contextual rule");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(requirement));
    }

    let total_without_identifier = request_message_with_operation_payload(
        SET_ATTRIBUTE,
        structure([tagged(
            NEW_ATTRIBUTE,
            Value::structure(structure([tagged(
                USAGE_LIMITS,
                Value::structure(structure([
                    tagged(TOTAL, Value::long_integer(40)),
                    tagged(COUNT, Value::long_integer(3)),
                    tagged(UNIT, Value::enumeration(1)),
                ])),
            )])),
        )]),
    );
    for requirement in [GENERAL_REQUIREMENT, ALLOCATION_REQUIREMENT] {
        let error = table12_profile_result(&[requirement])
            .validate_request_with_context(
                &total_without_identifier,
                &crate::ProfileValidationContext::new(),
            )
            .expect_err("contextual Total changes require a bound Unique Identifier");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::ProfileStateBindingMismatch
        );
        assert_eq!(error.requirement_id(), Some(requirement));
    }

    let unrelated_set = request_message_with_operation_payload(
        SET_ATTRIBUTE,
        structure([tagged(
            UNIQUE_IDENTIFIER,
            Value::text_string(OBJECT_ID.to_owned()),
        )]),
    );
    for requirement in [
        GENERAL_REQUIREMENT,
        ALLOCATION_REQUIREMENT,
        COUNT_REQUIREMENT,
    ] {
        table12_profile_result(&[requirement])
            .validate_request(&unrelated_set)
            .expect("a Set Attribute request unrelated to Usage Limits needs no extra state");
    }
}

#[test]
// OASIS KMIP Specification v2.1 §§8.3, 9.8, and 9.21.
fn obtain_lease_response_rule_rejects_ambiguous_batch_correlation() {
    const DISCOVER_VERSIONS: u32 = 0x0000_001E;
    const GET: u32 = 0x0000_0008;

    let single_without_id = request_message_with_unique_batch_item_id(None);
    let single_response_without_id = response_message_with_operation_status(
        Some(DISCOVER_VERSIONS),
        0,
        None,
        Some(Structure::new()),
    );
    assert!(super::client_obtain_lease_response_values_are_typed(
        None,
        Some((&single_without_id, &single_response_without_id))
    ));

    let one_request = request_message_with_batch_item_ids(&[b"first"]);
    let too_many_responses = response_message_with_success_batch_payloads([
        (DISCOVER_VERSIONS, b"first".to_vec(), Structure::new()),
        (DISCOVER_VERSIONS, b"second".to_vec(), Structure::new()),
    ]);
    assert!(!super::client_obtain_lease_response_values_are_typed(
        None,
        Some((&one_request, &too_many_responses))
    ));

    let unknown_identifier = response_message_with_success_batch_payloads([(
        DISCOVER_VERSIONS,
        b"unknown".to_vec(),
        Structure::new(),
    )]);
    assert!(!super::client_obtain_lease_response_values_are_typed(
        None,
        Some((&one_request, &unknown_identifier))
    ));

    let two_requests = request_message_with_batch_item_ids(&[b"first", b"second"]);
    let duplicate_response = response_message_with_success_batch_payloads([
        (DISCOVER_VERSIONS, b"first".to_vec(), Structure::new()),
        (DISCOVER_VERSIONS, b"first".to_vec(), Structure::new()),
    ]);
    assert!(!super::client_obtain_lease_response_values_are_typed(
        None,
        Some((&two_requests, &duplicate_response))
    ));

    let response_without_multi_item_id = response_message_with_operation_status(
        Some(DISCOVER_VERSIONS),
        0,
        None,
        Some(Structure::new()),
    );
    assert!(!super::client_obtain_lease_response_values_are_typed(
        None,
        Some((&two_requests, &response_without_multi_item_id))
    ));

    let response_without_operation =
        response_message_with_operation_status(None, 0, None, Some(Structure::new()));
    assert!(!super::client_obtain_lease_response_values_are_typed(
        None,
        Some((&two_requests, &response_without_operation))
    ));

    let mismatched_operation =
        response_message_with_success_batch_payloads([(GET, b"first".to_vec(), Structure::new())]);
    assert!(!super::client_obtain_lease_response_values_are_typed(
        None,
        Some((&one_request, &mismatched_operation))
    ));
}

#[test]
// OASIS KMIP Specification v2.1 §4.38 paragraph 4 and Table 106.
fn original_creation_date_context_rejects_unbound_and_duplicate_targets() {
    const REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.38-003";
    const OBJECT_ID: &str = "original-creation-date-unbound-context-test";
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const NEW_ATTRIBUTE: u32 = 0x0042_013D;
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
    const ADD_ATTRIBUTE: u32 = 0x0000_000D;

    let original_creation_date = || {
        tagged(
            NEW_ATTRIBUTE,
            Value::structure(structure([tagged(
                ORIGINAL_CREATION_DATE,
                Value::date_time(1_700_000_000),
            )])),
        )
    };
    let selected = table12_profile_result(&[REQUIREMENT]);
    let missing_identifier = request_message_with_operation_payload(
        ADD_ATTRIBUTE,
        structure([original_creation_date()]),
    );
    let error = selected
        .validate_request_with_context(
            &missing_identifier,
            &crate::ProfileValidationContext::new().with_object_state(
                0,
                crate::ProfileObjectState::new(OBJECT_ID)
                    .with_original_creation_date_present(Some(false)),
            ),
        )
        .expect_err("the state assertion cannot bind without the request identifier");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::ProfileStateBindingMismatch
    );

    let duplicate_target = request_message_with_operation_payload(
        ADD_ATTRIBUTE,
        structure([
            tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
            original_creation_date(),
            original_creation_date(),
        ]),
    );
    let malformed = selected
        .validate_request_with_context(&duplicate_target, &crate::ProfileValidationContext::new())
        .expect_err("duplicate target wrappers are ambiguous and fail closed");
    assert_eq!(
        malformed.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );

    let unrecorded_presence = request_message_with_operation_payload(
        ADD_ATTRIBUTE,
        structure([
            tagged(UNIQUE_IDENTIFIER, Value::text_string(OBJECT_ID.to_owned())),
            original_creation_date(),
        ]),
    );
    let unknown_state = crate::ProfileValidationContext::new()
        .with_object_state(0, crate::ProfileObjectState::new(OBJECT_ID));
    let error = super::validate_original_creation_date_context(
        &unrecorded_presence,
        &unknown_state,
        "KMIPKIT-PROFILE-TEST",
    )
    .expect_err("an unknown creation-date fact cannot authorize an addition");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::ProfileStateContextRequired
    );
}

#[test]
// OASIS KMIP Specification v2.1 §§4.40–4.41.
fn profile_maps_process_start_and_protect_stop_date_rules_as_caller_owned() {
    const TEST_ID: &str = "crates/kmipkit-protocol/tests/support/profile_coverage_unit.rs::profile_maps_process_start_and_protect_stop_date_rules_as_caller_owned";

    assert_static_caller_owned_profile_rule_mapping(
        "KMIPKIT-REQ-SPEC-4.40-001-001",
        "KMIPKIT-CLAUSE-SPEC-4.40-001",
        GeneratedVerificationCategory::OptionalCapability,
        "process_start_date_use_after_date_caller_asserted",
        TEST_ID,
        RuleTestKind::OptionalCapabilityUsable,
    );
    assert_static_caller_owned_profile_rule_mapping(
        "KMIPKIT-REQ-SPEC-4.40-001-002",
        "KMIPKIT-CLAUSE-SPEC-4.40-001",
        GeneratedVerificationCategory::ProhibitedBehavior,
        "no_cryptographic_use_before_process_start_date_caller_asserted",
        TEST_ID,
        RuleTestKind::NegativeBehavior,
    );
    assert_static_caller_owned_profile_rule_mapping(
        "KMIPKIT-REQ-SPEC-4.40-001-003",
        "KMIPKIT-CLAUSE-SPEC-4.40-001",
        GeneratedVerificationCategory::OptionalCapability,
        "process_start_date_may_equal_or_follow_activation_date_caller_asserted",
        TEST_ID,
        RuleTestKind::OptionalCapabilityUsable,
    );
    assert_static_caller_owned_profile_rule_mapping(
        "KMIPKIT-REQ-SPEC-4.40-001-004",
        "KMIPKIT-CLAUSE-SPEC-4.40-001",
        GeneratedVerificationCategory::ProhibitedBehavior,
        "process_start_date_must_not_precede_activation_date_caller_asserted",
        TEST_ID,
        RuleTestKind::NegativeBehavior,
    );
    assert_static_caller_owned_profile_rule_mapping(
        "KMIPKIT-REQ-SPEC-4.40-001-005",
        "KMIPKIT-CLAUSE-SPEC-4.40-001",
        GeneratedVerificationCategory::ProhibitedBehavior,
        "process_start_date_immutable_after_occurrence_caller_asserted",
        TEST_ID,
        RuleTestKind::NegativeBehavior,
    );
    assert_static_caller_owned_profile_rule_mapping(
        "KMIPKIT-REQ-SPEC-4.41-001-001",
        "KMIPKIT-CLAUSE-SPEC-4.41-001",
        GeneratedVerificationCategory::ProhibitedBehavior,
        "no_cryptographic_protection_after_protect_stop_date_caller_asserted",
        TEST_ID,
        RuleTestKind::NegativeBehavior,
    );
    assert_static_caller_owned_profile_rule_mapping(
        "KMIPKIT-REQ-SPEC-4.41-001-002",
        "KMIPKIT-CLAUSE-SPEC-4.41-001",
        GeneratedVerificationCategory::OptionalCapability,
        "protect_stop_date_may_equal_or_precede_deactivation_date_caller_asserted",
        TEST_ID,
        RuleTestKind::OptionalCapabilityUsable,
    );
    assert_static_caller_owned_profile_rule_mapping(
        "KMIPKIT-REQ-SPEC-4.41-001-003",
        "KMIPKIT-CLAUSE-SPEC-4.41-001",
        GeneratedVerificationCategory::ProhibitedBehavior,
        "protect_stop_date_must_not_follow_deactivation_date_caller_asserted",
        TEST_ID,
        RuleTestKind::NegativeBehavior,
    );
    assert_static_caller_owned_profile_rule_mapping(
        "KMIPKIT-REQ-SPEC-4.41-001-004",
        "KMIPKIT-CLAUSE-SPEC-4.41-001",
        GeneratedVerificationCategory::ProhibitedBehavior,
        "protect_stop_date_immutable_after_occurrence_caller_asserted",
        TEST_ID,
        RuleTestKind::NegativeBehavior,
    );
}

#[test]
// OASIS KMIP Specification v2.1 §4.57 paragraph 2.
fn server_only_pre_active_use_prohibition_is_not_a_client_rule() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.57-002";

    assert!(
        CLIENT_REQUIREMENTS
            .iter()
            .all(|requirement| requirement.requirement_id != REQUIREMENT_ID),
        "server-owned Pre-Active use prohibition must not enter the client registry"
    );
    assert!(
        TYPED_RULE_MAPPINGS
            .iter()
            .all(|mapping| mapping.requirement_id != REQUIREMENT_ID),
        "server-owned Pre-Active use prohibition must not become a client runtime rule"
    );
}

#[test]
fn simple_typed_rule_mapping_defaults_gap_and_evaluator_to_absent() {
    const TEST_EVIDENCE: &[RuleTestEvidence] = &[RuleTestEvidence {
        test_id: "profile_unit::simple_typed_rule_mapping_defaults_gap_and_evaluator_to_absent",
        kind: RuleTestKind::OptionalCapabilityUsable,
    }];
    const CONSTRAINTS: &[RuleConstraint] = &[RuleConstraint {
        key: "caller_assertion",
        value: "explicit",
    }];

    let mapping = super::super::typed_rule_mapping_without_evaluator(
        "KMIPKIT-REQ-SPEC-TEST-001",
        "KMIPKIT-CLAUSE-SPEC-TEST-001",
        super::GeneratedVerificationCategory::OptionalCapability,
        TEST_EVIDENCE,
        CONSTRAINTS,
    );

    assert_eq!(mapping.requirement_id, "KMIPKIT-REQ-SPEC-TEST-001");
    assert_eq!(mapping.source_clause_id, "KMIPKIT-CLAUSE-SPEC-TEST-001");
    assert_eq!(mapping.source_gap_id, None);
    assert_eq!(
        mapping.verification_category,
        super::GeneratedVerificationCategory::OptionalCapability
    );
    assert!(mapping.evaluator.is_none());
    assert_eq!(mapping.test_evidence, TEST_EVIDENCE);
    assert_eq!(mapping.constraints, CONSTRAINTS);
}
