use kmipkit_protocol::{
    ProfileValidationErrorCode, ProfileValidationSelection,
    profiles_generated::{CLIENT_PROFILES, GeneratedVerificationCategory},
};

const BASELINE: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
const AES_XTS: &str = "KMIPKIT-PROFILE-AES-XTS-CLIENT";
const HTTPS: &str = "KMIPKIT-PROFILE-HTTPS-CLIENT";
const JSON: &str = "KMIPKIT-PROFILE-JSON-CLIENT";

#[test]
fn baseline_target_membership_does_not_make_runtime_selection_available() {
    let target = CLIENT_PROFILES
        .iter()
        .find(|profile| profile.profile_id == BASELINE)
        .expect("the generated catalog includes Baseline Client");
    assert!(target.selected_target);

    let error = ProfileValidationSelection::new([BASELINE])
        .validate()
        .expect_err("Baseline has no complete typed rule registry yet");

    assert_eq!(error.code(), ProfileValidationErrorCode::MissingRuleMapping);
    assert_eq!(error.profile_id(), Some(BASELINE));
    assert_eq!(error.requirement_id(), Some("KMIPKIT-REQ-PROF-3.1-001"));
    assert_eq!(error.source_clause_id(), Some("KMIPKIT-CLAUSE-PROF-3.1-001"));
}

#[test]
fn in_scope_non_target_profile_uses_dependency_closure_without_target_gating() {
    let target = CLIENT_PROFILES
        .iter()
        .find(|profile| profile.profile_id == AES_XTS)
        .expect("the generated catalog includes AES-XTS Client");
    assert_eq!(target.applicability, "client_1_0");
    assert!(!target.selected_target);

    let error = ProfileValidationSelection::new([AES_XTS])
        .validate()
        .expect_err("the Baseline dependency has no complete rule registry");

    assert_eq!(error.code(), ProfileValidationErrorCode::MissingRuleMapping);
    assert_eq!(error.profile_id(), Some(BASELINE));
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
    assert_eq!(out_of_scope.code(), ProfileValidationErrorCode::ProfileUnavailable);
    assert_eq!(out_of_scope.profile_id(), Some(JSON));
}

#[test]
fn unknown_and_duplicate_profile_ids_return_stable_validation_codes() {
    let unknown = ProfileValidationSelection::new(["KMIPKIT-PROFILE-UNKNOWN"])
        .validate()
        .expect_err("unknown IDs cannot resolve to catalog metadata");
    assert_eq!(unknown.code(), ProfileValidationErrorCode::UnknownProfile);

    let duplicate = ProfileValidationSelection::new([BASELINE, BASELINE])
        .validate()
        .expect_err("duplicate selection IDs are ambiguous");
    assert_eq!(duplicate.code(), ProfileValidationErrorCode::DuplicateProfile);
}

#[test]
fn generated_client_keywords_retain_all_five_verification_obligations() {
    let categories: Vec<GeneratedVerificationCategory> = CLIENT_PROFILES
        .iter()
        .flat_map(|profile| profile.requirement_ids)
        .filter_map(|requirement_id| {
            kmipkit_protocol::profiles_generated::CLIENT_REQUIREMENTS
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
        assert!(categories.contains(&expected), "missing category {expected:?}");
    }
}
