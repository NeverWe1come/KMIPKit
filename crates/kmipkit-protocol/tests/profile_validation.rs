//! Selection regressions trace Baseline Client's KMIP Profiles v2.1 §§3.1,
//! 3.1.1, 3.1.2, and 6.1 closure and the KMIP Specification v2.1 §14.1 client
//! profile prerequisite through the checked-in normative catalog. Normative
//! sources are the pinned OASIS `kmip-profiles-v2.1-os.html` and
//! `kmip-spec-v2.1-os.html` files; stable catalog IDs are asserted below.

use kmipkit_protocol::{
    ProfileValidationErrorCode, ProfileValidationSelection,
    profiles_generated::{CLIENT_PROFILES, CLIENT_REQUIREMENTS, GeneratedVerificationCategory},
};

const BASELINE: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
const AES_XTS: &str = "KMIPKIT-PROFILE-AES-XTS-CLIENT";
const HTTPS: &str = "KMIPKIT-PROFILE-HTTPS-CLIENT";
const JSON: &str = "KMIPKIT-PROFILE-JSON-CLIENT";
const BASELINE_SERVER: &str = "KMIPKIT-PROFILE-BASELINE-SERVER";

#[test]
fn baseline_target_selection_requires_category_appropriate_evidence() {
    let target = CLIENT_PROFILES
        .iter()
        .find(|profile| profile.profile_id == BASELINE)
        .expect("the generated catalog includes Baseline Client");
    assert!(target.selected_target);
    for attribute_requirement in [
        "KMIPKIT-REQ-SPEC-4.1-001-001",
        "KMIPKIT-REQ-SPEC-4.1-001-002",
        "KMIPKIT-REQ-SPEC-4.1-001-003",
        "KMIPKIT-REQ-SPEC-4.18-001-001",
        "KMIPKIT-REQ-SPEC-4.18-001-002",
        "KMIPKIT-REQ-PROF-6-001",
    ] {
        assert!(target.requirement_ids.contains(&attribute_requirement));
        assert!(
            !target
                .unresolved_scope_obligations
                .iter()
                .any(|obligation| obligation.requirement_id == attribute_requirement)
        );
    }
    assert!(target.requirement_ids.contains(&"KMIPKIT-REQ-SPEC-2.3-001"));

    // KMIP Profiles v2.1 §6.1 item 1 incorporates the entire KMIP
    // Specification. KMIP Specification v2.1 §14.1 supplies the client
    // profile prerequisite and complete-closure claim gate.
    for whole_spec_requirement in ["KMIPKIT-REQ-SPEC-14.1-001", "KMIPKIT-REQ-SPEC-14.1-002"] {
        assert!(target.requirement_ids.contains(&whole_spec_requirement));
    }
    for whole_spec_clause in [
        "KMIPKIT-CLAUSE-SPEC-14.1-001",
        "KMIPKIT-CLAUSE-SPEC-14.1-002",
    ] {
        assert!(target.source_clause_ids.contains(&whole_spec_clause));
    }

    // KMIP Profiles v2.1 §6.1 item 2 incorporates §3.1 and its subclauses.
    // Its TLS 1.2 MAY is kept traceable, but ADR-0005's TLS 1.3-only boundary
    // makes it out of scope for KMIPKit 1.0 and excludes it from effective
    // Baseline requirements.
    for inherited_requirement in [
        "KMIPKIT-REQ-PROF-3.1.1-002",
        "KMIPKIT-REQ-PROF-3.1.1-005",
        "KMIPKIT-REQ-PROF-3.1.2-002",
        "KMIPKIT-REQ-PROF-3.1.2-005",
    ] {
        assert!(target.requirement_ids.contains(&inherited_requirement));
    }
    assert!(
        target
            .source_clause_ids
            .contains(&"KMIPKIT-CLAUSE-PROF-3.1.1-004")
    );
    assert!(
        !target
            .requirement_ids
            .contains(&"KMIPKIT-REQ-PROF-3.1.1-004")
    );
    assert!(
        CLIENT_REQUIREMENTS
            .iter()
            .all(|requirement| requirement.requirement_id != "KMIPKIT-REQ-PROF-3.1.1-004")
    );

    let error = ProfileValidationSelection::new([BASELINE])
        .validate()
        .expect_err("caller-owned prohibitions need real negative behavior evidence");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::MissingCategoryTest
    );
    assert_eq!(error.requirement_id(), Some("KMIPKIT-REQ-SPEC-4.38-003"));
}

#[test]
fn baseline_selection_fails_when_category_test_evidence_is_incomplete() {
    let error = ProfileValidationSelection::new([BASELINE])
        .validate()
        .expect_err("category-incomplete rules keep Baseline unavailable");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::MissingCategoryTest
    );
    let target = CLIENT_PROFILES
        .iter()
        .find(|profile| profile.profile_id == BASELINE)
        .expect("the generated catalog includes Baseline Client");
    assert_eq!(target.unresolved_scope_obligations, []);
}

#[test]
fn profile_readiness_separates_runtime_availability_from_conformance_evidence() {
    let readiness = kmipkit_protocol::ProfileReadiness::evaluate(BASELINE, &[])
        .expect("Baseline Client is present in generated profile metadata");

    assert!(
        readiness
            .blockers()
            .iter()
            .any(|blocker| { blocker.as_str() == "runtime profile validation is unavailable" })
    );
    assert!(
        !readiness.blockers().iter().any(|blocker| {
            blocker.as_str() == "profile requirement applicability is unresolved"
        })
    );
    assert!(!readiness.runtime_validation_available());
    assert_eq!(readiness.effective_state(), "evidence_incomplete");
    assert!(!readiness.published_claim());
}

#[test]
fn in_scope_non_target_profile_inherits_baseline_evidence_blocker_without_target_gating() {
    let target = CLIENT_PROFILES
        .iter()
        .find(|profile| profile.profile_id == AES_XTS)
        .expect("the generated catalog includes AES-XTS Client");
    assert_eq!(target.applicability, "client_1_0");
    assert!(!target.selected_target);

    let error = ProfileValidationSelection::new([AES_XTS])
        .validate()
        .expect_err("AES-XTS has profile-specific requirements beyond Baseline");

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::MissingCategoryTest
    );
    assert_eq!(error.profile_id(), Some("KMIPKIT-PROFILE-BASELINE-CLIENT"));
    assert_eq!(error.requirement_id(), Some("KMIPKIT-REQ-SPEC-4.38-003"));
    assert!(error.source_clause_id().is_some());
}

#[test]
fn conditional_and_out_of_scope_profiles_fail_closed() {
    let conditional = ProfileValidationSelection::new([HTTPS])
        .validate()
        .expect_err("the HTTPS conditions have no typed satisfied configuration");
    assert_eq!(
        conditional.code(),
        ProfileValidationErrorCode::ConditionalProfileUnsatisfied
    );
    assert_eq!(conditional.profile_id(), Some(HTTPS));

    let out_of_scope = ProfileValidationSelection::new([JSON])
        .validate()
        .expect_err("JSON is outside the KMIPKit 1.0 product boundary");
    assert_eq!(
        out_of_scope.code(),
        ProfileValidationErrorCode::ProfileUnavailable
    );
    assert_eq!(out_of_scope.profile_id(), Some(JSON));
}

#[test]
fn unknown_and_duplicate_profile_ids_return_stable_validation_codes() {
    let empty = ProfileValidationSelection::new(std::iter::empty::<&str>())
        .validate()
        .expect_err("an empty selection cannot activate hidden defaults");
    assert_eq!(empty.code(), ProfileValidationErrorCode::EmptySelection);

    let secret_marker = "private-key-sentinel";
    let selection = ProfileValidationSelection::new([secret_marker]);
    let unknown = selection
        .validate()
        .expect_err("unknown IDs cannot resolve to catalog metadata");
    assert_eq!(unknown.code(), ProfileValidationErrorCode::UnknownProfile);
    assert!(!unknown.to_string().contains(secret_marker));
    assert!(!format!("{unknown:?}").contains(secret_marker));
    assert!(!format!("{selection:?}").contains(secret_marker));

    let server = ProfileValidationSelection::new([BASELINE_SERVER])
        .validate()
        .expect_err("server profiles cannot be selected by a KMIPKit client");
    assert_eq!(server.code(), ProfileValidationErrorCode::UnknownProfile);
    assert_eq!(server.profile_id(), None);

    let duplicate = ProfileValidationSelection::new([BASELINE, BASELINE])
        .validate()
        .expect_err("duplicate selection IDs are ambiguous");
    assert_eq!(
        duplicate.code(),
        ProfileValidationErrorCode::DuplicateProfile
    );
}

#[test]
fn generated_client_keywords_retain_all_five_verification_obligations() {
    let categories: Vec<GeneratedVerificationCategory> = CLIENT_PROFILES
        .iter()
        .flat_map(|profile| profile.requirement_ids)
        .filter_map(|requirement_id| {
            CLIENT_REQUIREMENTS
                .iter()
                .find(|requirement| requirement.requirement_id == *requirement_id)
                .map(|requirement| requirement.verification_category)
        })
        .collect();

    for expected in [
        GeneratedVerificationCategory::RequiredBehavior,
        GeneratedVerificationCategory::ProhibitedBehavior,
        GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        GeneratedVerificationCategory::DiscouragedAbsenceOrDeviation,
        GeneratedVerificationCategory::OptionalCapability,
    ] {
        assert!(
            categories.contains(&expected),
            "missing category {expected:?}"
        );
    }

    for requirement in CLIENT_REQUIREMENTS {
        let expected_category = match requirement.source_keyword {
            Some("MUST" | "SHALL" | "REQUIRED") => {
                Some(GeneratedVerificationCategory::RequiredBehavior)
            }
            Some("MUST NOT" | "SHALL NOT") => {
                Some(GeneratedVerificationCategory::ProhibitedBehavior)
            }
            Some("SHOULD" | "RECOMMENDED") => {
                Some(GeneratedVerificationCategory::RecommendedBehaviorOrDeviation)
            }
            Some("SHOULD NOT") => {
                Some(GeneratedVerificationCategory::DiscouragedAbsenceOrDeviation)
            }
            Some("MAY" | "OPTIONAL") => Some(GeneratedVerificationCategory::OptionalCapability),
            Some(keyword) => panic!("unclassified OASIS normative keyword {keyword}"),
            None => None,
        };
        if let Some(expected_category) = expected_category {
            assert_eq!(
                requirement.verification_category, expected_category,
                "wrong evidence category for {} from source clause {:?}",
                requirement.requirement_id, requirement.source_clause_ids
            );
        }
    }
}
