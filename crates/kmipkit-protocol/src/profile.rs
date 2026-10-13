//! Explicit, fail-closed client profile selection.
//!
//! Generated catalog records describe applicability and source obligations.
//! Only explicitly mapped rules with category-matched executable verification
//! are enforced; remaining unresolved or unmapped obligations keep a profile
//! unavailable for runtime validation. The profile prerequisite is KMIP
//! Specification v2.1 §14.1; Baseline Client's inherited profile closure
//! follows KMIP Profiles v2.1 §6.1.

use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{StructureView, ValueView};

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
    /// A linked client obligation has no reviewed applicability disposition.
    UnresolvedRequirementScope,
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
    /// A mapped rule rejected the outbound request.
    RequestRuleRejected,
    /// A mapped rule rejected the received response.
    ResponseRuleRejected,
    /// The configured transport violates a selected profile rule.
    TransportConstraintRejected,
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
            Self::UnresolvedRequirementScope => "unresolved_requirement_scope",
            Self::MissingRuleMapping => "missing_rule_mapping",
            Self::IncompleteRuleMapping => "incomplete_rule_mapping",
            Self::RuleMappingMismatch => "rule_mapping_mismatch",
            Self::MissingCategoryTest => "missing_category_test",
            Self::ConflictingConstraints => "conflicting_constraints",
            Self::RequestRuleRejected => "request_rule_rejected",
            Self::ResponseRuleRejected => "response_rule_rejected",
            Self::TransportConstraintRejected => "transport_constraint_rejected",
        }
    }
}

/// The KMIP 1.0 transport selected for one client exchange.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileTransport {
    /// Raw TTLV records over TLS.
    RawTtlvTls,
    /// TTLV over HTTPS using HTTP/1.1.
    Https,
}

impl ProfileTransport {
    const fn as_rule_value(self) -> &'static str {
        match self {
            Self::RawTtlvTls => "raw_tls",
            Self::Https => "https",
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
    /// consulted. Profiles fail closed while any applicable requirement
    /// remains unresolved or unmapped.
    ///
    /// # Errors
    ///
    /// Returns a stable source-backed error for unknown, duplicate,
    /// unavailable, conditional, unresolved-scope, cyclic, incomplete, or
    /// empty selections.
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

    /// Checks typed transport constraints attached to the selected rules.
    ///
    /// # Errors
    ///
    /// Returns a stable source-backed error when a selected rule requires a
    /// different transport or its mapping is no longer present.
    pub fn validate_transport(
        &self,
        transport: ProfileTransport,
    ) -> Result<(), ProfileValidationError> {
        self.validate_transport_with_mappings(transport, TYPED_RULE_MAPPINGS)
    }

    fn validate_transport_with_mappings(
        &self,
        transport: ProfileTransport,
        rule_mappings: &[TypedRuleMapping],
    ) -> Result<(), ProfileValidationError> {
        for requirement_id in &self.requirement_ids {
            let profile_id = self.profile_id_for_requirement(requirement_id);
            let Some(mapping) = rule_mappings
                .iter()
                .find(|mapping| mapping.requirement_id == requirement_id.as_str())
            else {
                return Err(ProfileValidationError::for_requirement(
                    ProfileValidationErrorCode::MissingRuleMapping,
                    profile_id,
                    requirement_id,
                    None,
                ));
            };
            for constraint in mapping.constraints {
                if constraint.key == "transport" && constraint.value != transport.as_rule_value() {
                    return Err(ProfileValidationError::for_requirement(
                        ProfileValidationErrorCode::TransportConstraintRejected,
                        profile_id,
                        requirement_id,
                        Some(mapping.source_clause_id),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Evaluates the selected typed rules against one complete outbound KMIP
    /// request before the client sends it.
    ///
    /// # Errors
    ///
    /// Returns a stable requirement and source-clause error when a mapped
    /// evaluator rejects the request or the validated registry is no longer
    /// complete.
    pub fn validate_request(&self, request: &RequestMessage) -> Result<(), ProfileValidationError> {
        self.evaluate_rules(
            Some(request),
            None,
            TYPED_RULE_MAPPINGS,
            ProfileValidationErrorCode::RequestRuleRejected,
        )
    }

    /// Evaluates the selected typed rules against one received response while
    /// retaining its original request context.
    ///
    /// # Errors
    ///
    /// Returns a stable requirement and source-clause error when a mapped
    /// evaluator rejects the response or the validated registry is no longer
    /// complete.
    pub fn validate_response(
        &self,
        request: &RequestMessage,
        response: &ResponseMessage,
    ) -> Result<(), ProfileValidationError> {
        self.evaluate_rules(
            Some(request),
            Some((request, response)),
            TYPED_RULE_MAPPINGS,
            ProfileValidationErrorCode::ResponseRuleRejected,
        )
    }

    fn evaluate_rules(
        &self,
        request: Option<&RequestMessage>,
        response: Option<(&RequestMessage, &ResponseMessage)>,
        rule_mappings: &[TypedRuleMapping],
        rejection_code: ProfileValidationErrorCode,
    ) -> Result<(), ProfileValidationError> {
        for requirement_id in &self.requirement_ids {
            let profile_id = self.profile_id_for_requirement(requirement_id);
            let Some(mapping) = rule_mappings
                .iter()
                .find(|mapping| mapping.requirement_id == requirement_id.as_str())
            else {
                return Err(ProfileValidationError::for_requirement(
                    ProfileValidationErrorCode::MissingRuleMapping,
                    profile_id,
                    requirement_id,
                    None,
                ));
            };
            let Some(evaluator) = mapping.evaluator else {
                return Err(ProfileValidationError::for_requirement(
                    ProfileValidationErrorCode::IncompleteRuleMapping,
                    profile_id,
                    requirement_id,
                    Some(mapping.source_clause_id),
                ));
            };
            if !evaluator(request, response) {
                return Err(ProfileValidationError::for_requirement(
                    rejection_code,
                    profile_id,
                    requirement_id,
                    Some(mapping.source_clause_id),
                ));
            }
        }
        Ok(())
    }

    fn profile_id_for_requirement(&self, requirement_id: &str) -> &str {
        self.profile_ids
            .iter()
            .find(|profile_id| {
                find_profile(CLIENT_PROFILES, profile_id)
                    .is_some_and(|profile| profile.requirement_ids.contains(&requirement_id))
            })
            .map_or_else(
                || self.profile_ids.first().map_or("", String::as_str),
                String::as_str,
            )
    }
}

/// One caller-visible local record for a stable profile requirement.
///
/// These records help diagnose catalog and implementation coverage. They are
/// not CI results and cannot establish release evidence or publish a claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileEvidenceRecord {
    requirement_id: String,
    passed: bool,
}

impl ProfileEvidenceRecord {
    /// Creates a local record that reports its requirement check passed.
    #[must_use]
    pub fn passed(requirement_id: impl Into<String>) -> Self {
        Self {
            requirement_id: requirement_id.into(),
            passed: true,
        }
    }

    /// Creates a local record that reports its requirement check failed.
    #[must_use]
    pub fn failed(requirement_id: impl Into<String>) -> Self {
        Self {
            requirement_id: requirement_id.into(),
            passed: false,
        }
    }
}

/// Stable reasons that local profile evidence cannot be considered complete.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProfileReadinessBlocker {
    /// A required catalog requirement has no local record.
    MissingRequirementRecord { requirement_id: &'static str },
    /// A required catalog requirement has a failing local record.
    FailedRequirementRecord { requirement_id: &'static str },
    /// A client obligation in the source closure has unresolved applicability.
    UnresolvedRequirementScope {
        requirement_id: &'static str,
        source_clause_id: &'static str,
    },
    /// Local evidence repeats one required requirement.
    DuplicateRequirementRecord { requirement_id: &'static str },
    /// Local evidence names a requirement outside the generated closure.
    UnknownRequirementRecord,
    /// The profile's typed runtime rule registry is incomplete.
    RuntimeValidationUnavailable,
    /// A required official fixture is unavailable.
    MandatoryFixtureUnavailable { test_id: &'static str },
    /// The profile is conditional and has no satisfied typed condition.
    ConditionalProfileUnsatisfied,
    /// A dependency has not reached the catalog evidence-complete state.
    DependencyEvidenceIncomplete { profile_id: &'static str },
    /// The required current-run protected-release-push manifest is unavailable.
    VerificationManifestUnavailable,
    /// The catalog lifecycle says complete while recomputed evidence is blocked.
    StaleCatalogEvidenceState,
    /// Complete local evidence still requires a reviewed catalog lifecycle update.
    CatalogEvidenceReviewRequired,
    /// The profile is outside the approved 1.0 client scope.
    ProfileUnavailable,
}

impl ProfileReadinessBlocker {
    /// Returns a stable, redacted diagnostic message.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingRequirementRecord { .. } => "required requirement evidence is missing",
            Self::FailedRequirementRecord { .. } => "required requirement evidence failed",
            Self::UnresolvedRequirementScope { .. } => {
                "profile requirement applicability is unresolved"
            }
            Self::DuplicateRequirementRecord { .. } => "requirement evidence record is duplicated",
            Self::UnknownRequirementRecord => "requirement evidence record is unknown",
            Self::RuntimeValidationUnavailable => "runtime profile validation is unavailable",
            Self::MandatoryFixtureUnavailable { .. } => "mandatory profile fixture is unavailable",
            Self::ConditionalProfileUnsatisfied => "conditional profile is unsatisfied",
            Self::DependencyEvidenceIncomplete { .. } => {
                "dependency profile evidence is incomplete"
            }
            Self::VerificationManifestUnavailable => "current verification manifest is unavailable",
            Self::StaleCatalogEvidenceState => "catalog evidence-complete state is stale",
            Self::CatalogEvidenceReviewRequired => "catalog evidence state requires human review",
            Self::ProfileUnavailable => "profile is outside the KMIPKit 1.0 client scope",
        }
    }

    /// Returns the requirement ID for a requirement-specific blocker.
    #[must_use]
    pub const fn requirement_id(self) -> Option<&'static str> {
        match self {
            Self::MissingRequirementRecord { requirement_id }
            | Self::FailedRequirementRecord { requirement_id }
            | Self::DuplicateRequirementRecord { requirement_id }
            | Self::UnresolvedRequirementScope { requirement_id, .. } => Some(requirement_id),
            _ => None,
        }
    }

    /// Returns the exact source clause for an unresolved applicability blocker.
    #[must_use]
    pub const fn source_clause_id(self) -> Option<&'static str> {
        match self {
            Self::UnresolvedRequirementScope {
                source_clause_id, ..
            } => Some(source_clause_id),
            _ => None,
        }
    }
}

/// Read-only local readiness view for one generated client profile.
///
/// The result never publishes a claim. Local records, generated metadata, and
/// runtime rule completeness are insufficient without the repository's
/// current protected-release-push verification manifest. KMIP Specification
/// v2.1 §14.1 and Profiles v2.1 §6.1 remain the source for client profile
/// obligations; this local view is diagnostic evidence only.
///
/// # Examples
///
/// ```
/// use kmipkit_protocol::{ProfileReadiness, ProfileReadinessBlocker};
///
/// let readiness = ProfileReadiness::evaluate("KMIPKIT-PROFILE-BASELINE-CLIENT", &[])
///     .expect("Baseline Client is in the generated catalog");
///
/// assert!(readiness.target_selected());
/// assert!(!readiness.runtime_validation_available());
/// assert_eq!(readiness.effective_state(), "evidence_incomplete");
/// assert!(!readiness.published_claim());
/// assert!(readiness
///     .blockers()
///     .contains(&ProfileReadinessBlocker::VerificationManifestUnavailable));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileReadiness {
    profile_id: String,
    target_selected: bool,
    runtime_validation_available: bool,
    effective_state: String,
    requirement_records_complete: bool,
    blockers: Vec<ProfileReadinessBlocker>,
}

fn reconcile_effective_state(
    catalog_state: &str,
    blockers: &mut Vec<ProfileReadinessBlocker>,
) -> String {
    let effective_state = if catalog_state == "evidence_complete" && !blockers.is_empty() {
        blockers.push(ProfileReadinessBlocker::StaleCatalogEvidenceState);
        "evidence_incomplete"
    } else if catalog_state != "evidence_complete" && blockers.is_empty() {
        blockers.push(ProfileReadinessBlocker::CatalogEvidenceReviewRequired);
        catalog_state
    } else {
        catalog_state
    };
    blockers.sort();
    blockers.dedup();
    effective_state.to_owned()
}

impl ProfileReadiness {
    /// Evaluates local evidence records against a generated client profile.
    ///
    /// Returns `None` when `profile_id` does not resolve to generated client
    /// metadata. A complete set of local records does not advance catalog state
    /// or replace a current CI verification manifest.
    #[must_use]
    pub fn evaluate(profile_id: &str, records: &[ProfileEvidenceRecord]) -> Option<Self> {
        let profile = find_profile(CLIENT_PROFILES, profile_id)?;
        let required_ids: BTreeSet<_> = profile.requirement_ids.iter().copied().collect();
        let mut evidence_by_requirement: HashMap<&str, Vec<bool>> = HashMap::new();
        let mut has_unexpected_requirement = false;
        for record in records {
            if required_ids.contains(record.requirement_id.as_str()) {
                evidence_by_requirement
                    .entry(record.requirement_id.as_str())
                    .or_default()
                    .push(record.passed);
            } else {
                has_unexpected_requirement = true;
            }
        }

        let mut blockers = Vec::new();
        blockers.extend(
            profile
                .unresolved_scope_obligations
                .iter()
                .map(
                    |obligation| ProfileReadinessBlocker::UnresolvedRequirementScope {
                        requirement_id: obligation.requirement_id,
                        source_clause_id: obligation.source_clause_id,
                    },
                ),
        );
        let mut requirement_records_complete = true;
        for requirement_id in profile.requirement_ids {
            match evidence_by_requirement.get(requirement_id) {
                None => {
                    requirement_records_complete = false;
                    blockers
                        .push(ProfileReadinessBlocker::MissingRequirementRecord { requirement_id });
                }
                Some(results) if results.len() != 1 => {
                    requirement_records_complete = false;
                    blockers.push(ProfileReadinessBlocker::DuplicateRequirementRecord {
                        requirement_id,
                    });
                }
                Some(results) if results[0] => {}
                Some(_) => {
                    requirement_records_complete = false;
                    blockers
                        .push(ProfileReadinessBlocker::FailedRequirementRecord { requirement_id });
                }
            }
        }
        if has_unexpected_requirement {
            requirement_records_complete = false;
            blockers.push(ProfileReadinessBlocker::UnknownRequirementRecord);
        }

        let runtime_validation_available = ProfileValidationSelection::new([profile_id])
            .validate()
            .is_ok();
        if !runtime_validation_available {
            blockers.push(ProfileReadinessBlocker::RuntimeValidationUnavailable);
        }
        if profile.applicability == "conditional" {
            blockers.push(ProfileReadinessBlocker::ConditionalProfileUnsatisfied);
        } else if profile.applicability != "client_1_0" {
            blockers.push(ProfileReadinessBlocker::ProfileUnavailable);
        }
        blockers.extend(profile.test_cases.iter().filter_map(|test_case| {
            (test_case.mandatory_status == "mandatory"
                && test_case.fixture_availability != "available")
                .then_some(ProfileReadinessBlocker::MandatoryFixtureUnavailable {
                    test_id: test_case.test_id,
                })
        }));
        blockers.extend(
            profile
                .dependency_profile_ids
                .iter()
                .filter_map(|dependency_id| {
                    find_profile(CLIENT_PROFILES, dependency_id)
                        .is_none_or(|dependency| dependency.claim_state != "evidence_complete")
                        .then_some(ProfileReadinessBlocker::DependencyEvidenceIncomplete {
                            profile_id: dependency_id,
                        })
                }),
        );
        blockers.push(ProfileReadinessBlocker::VerificationManifestUnavailable);

        let effective_state = reconcile_effective_state(profile.claim_state, &mut blockers);

        Some(Self {
            profile_id: profile_id.to_owned(),
            target_selected: profile.selected_target,
            runtime_validation_available,
            effective_state,
            requirement_records_complete,
            blockers,
        })
    }

    /// Returns the generated profile identifier.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    /// Returns whether the fixed project target manifest selects this profile.
    #[must_use]
    pub const fn target_selected(&self) -> bool {
        self.target_selected
    }

    /// Returns whether a complete typed rule registry permits runtime validation.
    #[must_use]
    pub const fn runtime_validation_available(&self) -> bool {
        self.runtime_validation_available
    }

    /// Returns the effective catalog evidence lifecycle without upgrading it.
    #[must_use]
    pub fn effective_state(&self) -> &str {
        &self.effective_state
    }

    /// Returns whether every local requirement record is present and passing.
    #[must_use]
    pub const fn requirement_records_complete(&self) -> bool {
        self.requirement_records_complete
    }

    /// Returns deterministic stable blockers for this local readiness view.
    #[must_use]
    pub fn blockers(&self) -> &[ProfileReadinessBlocker] {
        &self.blockers
    }

    /// Returns `false`; readiness evaluation never publishes a conformance claim.
    #[must_use]
    pub const fn published_claim(&self) -> bool {
        false
    }
}

/// Compatibility name emphasizing that this is evidence metadata, not support status.
pub type ProfileEvidenceReadiness = ProfileReadiness;

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

const DISCOVER_VERSIONS_OPERATION: u32 = 0x0000_001E;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const CREATE_KEY_PAIR_OPERATION: u32 = 0x0000_0002;
const CREATE_KEY_PAIR_ATTRIBUTE_GROUPS: [u32; 3] = [0x0042_0126, 0x0042_0127, 0x0042_0128];
const CRYPTOGRAPHIC_DOMAIN_PARAMETERS: u32 = 0x0042_0029;

fn discover_versions_advertises_only_client_supported_versions(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if operation != DISCOVER_VERSIONS_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == PROTOCOL_VERSION)
                    .all(|field| {
                        field.with_value(|value| match value {
                            ValueView::Structure(version) => protocol_version_is_kmip_2_1(&version),
                            _ => false,
                        })
                    })
            })
            .unwrap_or(false)
    })
}

fn type_specific_parameters_do_not_infer_remote_object_type(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == CRYPTOGRAPHIC_PARAMETERS)
                    .all(|field| field.with_value(|value| matches!(value, ValueView::Structure(_))))
            })
            .unwrap_or(false)
    })
}

fn client_supplied_create_key_pair_domain_parameters_are_structures(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if operation != CREATE_KEY_PAIR_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| CREATE_KEY_PAIR_ATTRIBUTE_GROUPS.contains(&field.tag().raw()))
                    .all(|group| {
                        group.with_value(|value| match value {
                            ValueView::Structure(attributes) => attributes
                                .children()
                                .iter()
                                .filter(|attribute| {
                                    attribute.tag().raw() == CRYPTOGRAPHIC_DOMAIN_PARAMETERS
                                })
                                .all(|attribute| {
                                    attribute.with_value(|value| {
                                        matches!(value, ValueView::Structure(_))
                                    })
                                }),
                            _ => false,
                        })
                    })
            })
            .unwrap_or(false)
    })
}

fn protocol_version_is_kmip_2_1(version: &StructureView<'_>) -> bool {
    let mut major_seen = false;
    let mut minor_seen = false;
    let mut major = None;
    let mut minor = None;

    for field in version.children() {
        match field.tag().raw() {
            PROTOCOL_VERSION_MAJOR => {
                if major_seen {
                    return false;
                }
                major_seen = true;
                major = field.with_value(|value| match value {
                    ValueView::Integer(value) => Some(*value),
                    _ => None,
                });
            }
            PROTOCOL_VERSION_MINOR => {
                if minor_seen {
                    return false;
                }
                minor_seen = true;
                minor = field.with_value(|value| match value {
                    ValueView::Integer(value) => Some(*value),
                    _ => None,
                });
            }
            _ => {}
        }
    }

    major_seen && minor_seen && major == Some(2) && minor == Some(1)
}

const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.14-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.14-001",
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_supplied_create_key_pair_domain_parameters_are_structures),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_supplied_create_key_pair_domain_parameters",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/create_key_pair_execution_tests.rs::create_key_pair_sends_client_supplied_dsa_domain_parameter_qlength",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.16-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.16-001",
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(type_specific_parameters_do_not_infer_remote_object_type),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_allows_type_specific_cryptographic_parameters_without_remote_object_inference",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(discover_versions_advertises_only_client_supported_versions),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::production_discover_versions_rule_accepts_only_kmip_21_offers",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];

/// Redacted validation failure containing only stable IDs and a fixed code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileValidationError {
    code: ProfileValidationErrorCode,
    profile_id: Option<String>,
    requirement_id: Option<String>,
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

    /// Constructs a stable validation error for cross-crate client tests.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    #[must_use]
    pub fn for_test(
        code: ProfileValidationErrorCode,
        profile_id: &'static str,
        requirement_id: &'static str,
        source_clause_id: &'static str,
    ) -> Self {
        Self::for_requirement(code, profile_id, requirement_id, Some(source_clause_id))
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
        requirement_id: &str,
        source_clause_id: Option<&'static str>,
    ) -> Self {
        Self {
            code,
            profile_id: Some(profile_id.to_owned()),
            requirement_id: Some(requirement_id.to_owned()),
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
    pub fn requirement_id(&self) -> Option<&str> {
        self.requirement_id.as_deref()
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
        if let Some(requirement_id) = &self.requirement_id {
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
        if let Some(obligation) = profile.unresolved_scope_obligations.first() {
            return Err(ProfileValidationError::for_requirement(
                ProfileValidationErrorCode::UnresolvedRequirementScope,
                profile_id,
                obligation.requirement_id,
                Some(obligation.source_clause_id),
            ));
        }
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
    if mapping.constraints.iter().any(|constraint| {
        !matches!(
            (constraint.key, constraint.value),
            ("transport", "raw_tls" | "https") | ("encoding", "ttlv")
        )
    }) {
        return Err(error(ProfileValidationErrorCode::RuleMappingMismatch));
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
#[path = "../tests/support/profile_unit.rs"]
mod tests;
