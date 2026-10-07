//! Explicit, fail-closed client profile selection.
//!
//! Generated catalog records describe applicability and source obligations.
//! They do not supply executable rules. The rule registry remains empty until
//! a requirement has an explicit typed implementation and category-matched
//! executable verification, so current catalog profiles cannot be selected
//! for runtime validation. The profile prerequisite is KMIP Specification v2.1
//! §14.1; Baseline Client's inherited profile closure follows KMIP Profiles
//! v2.1 §6.1.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use crate::profiles_generated::{
    CLIENT_PROFILES, CLIENT_REQUIREMENTS, GeneratedClientProfile, GeneratedClientRequirement,
    GeneratedVerificationCategory,
};
use crate::{RequestMessage, ResponseMessage};

/// Stable reason that explicit profile validation could not be prepared.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileValidationErrorCode {
    /// No profile was explicitly selected.
    EmptySelection,
    /// The selection repeats a profile identifier.
    DuplicateProfile,
    /// The identifier does not resolve to a generated client profile.
    UnknownProfile,
    /// The profile is outside the approved client product scope.
    ProfileUnavailable,
    /// A conditional profile has no satisfied typed condition in this release.
    ConditionalProfileUnsatisfied,
    /// The generated dependency graph contains a cycle.
    DependencyCycle,
    /// A profile closure references a missing generated requirement record.
    MissingRequirementMetadata,
    /// An applicable requirement has no complete typed rule mapping.
    MissingRuleMapping,
    /// A typed mapping is present but lacks an executable evaluator.
    IncompleteRuleMapping,
    /// A typed mapping does not cite the exact requirement category or clause.
    RuleMappingMismatch,
    /// A mapping does not contain the category-appropriate test reference.
    MissingCategoryTest,
    /// Selected rules require incompatible values for the same typed constraint.
    ConflictingConstraints,
}

impl ProfileValidationErrorCode {
    /// Returns the stable machine-readable error code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptySelection => "empty_selection",
            Self::DuplicateProfile => "duplicate_profile",
            Self::UnknownProfile => "unknown_profile",
            Self::ProfileUnavailable => "profile_unavailable",
            Self::ConditionalProfileUnsatisfied => "conditional_profile_unsatisfied",
            Self::DependencyCycle => "dependency_cycle",
            Self::MissingRequirementMetadata => "missing_requirement_metadata",
            Self::MissingRuleMapping => "missing_rule_mapping",
            Self::IncompleteRuleMapping => "incomplete_rule_mapping",
            Self::RuleMappingMismatch => "rule_mapping_mismatch",
            Self::MissingCategoryTest => "missing_category_test",
            Self::ConflictingConstraints => "conflicting_constraints",
        }
    }
}

/// Explicit profile IDs requested for one client exchange.
#[derive(Clone, Eq, PartialEq)]
pub struct ProfileValidationSelection {
    profile_ids: Vec<String>,
}

impl ProfileValidationSelection {
    /// Creates an explicit selection. Validation resolves the IDs and their
    /// dependencies against generated catalog metadata.
    #[must_use]
    pub fn new<I, S>(profile_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            profile_ids: profile_ids.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns the explicitly selected profile IDs in caller order.
    #[must_use]
    pub fn profile_ids(&self) -> &[String] {
        &self.profile_ids
    }

    /// Resolves profile dependencies and verifies that every applicable
    /// requirement has a typed rule mapping. Product target membership is not
    /// consulted. Current profiles fail closed because no mappings are
    /// registered yet.
    ///
    /// # Errors
    ///
    /// Returns a stable source-backed error for unknown, duplicate,
    /// unavailable, conditional, cyclic, incomplete, or empty selections.
    pub fn validate(&self) -> Result<ProfileValidationResult, ProfileValidationError> {
        validate_selection(self, CLIENT_PROFILES, CLIENT_REQUIREMENTS)
    }
}

impl fmt::Debug for ProfileValidationSelection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProfileValidationSelection")
            .field("profile_count", &self.profile_ids.len())
            .finish()
    }
}

/// A profile selection that has resolved all dependencies and rule coverage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileValidationResult {
    profile_ids: Vec<String>,
    requirement_ids: Vec<String>,
}

impl ProfileValidationResult {
    /// Returns the selected profiles and their dependency closure in stable order.
    #[must_use]
    pub fn profile_ids(&self) -> &[String] {
        &self.profile_ids
    }

    /// Returns all applicable requirement IDs in stable order.
    #[must_use]
    pub fn requirement_ids(&self) -> &[String] {
        &self.requirement_ids
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuleTestKind {
    PositiveBehavior,
    NegativeBehavior,
    RecommendedBehavior,
    DiscouragedBehaviorAbsent,
    OptionalCapabilityUsable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RuleTestEvidence {
    test_id: &'static str,
    kind: RuleTestKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RuleConstraint {
    key: &'static str,
    value: &'static str,
}

type RuleEvaluator = for<'a> fn(
    Option<&'a RequestMessage>,
    Option<(&'a RequestMessage, &'a ResponseMessage)>,
) -> bool;

#[derive(Clone, Copy)]
struct TypedRuleMapping {
    requirement_id: &'static str,
    source_clause_id: &'static str,
    verification_category: GeneratedVerificationCategory,
    evaluator: Option<RuleEvaluator>,
    test_evidence: &'static [RuleTestEvidence],
    constraints: &'static [RuleConstraint],
}

// This registry intentionally remains empty until each entry has a typed
// evaluator and category-appropriate executable tests. Empty means unavailable.
const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[];

/// Redacted validation failure containing only stable IDs and a fixed code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileValidationError {
    code: ProfileValidationErrorCode,
    profile_id: Option<String>,
    requirement_id: Option<&'static str>,
    source_clause_id: Option<&'static str>,
}

impl ProfileValidationError {
    fn new(code: ProfileValidationErrorCode) -> Self {
        Self {
            code,
            profile_id: None,
            requirement_id: None,
            source_clause_id: None,
        }
    }

    fn for_profile(code: ProfileValidationErrorCode, profile_id: &str) -> Self {
        Self {
            code,
            profile_id: Some(profile_id.to_owned()),
            requirement_id: None,
            source_clause_id: None,
        }
    }

    fn for_requirement(
        code: ProfileValidationErrorCode,
        profile_id: &str,
        requirement_id: &'static str,
        source_clause_id: Option<&'static str>,
    ) -> Self {
        Self {
            code,
            profile_id: Some(profile_id.to_owned()),
            requirement_id: Some(requirement_id),
            source_clause_id,
        }
    }

    /// Returns the stable machine-readable failure code.
    #[must_use]
    pub const fn code(&self) -> ProfileValidationErrorCode {
        self.code
    }

    /// Returns the profile involved in the failure, when it resolved.
    #[must_use]
    pub fn profile_id(&self) -> Option<&str> {
        self.profile_id.as_deref()
    }

    /// Returns the source-backed requirement involved in the failure.
    #[must_use]
    pub const fn requirement_id(&self) -> Option<&'static str> {
        self.requirement_id
    }

    /// Returns the exact source clause for a requirement failure.
    #[must_use]
    pub const fn source_clause_id(&self) -> Option<&'static str> {
        self.source_clause_id
    }
}

impl fmt::Display for ProfileValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "profile validation failed: {}",
            self.code.as_str()
        )?;
        if let Some(profile_id) = &self.profile_id {
            write!(formatter, "; profile {profile_id}")?;
        }
        if let Some(requirement_id) = self.requirement_id {
            write!(formatter, "; requirement {requirement_id}")?;
        }
        if let Some(source_clause_id) = self.source_clause_id {
            write!(formatter, "; source clause {source_clause_id}")?;
        }
        Ok(())
    }
}

impl Error for ProfileValidationError {}

fn validate_selection(
    selection: &ProfileValidationSelection,
    profiles: &[GeneratedClientProfile],
    requirements: &[GeneratedClientRequirement],
) -> Result<ProfileValidationResult, ProfileValidationError> {
    validate_selection_with_registry(selection, profiles, requirements, TYPED_RULE_MAPPINGS)
}

fn validate_selection_with_registry(
    selection: &ProfileValidationSelection,
    profiles: &[GeneratedClientProfile],
    requirements: &[GeneratedClientRequirement],
    rule_mappings: &[TypedRuleMapping],
) -> Result<ProfileValidationResult, ProfileValidationError> {
    if selection.profile_ids.is_empty() {
        return Err(ProfileValidationError::new(
            ProfileValidationErrorCode::EmptySelection,
        ));
    }

    let mut unique_ids = BTreeSet::new();
    for profile_id in &selection.profile_ids {
        if !unique_ids.insert(profile_id.as_str()) {
            return Err(if find_profile(profiles, profile_id).is_some() {
                ProfileValidationError::for_profile(
                    ProfileValidationErrorCode::DuplicateProfile,
                    profile_id,
                )
            } else {
                ProfileValidationError::new(ProfileValidationErrorCode::DuplicateProfile)
            });
        }
    }

    let mut ordered_roots = selection.profile_ids.clone();
    ordered_roots.sort();
    let mut resolved_ids = BTreeSet::new();
    let mut visiting_ids = BTreeSet::new();
    let mut dependency_order = Vec::new();
    for profile_id in &ordered_roots {
        resolve_profile(
            profile_id,
            profiles,
            &mut resolved_ids,
            &mut visiting_ids,
            &mut dependency_order,
        )?;
    }

    let mut requirement_ids = BTreeSet::new();
    let mut constraints = std::collections::BTreeMap::new();
    for profile_id in &dependency_order {
        let profile = find_profile(profiles, profile_id).ok_or_else(|| {
            ProfileValidationError::for_profile(
                ProfileValidationErrorCode::UnknownProfile,
                profile_id,
            )
        })?;
        if profile.requirement_ids.is_empty() {
            return Err(ProfileValidationError::for_profile(
                ProfileValidationErrorCode::MissingRequirementMetadata,
                profile_id,
            ));
        }
        for requirement_id in profile.requirement_ids {
            let requirement = requirements
                .iter()
                .find(|requirement| requirement.requirement_id == *requirement_id)
                .ok_or_else(|| {
                    ProfileValidationError::for_profile(
                        ProfileValidationErrorCode::MissingRequirementMetadata,
                        profile_id,
                    )
                })?;
            let Some(mapping) = rule_mappings
                .iter()
                .find(|mapping| mapping.requirement_id == requirement.requirement_id)
            else {
                return Err(ProfileValidationError::for_requirement(
                    ProfileValidationErrorCode::MissingRuleMapping,
                    profile_id,
                    requirement.requirement_id,
                    requirement.source_clause_ids.first().copied(),
                ));
            };
            validate_mapping(profile_id, requirement, mapping)?;
            for constraint in mapping.constraints {
                if let Some(existing_value) = constraints.get(constraint.key) {
                    if *existing_value != constraint.value {
                        return Err(ProfileValidationError::for_requirement(
                            ProfileValidationErrorCode::ConflictingConstraints,
                            profile_id,
                            requirement.requirement_id,
                            Some(mapping.source_clause_id),
                        ));
                    }
                } else {
                    constraints.insert(constraint.key, constraint.value);
                }
            }
            requirement_ids.insert(requirement.requirement_id);
        }
    }

    Ok(ProfileValidationResult {
        profile_ids: resolved_ids.into_iter().collect(),
        requirement_ids: requirement_ids.into_iter().map(str::to_owned).collect(),
    })
}

fn resolve_profile(
    profile_id: &str,
    profiles: &[GeneratedClientProfile],
    resolved_ids: &mut BTreeSet<String>,
    visiting_ids: &mut BTreeSet<String>,
    dependency_order: &mut Vec<String>,
) -> Result<(), ProfileValidationError> {
    if resolved_ids.contains(profile_id) {
        return Ok(());
    }
    if !visiting_ids.insert(profile_id.to_owned()) {
        return Err(ProfileValidationError::for_profile(
            ProfileValidationErrorCode::DependencyCycle,
            profile_id,
        ));
    }
    let Some(profile) = find_profile(profiles, profile_id) else {
        return Err(ProfileValidationError::new(
            ProfileValidationErrorCode::UnknownProfile,
        ));
    };
    match profile.applicability {
        "client_1_0" => {}
        "conditional" => {
            return Err(ProfileValidationError::for_profile(
                ProfileValidationErrorCode::ConditionalProfileUnsatisfied,
                profile_id,
            ));
        }
        _ => {
            return Err(ProfileValidationError::for_profile(
                ProfileValidationErrorCode::ProfileUnavailable,
                profile_id,
            ));
        }
    }

    let mut dependencies = profile.dependency_profile_ids.to_vec();
    dependencies.sort_unstable();
    for dependency_id in dependencies {
        resolve_profile(
            dependency_id,
            profiles,
            resolved_ids,
            visiting_ids,
            dependency_order,
        )?;
    }
    visiting_ids.remove(profile_id);
    resolved_ids.insert(profile_id.to_owned());
    dependency_order.push(profile_id.to_owned());
    Ok(())
}

fn find_profile<'a>(
    profiles: &'a [GeneratedClientProfile],
    profile_id: &str,
) -> Option<&'a GeneratedClientProfile> {
    profiles
        .iter()
        .find(|profile| profile.profile_id == profile_id)
}

fn validate_mapping(
    profile_id: &str,
    requirement: &GeneratedClientRequirement,
    mapping: &TypedRuleMapping,
) -> Result<(), ProfileValidationError> {
    let error = |code| {
        ProfileValidationError::for_requirement(
            code,
            profile_id,
            requirement.requirement_id,
            Some(mapping.source_clause_id),
        )
    };
    if !requirement
        .source_clause_ids
        .contains(&mapping.source_clause_id)
        || mapping.verification_category != requirement.verification_category
    {
        return Err(error(ProfileValidationErrorCode::RuleMappingMismatch));
    }
    if mapping.evaluator.is_none() {
        return Err(error(ProfileValidationErrorCode::IncompleteRuleMapping));
    }

    let required_test_kind = match requirement.verification_category {
        GeneratedVerificationCategory::RequiredBehavior => RuleTestKind::PositiveBehavior,
        GeneratedVerificationCategory::ProhibitedBehavior => RuleTestKind::NegativeBehavior,
        GeneratedVerificationCategory::RecommendedBehaviorOrDeviation
        | GeneratedVerificationCategory::DiscouragedAbsenceOrDeviation
            if requirement.has_accepted_deviation =>
        {
            return Ok(());
        }
        GeneratedVerificationCategory::RecommendedBehaviorOrDeviation => {
            RuleTestKind::RecommendedBehavior
        }
        GeneratedVerificationCategory::DiscouragedAbsenceOrDeviation => {
            RuleTestKind::DiscouragedBehaviorAbsent
        }
        GeneratedVerificationCategory::OptionalCapability => RuleTestKind::OptionalCapabilityUsable,
    };
    if !mapping
        .test_evidence
        .iter()
        .any(|evidence| !evidence.test_id.trim().is_empty() && evidence.kind == required_test_kind)
    {
        return Err(error(ProfileValidationErrorCode::MissingCategoryTest));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ProfileValidationErrorCode, ProfileValidationSelection, RuleConstraint, RuleTestEvidence,
        RuleTestKind, TypedRuleMapping, validate_mapping, validate_selection_with_registry,
    };
    use crate::profiles_generated::{
        GeneratedClientProfile, GeneratedClientRequirement, GeneratedVerificationCategory,
    };

    static CONSTRAINTS_HTTPS: [RuleConstraint; 1] = [RuleConstraint {
        key: "transport",
        value: "https",
    }];
    static CONSTRAINTS_RAW_TLS: [RuleConstraint; 1] = [RuleConstraint {
        key: "transport",
        value: "raw_tls",
    }];
    static POSITIVE_TEST: [RuleTestEvidence; 1] = [RuleTestEvidence {
        test_id: "profile-rule-positive-test",
        kind: RuleTestKind::PositiveBehavior,
    }];
    static NEGATIVE_TEST: [RuleTestEvidence; 1] = [RuleTestEvidence {
        test_id: "profile-rule-negative-test",
        kind: RuleTestKind::NegativeBehavior,
    }];
    static RECOMMENDED_TEST: [RuleTestEvidence; 1] = [RuleTestEvidence {
        test_id: "profile-rule-recommended-test",
        kind: RuleTestKind::RecommendedBehavior,
    }];
    static DISCOURAGED_ABSENCE_TEST: [RuleTestEvidence; 1] = [RuleTestEvidence {
        test_id: "profile-rule-discouraged-absence-test",
        kind: RuleTestKind::DiscouragedBehaviorAbsent,
    }];
    static OPTIONAL_CAPABILITY_TEST: [RuleTestEvidence; 1] = [RuleTestEvidence {
        test_id: "profile-rule-optional-capability-test",
        kind: RuleTestKind::OptionalCapabilityUsable,
    }];

    fn passing_rule(
        _request: Option<&crate::RequestMessage>,
        _response: Option<(&crate::RequestMessage, &crate::ResponseMessage)>,
    ) -> bool {
        true
    }

    fn profile(
        profile_id: &'static str,
        requirement_ids: &'static [&'static str],
        dependency_profile_ids: &'static [&'static str],
    ) -> GeneratedClientProfile {
        GeneratedClientProfile {
            profile_id,
            name: "synthetic test profile",
            applicability: "client_1_0",
            claim_state: "not_claimed",
            selected_target: false,
            direct_source_clause_ids: &["TEST-CLAUSE"],
            source_clause_ids: &["TEST-CLAUSE"],
            requirement_ids,
            dependency_profile_ids,
            transport_requirements: &[],
            encoding_requirements: &[],
            test_cases: &[],
        }
    }

    fn requirement(
        requirement_id: &'static str,
        category: GeneratedVerificationCategory,
        has_accepted_deviation: bool,
    ) -> GeneratedClientRequirement {
        GeneratedClientRequirement {
            requirement_id,
            source_keyword: "SHALL",
            verification_category: category,
            scope_state: "client_1_0",
            source_clause_ids: &["TEST-CLAUSE"],
            profile_ids: &[],
            official_test_case_ids: &[],
            has_accepted_deviation,
        }
    }

    fn mapping(
        requirement_id: &'static str,
        category: GeneratedVerificationCategory,
        test_kind: RuleTestKind,
        constraints: &'static [RuleConstraint],
    ) -> TypedRuleMapping {
        let test_evidence = match test_kind {
            RuleTestKind::PositiveBehavior => &POSITIVE_TEST,
            RuleTestKind::NegativeBehavior => &NEGATIVE_TEST,
            RuleTestKind::RecommendedBehavior => &RECOMMENDED_TEST,
            RuleTestKind::DiscouragedBehaviorAbsent => &DISCOURAGED_ABSENCE_TEST,
            RuleTestKind::OptionalCapabilityUsable => &OPTIONAL_CAPABILITY_TEST,
        };
        TypedRuleMapping {
            requirement_id,
            source_clause_id: "TEST-CLAUSE",
            verification_category: category,
            evaluator: Some(passing_rule),
            test_evidence,
            constraints,
        }
    }

    #[test]
    fn each_keyword_category_requires_the_matching_test_form() {
        let cases = [
            (
                GeneratedVerificationCategory::RequiredBehavior,
                RuleTestKind::PositiveBehavior,
            ),
            (
                GeneratedVerificationCategory::ProhibitedBehavior,
                RuleTestKind::NegativeBehavior,
            ),
            (
                GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
                RuleTestKind::RecommendedBehavior,
            ),
            (
                GeneratedVerificationCategory::DiscouragedAbsenceOrDeviation,
                RuleTestKind::DiscouragedBehaviorAbsent,
            ),
            (
                GeneratedVerificationCategory::OptionalCapability,
                RuleTestKind::OptionalCapabilityUsable,
            ),
        ];

        for (category, test_kind) in cases {
            let requirement = requirement("TEST-REQ", category, false);
            let good_mapping = mapping("TEST-REQ", category, test_kind, &[]);
            assert!(validate_mapping("TEST-PROFILE", &requirement, &good_mapping).is_ok());

            let wrong_test_kind = match test_kind {
                RuleTestKind::PositiveBehavior => RuleTestKind::NegativeBehavior,
                _ => RuleTestKind::PositiveBehavior,
            };
            let mapping = mapping("TEST-REQ", category, wrong_test_kind, &[]);
            let error = validate_mapping("TEST-PROFILE", &requirement, &mapping)
                .expect_err("a different test category cannot cover the rule");
            assert_eq!(
                error.code(),
                ProfileValidationErrorCode::MissingCategoryTest
            );
        }
    }

    #[test]
    fn accepted_deviations_cover_only_recommended_and_discouraged_categories() {
        for category in [
            GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
            GeneratedVerificationCategory::DiscouragedAbsenceOrDeviation,
        ] {
            let requirement = requirement("TEST-REQ", category, true);
            let mapping = TypedRuleMapping {
                test_evidence: &[],
                ..mapping("TEST-REQ", category, RuleTestKind::PositiveBehavior, &[])
            };
            assert!(validate_mapping("TEST-PROFILE", &requirement, &mapping).is_ok());
        }

        let requirement = requirement(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            true,
        );
        let mapping = TypedRuleMapping {
            test_evidence: &[],
            ..mapping(
                "TEST-REQ",
                GeneratedVerificationCategory::RequiredBehavior,
                RuleTestKind::PositiveBehavior,
                &[],
            )
        };
        let error = validate_mapping("TEST-PROFILE", &requirement, &mapping)
            .expect_err("a deviation cannot replace required behavior verification");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::MissingCategoryTest
        );
    }

    #[test]
    fn a_complete_synthetic_mapping_allows_selection_without_target_membership() {
        let profiles = [profile("TEST-PROFILE", &["TEST-REQ"], &[])];
        let requirements = [requirement(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            false,
        )];
        let mappings = [mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &[],
        )];
        let selection = ProfileValidationSelection::new(["TEST-PROFILE"]);

        let result =
            validate_selection_with_registry(&selection, &profiles, &requirements, &mappings)
                .expect("complete synthetic code and verification evidence is selectable");

        assert_eq!(result.profile_ids(), &["TEST-PROFILE"]);
        assert_eq!(result.requirement_ids(), &["TEST-REQ"]);
    }

    #[test]
    fn dependency_cycles_and_constraint_conflicts_fail_deterministically() {
        let cyclic_profiles = [
            profile("TEST-A", &[], &["TEST-B"]),
            profile("TEST-B", &[], &["TEST-A"]),
        ];
        let cycle_error = validate_selection_with_registry(
            &ProfileValidationSelection::new(["TEST-A"]),
            &cyclic_profiles,
            &[],
            &[],
        )
        .expect_err("cyclic dependencies are rejected");
        assert_eq!(
            cycle_error.code(),
            ProfileValidationErrorCode::DependencyCycle
        );

        let profiles = [
            profile("TEST-A", &["TEST-REQ-A"], &[]),
            profile("TEST-B", &["TEST-REQ-B"], &[]),
        ];
        let requirements = [
            requirement(
                "TEST-REQ-A",
                GeneratedVerificationCategory::RequiredBehavior,
                false,
            ),
            requirement(
                "TEST-REQ-B",
                GeneratedVerificationCategory::RequiredBehavior,
                false,
            ),
        ];
        let mappings = [
            mapping(
                "TEST-REQ-A",
                GeneratedVerificationCategory::RequiredBehavior,
                RuleTestKind::PositiveBehavior,
                &CONSTRAINTS_HTTPS,
            ),
            mapping(
                "TEST-REQ-B",
                GeneratedVerificationCategory::RequiredBehavior,
                RuleTestKind::PositiveBehavior,
                &CONSTRAINTS_RAW_TLS,
            ),
        ];
        let error = validate_selection_with_registry(
            &ProfileValidationSelection::new(["TEST-B", "TEST-A"]),
            &profiles,
            &requirements,
            &mappings,
        )
        .expect_err("conflicting selected constraints have no order-based resolution");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::ConflictingConstraints
        );
    }

    #[test]
    fn missing_evaluator_and_wrong_source_link_fail_closed() {
        let requirement = requirement(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            false,
        );
        let missing_evaluator = TypedRuleMapping {
            evaluator: None,
            ..mapping(
                "TEST-REQ",
                GeneratedVerificationCategory::RequiredBehavior,
                RuleTestKind::PositiveBehavior,
                &[],
            )
        };
        let error = validate_mapping("TEST-PROFILE", &requirement, &missing_evaluator)
            .expect_err("rule metadata without executable code is incomplete");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::IncompleteRuleMapping
        );

        let wrong_clause = TypedRuleMapping {
            source_clause_id: "OTHER-CLAUSE",
            ..mapping(
                "TEST-REQ",
                GeneratedVerificationCategory::RequiredBehavior,
                RuleTestKind::PositiveBehavior,
                &[],
            )
        };
        let error = validate_mapping("TEST-PROFILE", &requirement, &wrong_clause)
            .expect_err("rule source links must match the catalog");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RuleMappingMismatch
        );
    }
}
