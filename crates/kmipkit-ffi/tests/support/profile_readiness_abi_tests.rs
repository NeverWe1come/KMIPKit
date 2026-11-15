use super::*;
use crate::{
    kmipkit_byte_span_v1_t, kmipkit_profile_evidence_record_v1_t,
    kmipkit_profile_readiness_blocker_view_v1_t, kmipkit_profile_readiness_t,
    kmipkit_profile_readiness_view_v1_t,
};
use kmipkit_protocol::ProfileReadinessBlocker;
use std::mem::size_of;
use std::ptr;

fn span_text(span: kmipkit_byte_span_v1_t) -> Option<String> {
    if span.length == 0 {
        return Some(String::new());
    }
    let Ok(length) = usize::try_from(span.length) else {
        return None;
    };
    if span.data.is_null() {
        return None;
    }
    // SAFETY: Callers invoke this only on views while the readiness owner is live, and the FFI
    // contract guarantees each returned span points to initialized UTF-8 bytes of `length`.
    let bytes = unsafe { std::slice::from_raw_parts(span.data, length) };
    std::str::from_utf8(bytes).ok().map(str::to_owned)
}

fn abi_size<T>() -> u32 {
    u32::try_from(size_of::<T>()).expect("test ABI layouts fit in the fixed-width size field")
}

fn optional_span_text(span: kmipkit_byte_span_v1_t, present: u8) -> Option<String> {
    (present != 0).then(|| span_text(span)).flatten()
}

fn initialized_blocker_view() -> kmipkit_profile_readiness_blocker_view_v1_t {
    kmipkit_profile_readiness_blocker_view_v1_t {
        struct_size: abi_size::<kmipkit_profile_readiness_blocker_view_v1_t>(),
        abi_version: 1,
        ..Default::default()
    }
}

fn assert_blocker_view(
    blocker: ProfileReadinessBlocker,
    expected_kind: &str,
    requirement_id: Option<&str>,
    profile_id: Option<&str>,
    test_id: Option<&str>,
    source_clause_id: Option<&str>,
) {
    let mut view = initialized_blocker_view();
    populate_profile_readiness_blocker_view(blocker, &mut view);
    assert_eq!(span_text(view.kind), Some(expected_kind.to_owned()));
    assert_eq!(span_text(view.message), Some(blocker.as_str().to_owned()));
    assert_eq!(view.requirement_id_present != 0, requirement_id.is_some());
    assert_eq!(view.profile_id_present != 0, profile_id.is_some());
    assert_eq!(view.test_id_present != 0, test_id.is_some());
    assert_eq!(
        view.source_clause_id_present != 0,
        source_clause_id.is_some()
    );
    assert_eq!(
        requirement_id.map(str::to_owned),
        (view.requirement_id_present != 0).then(|| span_text(view.requirement_id).unwrap())
    );
    assert_eq!(
        profile_id.map(str::to_owned),
        (view.profile_id_present != 0).then(|| span_text(view.profile_id).unwrap())
    );
    assert_eq!(
        test_id.map(str::to_owned),
        (view.test_id_present != 0).then(|| span_text(view.test_id).unwrap())
    );
    assert_eq!(
        source_clause_id.map(str::to_owned),
        (view.source_clause_id_present != 0).then(|| span_text(view.source_clause_id).unwrap())
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Keeps the full closed blocker-kind mapping beside its assertions.
fn readiness_blocker_view_preserves_all_thirteen_kinds_and_optional_ids() {
    assert_blocker_view(
        ProfileReadinessBlocker::MissingRequirementRecord {
            requirement_id: "req-1",
        },
        "missing_requirement_record",
        Some("req-1"),
        None,
        None,
        None,
    );
    assert_blocker_view(
        ProfileReadinessBlocker::FailedRequirementRecord {
            requirement_id: "req-2",
        },
        "failed_requirement_record",
        Some("req-2"),
        None,
        None,
        None,
    );
    assert_blocker_view(
        ProfileReadinessBlocker::UnresolvedRequirementScope {
            requirement_id: "req-3",
            source_clause_id: "clause-3",
        },
        "unresolved_requirement_scope",
        Some("req-3"),
        None,
        None,
        Some("clause-3"),
    );
    assert_blocker_view(
        ProfileReadinessBlocker::DuplicateRequirementRecord {
            requirement_id: "req-4",
        },
        "duplicate_requirement_record",
        Some("req-4"),
        None,
        None,
        None,
    );
    assert_blocker_view(
        ProfileReadinessBlocker::UnknownRequirementRecord,
        "unknown_requirement_record",
        None,
        None,
        None,
        None,
    );
    assert_blocker_view(
        ProfileReadinessBlocker::RuntimeValidationUnavailable,
        "runtime_validation_unavailable",
        None,
        None,
        None,
        None,
    );
    assert_blocker_view(
        ProfileReadinessBlocker::MandatoryFixtureUnavailable { test_id: "test-1" },
        "mandatory_fixture_unavailable",
        None,
        None,
        Some("test-1"),
        None,
    );
    assert_blocker_view(
        ProfileReadinessBlocker::ConditionalProfileUnsatisfied,
        "conditional_profile_unsatisfied",
        None,
        None,
        None,
        None,
    );
    assert_blocker_view(
        ProfileReadinessBlocker::DependencyEvidenceIncomplete {
            profile_id: "profile-1",
        },
        "dependency_evidence_incomplete",
        None,
        Some("profile-1"),
        None,
        None,
    );
    assert_blocker_view(
        ProfileReadinessBlocker::VerificationManifestUnavailable,
        "verification_manifest_unavailable",
        None,
        None,
        None,
        None,
    );
    assert_blocker_view(
        ProfileReadinessBlocker::StaleCatalogEvidenceState,
        "stale_catalog_evidence_state",
        None,
        None,
        None,
        None,
    );
    assert_blocker_view(
        ProfileReadinessBlocker::CatalogEvidenceReviewRequired,
        "catalog_evidence_review_required",
        None,
        None,
        None,
        None,
    );
    assert_blocker_view(
        ProfileReadinessBlocker::ProfileUnavailable,
        "profile_unavailable",
        None,
        None,
        None,
        None,
    );
}

#[test]
fn readiness_ffi_preflights_versions_limits_and_invalid_flags() {
    let profile_id = b"KMIPKIT-PROFILE-BASELINE-CLIENT";
    let profile_span = kmipkit_byte_span_v1_t {
        data: profile_id.as_ptr(),
        length: profile_id.len() as u64,
    };
    let mut out = ptr::null_mut::<kmipkit_profile_readiness_t>();
    let too_many = test_pointer_ffi_call!(kmipkit_profile_readiness_evaluate_v1(
        &raw const profile_span,
        ptr::NonNull::<kmipkit_profile_evidence_record_v1_t>::dangling().as_ptr(),
        100_001,
        &raw mut out,
    ));
    assert_eq!(too_many, ERROR_RESOURCE_LIMIT);
    assert!(out.is_null());

    let mut oversized = kmipkit_profile_evidence_record_v1_t {
        struct_size: abi_size::<kmipkit_profile_evidence_record_v1_t>(),
        abi_version: 1,
        requirement_id: kmipkit_byte_span_v1_t {
            data: ptr::NonNull::<u8>::dangling().as_ptr(),
            length: 16 * 1024 * 1024 + 1,
        },
        passed: 1,
        reserved_0: 0,
        reserved_1: 0,
        reserved_2: 0,
        reserved_3: 0,
        reserved_4: 0,
        reserved_5: 0,
        reserved_6: 0,
    };
    let limited = test_pointer_ffi_call!(kmipkit_profile_readiness_evaluate_v1(
        &raw const profile_span,
        &raw const oversized,
        1,
        &raw mut out,
    ));
    assert_eq!(limited, ERROR_RESOURCE_LIMIT);
    assert!(out.is_null());

    let long_requirement_id = vec![b'x'; 8 * 1024];
    oversized.requirement_id.data = long_requirement_id.as_ptr();
    oversized.requirement_id.length = long_requirement_id.len() as u64;
    let accepted_evidence_id = test_pointer_ffi_call!(kmipkit_profile_readiness_evaluate_v1(
        &raw const profile_span,
        &raw const oversized,
        1,
        &raw mut out,
    ));
    assert_eq!(accepted_evidence_id, SUCCESS);
    assert!(!out.is_null());
    kmipkit_profile_readiness_release(out);
    out = ptr::null_mut();

    oversized.requirement_id.length = 0;
    oversized.passed = 2;
    let invalid_flag = test_pointer_ffi_call!(kmipkit_profile_readiness_evaluate_v1(
        &raw const profile_span,
        &raw const oversized,
        1,
        &raw mut out,
    ));
    assert_eq!(invalid_flag, ERROR_INVALID_INPUT);
    assert!(out.is_null());

    oversized.passed = 1;
    oversized.abi_version = 2;
    let invalid_version = test_pointer_ffi_call!(kmipkit_profile_readiness_evaluate_v1(
        &raw const profile_span,
        &raw const oversized,
        1,
        &raw mut out,
    ));
    assert_eq!(invalid_version, ERROR_INVALID_INPUT);
    assert!(out.is_null());
}

#[test]
fn readiness_views_borrow_from_the_live_handle_and_baseline_stays_unclaimed() {
    let profile_id = b"KMIPKIT-PROFILE-BASELINE-CLIENT";
    let profile_span = kmipkit_byte_span_v1_t {
        data: profile_id.as_ptr(),
        length: profile_id.len() as u64,
    };
    let mut readiness = ptr::null_mut::<kmipkit_profile_readiness_t>();
    let status = test_pointer_ffi_call!(kmipkit_profile_readiness_evaluate_v1(
        &raw const profile_span,
        ptr::null(),
        0,
        &raw mut readiness,
    ));
    assert_eq!(status, 0);
    assert!(!readiness.is_null());

    let mut view = kmipkit_profile_readiness_view_v1_t {
        struct_size: abi_size::<kmipkit_profile_readiness_view_v1_t>(),
        abi_version: 1,
        ..Default::default()
    };
    let status = test_pointer_ffi_call!(kmipkit_profile_readiness_view_get_v1(
        readiness,
        &raw mut view
    ));
    assert_eq!(status, 0);
    assert_eq!(
        span_text(view.profile_id).as_deref(),
        Some("KMIPKIT-PROFILE-BASELINE-CLIENT")
    );
    assert_eq!(
        span_text(view.effective_state).as_deref(),
        Some("evidence_incomplete")
    );
    assert_eq!(view.published_claim, 0);
    assert!(view.target_selected != 0);
    assert!(!view.profile_id.data.is_null());
    kmipkit_profile_readiness_release(readiness);
}

fn assert_catalog_strings(table: u32, row_index: u64, relation: u32, expected: &[&str]) {
    let mut count = u64::MAX;
    assert_eq!(
        unsafe {
            kmipkit_profile_catalog_relation_count_v1(table, row_index, relation, &raw mut count)
        },
        0
    );
    assert_eq!(
        usize::try_from(count).expect("count fits usize"),
        expected.len()
    );
    for (index, expected_value) in expected.iter().enumerate() {
        let mut row = crate::kmipkit_profile_catalog_row_view_v1_t {
            struct_size: abi_size::<crate::kmipkit_profile_catalog_row_view_v1_t>(),
            abi_version: 1,
            ..Default::default()
        };
        assert_eq!(
            unsafe {
                kmipkit_profile_catalog_relation_at_v1(
                    table,
                    row_index,
                    relation,
                    index as u64,
                    &raw mut row,
                )
            },
            0
        );
        assert_eq!(span_text(row.text_value).as_deref(), Some(*expected_value));
    }
}

fn expected_category(
    value: kmipkit_protocol::profiles_generated::GeneratedVerificationCategory,
) -> &'static str {
    use kmipkit_protocol::profiles_generated::GeneratedVerificationCategory as Category;
    match value {
        Category::RequiredBehavior => "required_behavior",
        Category::ProhibitedBehavior => "prohibited_behavior",
        Category::RecommendedBehaviorOrDeviation => "recommended_behavior_or_deviation",
        Category::DiscouragedAbsenceOrDeviation => "discouraged_absence_or_deviation",
        Category::OptionalCapability => "optional_capability",
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Verifies every generated row and ordered relation through the ABI.
fn catalog_ffi_matches_every_generated_rust_row_and_ordered_relation() {
    use kmipkit_protocol::profiles_generated::{
        CLIENT_PROFILES, CLIENT_REQUIREMENTS, PROFILE_TEST_CASES,
    };
    const PROFILE_TABLE: u32 = KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES;
    const REQUIREMENT_TABLE: u32 = KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_REQUIREMENTS;
    const TEST_CASE_TABLE: u32 = KMIPKIT_PROFILE_CATALOG_TABLE_PROFILE_TEST_CASES;

    for (table, expected_count) in [
        (PROFILE_TABLE, CLIENT_PROFILES.len()),
        (REQUIREMENT_TABLE, CLIENT_REQUIREMENTS.len()),
        (TEST_CASE_TABLE, PROFILE_TEST_CASES.len()),
    ] {
        let mut actual_count = 0;
        assert_eq!(
            unsafe { kmipkit_profile_catalog_table_count_v1(table, &raw mut actual_count) },
            0
        );
        assert_eq!(
            usize::try_from(actual_count).expect("count fits usize"),
            expected_count
        );
    }

    for (index, expected) in CLIENT_PROFILES.iter().enumerate() {
        let mut row = crate::kmipkit_profile_catalog_row_view_v1_t {
            struct_size: abi_size::<crate::kmipkit_profile_catalog_row_view_v1_t>(),
            abi_version: 1,
            ..Default::default()
        };
        assert_eq!(
            unsafe { kmipkit_profile_catalog_row_at_v1(PROFILE_TABLE, index as u64, &raw mut row) },
            0
        );
        assert_eq!(
            span_text(row.profile_id).as_deref(),
            Some(expected.profile_id)
        );
        assert_eq!(span_text(row.name).as_deref(), Some(expected.name));
        assert_eq!(
            span_text(row.applicability).as_deref(),
            Some(expected.applicability)
        );
        assert_eq!(
            span_text(row.claim_state).as_deref(),
            Some(expected.claim_state)
        );
        assert_eq!(row.selected_target != 0, expected.selected_target);
        assert_catalog_strings(
            PROFILE_TABLE,
            index as u64,
            KMIPKIT_PROFILE_CATALOG_RELATION_DIRECT_SOURCE_CLAUSE_IDS,
            expected.direct_source_clause_ids,
        );
        assert_catalog_strings(
            PROFILE_TABLE,
            index as u64,
            KMIPKIT_PROFILE_CATALOG_RELATION_SOURCE_CLAUSE_IDS,
            expected.source_clause_ids,
        );
        assert_catalog_strings(
            PROFILE_TABLE,
            index as u64,
            KMIPKIT_PROFILE_CATALOG_RELATION_REQUIREMENT_IDS,
            expected.requirement_ids,
        );
        assert_catalog_strings(
            PROFILE_TABLE,
            index as u64,
            KMIPKIT_PROFILE_CATALOG_RELATION_DEPENDENCY_PROFILE_IDS,
            expected.dependency_profile_ids,
        );
        assert_catalog_strings(
            PROFILE_TABLE,
            index as u64,
            KMIPKIT_PROFILE_CATALOG_RELATION_TRANSPORT_REQUIREMENTS,
            expected.transport_requirements,
        );
        assert_catalog_strings(
            PROFILE_TABLE,
            index as u64,
            KMIPKIT_PROFILE_CATALOG_RELATION_ENCODING_REQUIREMENTS,
            expected.encoding_requirements,
        );
        let mut count = 0;
        assert_eq!(
            unsafe {
                kmipkit_profile_catalog_relation_count_v1(
                    PROFILE_TABLE,
                    index as u64,
                    KMIPKIT_PROFILE_CATALOG_RELATION_UNRESOLVED_SCOPE_OBLIGATIONS,
                    &raw mut count,
                )
            },
            0
        );
        assert_eq!(
            usize::try_from(count).expect("count fits usize"),
            expected.unresolved_scope_obligations.len()
        );
        for (nested_index, blocker) in expected.unresolved_scope_obligations.iter().enumerate() {
            let mut nested = crate::kmipkit_profile_catalog_row_view_v1_t {
                struct_size: abi_size::<crate::kmipkit_profile_catalog_row_view_v1_t>(),
                abi_version: 1,
                ..Default::default()
            };
            assert_eq!(
                unsafe {
                    kmipkit_profile_catalog_relation_at_v1(
                        PROFILE_TABLE,
                        index as u64,
                        KMIPKIT_PROFILE_CATALOG_RELATION_UNRESOLVED_SCOPE_OBLIGATIONS,
                        nested_index as u64,
                        &raw mut nested,
                    )
                },
                0
            );
            assert_eq!(
                span_text(nested.requirement_id).as_deref(),
                Some(blocker.requirement_id)
            );
            assert_eq!(
                span_text(nested.source_clause_id).as_deref(),
                Some(blocker.source_clause_id)
            );
            assert_eq!(
                span_text(nested.scope_state).as_deref(),
                Some(blocker.scope_state)
            );
        }
        let mut tests_count = 0;
        assert_eq!(
            unsafe {
                kmipkit_profile_catalog_relation_count_v1(
                    PROFILE_TABLE,
                    index as u64,
                    KMIPKIT_PROFILE_CATALOG_RELATION_TEST_CASES,
                    &raw mut tests_count,
                )
            },
            0
        );
        assert_eq!(
            usize::try_from(tests_count).expect("count fits usize"),
            expected.test_cases.len()
        );
        for (nested_index, test) in expected.test_cases.iter().enumerate() {
            let mut nested = crate::kmipkit_profile_catalog_row_view_v1_t {
                struct_size: abi_size::<crate::kmipkit_profile_catalog_row_view_v1_t>(),
                abi_version: 1,
                ..Default::default()
            };
            assert_eq!(
                unsafe {
                    kmipkit_profile_catalog_relation_at_v1(
                        PROFILE_TABLE,
                        index as u64,
                        KMIPKIT_PROFILE_CATALOG_RELATION_TEST_CASES,
                        nested_index as u64,
                        &raw mut nested,
                    )
                },
                0
            );
            assert_eq!(span_text(nested.test_id).as_deref(), Some(test.test_id));
            assert_eq!(
                span_text(nested.official_case_id).as_deref(),
                Some(test.official_case_id)
            );
            assert_eq!(
                span_text(nested.mandatory_status).as_deref(),
                Some(test.mandatory_status)
            );
            assert_eq!(
                span_text(nested.fixture_availability).as_deref(),
                Some(test.fixture_availability)
            );
        }
    }

    for (index, expected) in CLIENT_REQUIREMENTS.iter().enumerate() {
        let mut row = crate::kmipkit_profile_catalog_row_view_v1_t {
            struct_size: abi_size::<crate::kmipkit_profile_catalog_row_view_v1_t>(),
            abi_version: 1,
            ..Default::default()
        };
        assert_eq!(
            unsafe {
                kmipkit_profile_catalog_row_at_v1(REQUIREMENT_TABLE, index as u64, &raw mut row)
            },
            0
        );
        assert_eq!(
            span_text(row.requirement_id).as_deref(),
            Some(expected.requirement_id)
        );
        assert_eq!(
            row.source_keyword_present != 0,
            expected.source_keyword.is_some()
        );
        assert_eq!(
            optional_span_text(row.source_keyword, row.source_keyword_present).as_deref(),
            expected.source_keyword
        );
        assert_eq!(
            row.source_force_present != 0,
            expected.source_force.is_some()
        );
        assert_eq!(
            optional_span_text(row.source_force, row.source_force_present).as_deref(),
            expected.source_force
        );
        assert_eq!(
            row.source_gap_id_present != 0,
            expected.source_gap_id.is_some()
        );
        assert_eq!(
            optional_span_text(row.source_gap_id, row.source_gap_id_present).as_deref(),
            expected.source_gap_id
        );
        assert_eq!(
            row.source_locator_present != 0,
            expected.source_locator.is_some()
        );
        assert_eq!(
            optional_span_text(row.source_locator, row.source_locator_present).as_deref(),
            expected.source_locator
        );
        assert_eq!(
            span_text(row.verification_category).as_deref(),
            Some(expected_category(expected.verification_category))
        );
        assert_eq!(
            span_text(row.scope_state).as_deref(),
            Some(expected.scope_state)
        );
        assert_eq!(
            row.has_accepted_deviation != 0,
            expected.has_accepted_deviation
        );
        assert_catalog_strings(
            REQUIREMENT_TABLE,
            index as u64,
            KMIPKIT_PROFILE_CATALOG_RELATION_SOURCE_CLAUSE_IDS,
            expected.source_clause_ids,
        );
        assert_catalog_strings(
            REQUIREMENT_TABLE,
            index as u64,
            KMIPKIT_PROFILE_CATALOG_RELATION_PROFILE_IDS,
            expected.profile_ids,
        );
        assert_catalog_strings(
            REQUIREMENT_TABLE,
            index as u64,
            KMIPKIT_PROFILE_CATALOG_RELATION_OFFICIAL_TEST_CASE_IDS,
            expected.official_test_case_ids,
        );
    }

    for (index, expected) in PROFILE_TEST_CASES.iter().enumerate() {
        let mut row = crate::kmipkit_profile_catalog_row_view_v1_t {
            struct_size: abi_size::<crate::kmipkit_profile_catalog_row_view_v1_t>(),
            abi_version: 1,
            ..Default::default()
        };
        assert_eq!(
            unsafe {
                kmipkit_profile_catalog_row_at_v1(TEST_CASE_TABLE, index as u64, &raw mut row)
            },
            0
        );
        assert_eq!(span_text(row.test_id).as_deref(), Some(expected.test_id));
        assert_eq!(
            span_text(row.official_case_id).as_deref(),
            Some(expected.official_case_id)
        );
        assert_eq!(
            span_text(row.mandatory_status).as_deref(),
            Some(expected.mandatory_status)
        );
        assert_eq!(
            span_text(row.fixture_availability).as_deref(),
            Some(expected.fixture_availability)
        );
    }
}

fn profile_byte_span(bytes: &[u8]) -> kmipkit_byte_span_v1_t {
    kmipkit_byte_span_v1_t {
        data: bytes.as_ptr(),
        length: u64::try_from(bytes.len()).expect("test span length fits the ABI"),
    }
}

fn assert_readiness_evaluate_status(
    profile_id: *const kmipkit_byte_span_v1_t,
    records: *const kmipkit_profile_evidence_record_v1_t,
    record_count: u64,
    expected_status: i32,
) {
    let mut readiness = ptr::dangling_mut();
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_readiness_evaluate_v1(
            profile_id,
            records,
            record_count,
            &raw mut readiness,
        )),
        expected_status
    );
    assert!(readiness.is_null());
}

fn evidence_record(requirement_id: kmipkit_byte_span_v1_t) -> kmipkit_profile_evidence_record_v1_t {
    kmipkit_profile_evidence_record_v1_t {
        struct_size: abi_size::<kmipkit_profile_evidence_record_v1_t>(),
        abi_version: 1,
        requirement_id,
        passed: 1,
        reserved_0: 0,
        reserved_1: 0,
        reserved_2: 0,
        reserved_3: 0,
        reserved_4: 0,
        reserved_5: 0,
        reserved_6: 0,
    }
}

fn set_evidence_reserved(
    record: &mut kmipkit_profile_evidence_record_v1_t,
    index: usize,
    value: u8,
) {
    match index {
        0 => record.reserved_0 = value,
        1 => record.reserved_1 = value,
        2 => record.reserved_2 = value,
        3 => record.reserved_3 = value,
        4 => record.reserved_4 = value,
        5 => record.reserved_5 = value,
        6 => record.reserved_6 = value,
        _ => unreachable!("reserved field index is bounded by the test"),
    }
}

#[test]
fn readiness_evaluate_rejects_invalid_spans_and_reserved_evidence_fields() {
    let profile_id = b"KMIPKIT-PROFILE-BASELINE-CLIENT";
    let profile_span = profile_byte_span(profile_id);
    assert_readiness_evaluate_status(ptr::null(), ptr::null(), 0, ERROR_INVALID_INPUT);

    let invalid_utf8 = [0xff];
    let invalid_profile_span = profile_byte_span(&invalid_utf8);
    assert_readiness_evaluate_status(
        &raw const invalid_profile_span,
        ptr::null(),
        0,
        ERROR_INVALID_INPUT,
    );
    assert_readiness_evaluate_status(&raw const profile_span, ptr::null(), 1, ERROR_INVALID_INPUT);

    let unknown_profile_span = profile_byte_span(b"vendor.profile.unknown");
    assert_readiness_evaluate_status(&raw const unknown_profile_span, ptr::null(), 0, SUCCESS);

    let mut record = evidence_record(profile_byte_span(b"KMIP-REQ-EXAMPLE"));
    for reserved_index in 0..7 {
        set_evidence_reserved(&mut record, reserved_index, 1);
        assert_readiness_evaluate_status(
            &raw const profile_span,
            &raw const record,
            1,
            ERROR_INVALID_INPUT,
        );
        set_evidence_reserved(&mut record, reserved_index, 0);
    }

    let invalid_evidence = [0xff];
    record.requirement_id = profile_byte_span(&invalid_evidence);
    assert_readiness_evaluate_status(
        &raw const profile_span,
        &raw const record,
        1,
        ERROR_INVALID_INPUT,
    );
}

fn assert_readiness_view_rejects_invalid_headers(view: &mut kmipkit_profile_readiness_view_v1_t) {
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_readiness_view_get_v1(
            ptr::null_mut(),
            ptr::null_mut(),
        )),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_readiness_view_get_v1(
            ptr::null_mut(),
            &raw mut *view,
        )),
        ERROR_INVALID_INPUT
    );
    view.abi_version = 2;
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_readiness_view_get_v1(
            ptr::null_mut(),
            &raw mut *view,
        )),
        ERROR_INVALID_INPUT
    );
}

fn assert_blocker_view_rejects_invalid_headers(
    view: &mut kmipkit_profile_readiness_blocker_view_v1_t,
) {
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_readiness_blocker_at_v1(
            ptr::null_mut(),
            0,
            ptr::null_mut(),
        )),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_readiness_blocker_at_v1(
            ptr::null_mut(),
            0,
            &raw mut *view,
        )),
        ERROR_INVALID_INPUT
    );
    view.abi_version = 2;
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_readiness_blocker_at_v1(
            ptr::null_mut(),
            0,
            &raw mut *view,
        )),
        ERROR_INVALID_INPUT
    );
}

#[test]
fn readiness_accessors_reject_invalid_headers_handles_and_blocker_indexes() {
    let mut readiness_view = kmipkit_profile_readiness_view_v1_t {
        struct_size: abi_size::<kmipkit_profile_readiness_view_v1_t>(),
        abi_version: 1,
        ..Default::default()
    };
    assert_readiness_view_rejects_invalid_headers(&mut readiness_view);
    let mut blocker_view = initialized_blocker_view();
    assert_blocker_view_rejects_invalid_headers(&mut blocker_view);

    let profile_span = profile_byte_span(b"KMIPKIT-PROFILE-BASELINE-CLIENT");
    let mut readiness = ptr::null_mut();
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_readiness_evaluate_v1(
            &raw const profile_span,
            ptr::null(),
            0,
            &raw mut readiness,
        )),
        SUCCESS
    );
    assert!(!readiness.is_null());
    readiness_view.abi_version = 1;
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_readiness_view_get_v1(
            readiness,
            &raw mut readiness_view,
        )),
        SUCCESS
    );
    assert!(readiness_view.blocker_count > 0);

    blocker_view.abi_version = 1;
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_readiness_blocker_at_v1(
            readiness,
            0,
            &raw mut blocker_view,
        )),
        SUCCESS
    );
    assert_eq!(
        span_text(blocker_view.kind).as_deref(),
        Some("missing_requirement_record")
    );
    for invalid_index in [readiness_view.blocker_count, u64::MAX] {
        assert_eq!(
            test_pointer_ffi_call!(kmipkit_profile_readiness_blocker_at_v1(
                readiness,
                invalid_index,
                &raw mut blocker_view,
            )),
            ERROR_INVALID_INPUT
        );
    }
    kmipkit_profile_readiness_release(readiness);
}

fn assert_profile_catalog_count_errors(count: &mut u64) {
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_catalog_table_count_v1(
            u32::MAX,
            &raw mut *count,
        )),
        ERROR_INVALID_INPUT
    );
    assert_eq!(*count, 0);
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_catalog_table_count_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES,
            ptr::null_mut(),
        )),
        ERROR_INVALID_INPUT
    );
}

fn assert_profile_catalog_row_errors(row: &mut crate::kmipkit_profile_catalog_row_view_v1_t) {
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_catalog_row_at_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES,
            0,
            ptr::null_mut(),
        )),
        ERROR_INVALID_INPUT
    );
    for (struct_size, abi_version) in [(row.struct_size - 1, 1), (row.struct_size, 2)] {
        row.struct_size = struct_size;
        row.abi_version = abi_version;
        assert_eq!(
            test_pointer_ffi_call!(kmipkit_profile_catalog_row_at_v1(
                KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES,
                0,
                &raw mut *row,
            )),
            ERROR_INVALID_INPUT
        );
    }
    row.struct_size = abi_size::<crate::kmipkit_profile_catalog_row_view_v1_t>();
    row.abi_version = 1;
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_catalog_row_at_v1(
            u32::MAX,
            0,
            &raw mut *row,
        )),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_catalog_row_at_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES,
            u64::MAX,
            &raw mut *row,
        )),
        ERROR_INVALID_INPUT
    );
}

fn assert_profile_catalog_relation_count_errors(count: &mut u64) {
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_catalog_relation_count_v1(
            u32::MAX,
            0,
            0,
            &raw mut *count,
        )),
        ERROR_INVALID_INPUT
    );
    assert_eq!(*count, 0);
    for (table, row_index, relation) in [
        (KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, u64::MAX, 1),
        (KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, 0, u32::MAX),
        (KMIPKIT_PROFILE_CATALOG_TABLE_PROFILE_TEST_CASES, 0, 1),
    ] {
        assert_eq!(
            test_pointer_ffi_call!(kmipkit_profile_catalog_relation_count_v1(
                table,
                row_index,
                relation,
                &raw mut *count,
            )),
            ERROR_INVALID_INPUT
        );
        assert_eq!(*count, 0);
    }
}

fn assert_profile_catalog_relation_at_boundaries(
    row: &mut crate::kmipkit_profile_catalog_row_view_v1_t,
) {
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_catalog_relation_at_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES,
            0,
            KMIPKIT_PROFILE_CATALOG_RELATION_SOURCE_CLAUSE_IDS,
            0,
            ptr::null_mut(),
        )),
        ERROR_INVALID_INPUT
    );
    let mut relation_count = 0;
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_catalog_relation_count_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES,
            0,
            KMIPKIT_PROFILE_CATALOG_RELATION_SOURCE_CLAUSE_IDS,
            &raw mut relation_count,
        )),
        SUCCESS
    );
    assert!(relation_count > 0);
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_catalog_relation_at_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES,
            0,
            KMIPKIT_PROFILE_CATALOG_RELATION_SOURCE_CLAUSE_IDS,
            0,
            &raw mut *row,
        )),
        SUCCESS
    );
    assert_eq!(
        test_pointer_ffi_call!(kmipkit_profile_catalog_relation_at_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES,
            0,
            KMIPKIT_PROFILE_CATALOG_RELATION_SOURCE_CLAUSE_IDS,
            relation_count,
            &raw mut *row,
        )),
        ERROR_INVALID_INPUT
    );
}

#[test]
fn profile_catalog_ffi_clears_outputs_and_rejects_invalid_selectors() {
    let mut count = u64::MAX;
    assert_profile_catalog_count_errors(&mut count);
    let mut row = crate::kmipkit_profile_catalog_row_view_v1_t {
        struct_size: abi_size::<crate::kmipkit_profile_catalog_row_view_v1_t>(),
        abi_version: 1,
        ..Default::default()
    };
    assert_profile_catalog_row_errors(&mut row);
    count = u64::MAX;
    assert_profile_catalog_relation_count_errors(&mut count);
    assert_profile_catalog_relation_at_boundaries(&mut row);
}
