//! Explicit, fail-closed client profile selection.
//!
//! Generated catalog records describe applicability and source obligations.
//! Only explicitly mapped rules with category-matched executable verification
//! are enforced; remaining unresolved or unmapped obligations keep a profile
//! unavailable for runtime validation. The profile prerequisite is KMIP
//! Specification v2.1 §14.1; Baseline Client's inherited profile closure
//! follows KMIP Profiles v2.1 §6.1.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ItemType, RawTag, StructureView, ValueView};

use crate::attribute::{ClientAttributeMutation, client_attribute_mutation_is_prohibited};
use crate::profiles_generated::{
    CLIENT_PROFILES, CLIENT_REQUIREMENTS, GeneratedClientProfile, GeneratedClientRequirement,
    GeneratedVerificationCategory,
};
use crate::{AttributeReference, CredentialType, RequestMessage, ResponseMessage};

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
    /// Caller state required by a selected profile rule is missing or unknown.
    ProfileStateContextRequired,
    /// Caller state cannot be bound uniquely to the request object identifier.
    ProfileStateBindingMismatch,
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
            Self::ProfileStateContextRequired => "profile_state_context_required",
            Self::ProfileStateBindingMismatch => "profile_state_binding_mismatch",
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

    fn satisfies_transport_security(self, required_security: &str) -> bool {
        matches!(
            (self, required_security),
            (Self::RawTtlvTls | Self::Https, "tls" | "tls13")
        )
    }

    fn satisfies_tls13_cipher_suite_policy(self, policy: &str) -> bool {
        matches!(
            (self, policy),
            (Self::RawTtlvTls | Self::Https, "kmip_profiles_3_1_2")
        )
    }
}

/// Explicit profile IDs requested for one client exchange.
#[derive(Clone, Eq, PartialEq)]
pub struct ProfileValidationSelection {
    profile_ids: Vec<String>,
}

/// Caller-attested state for one concrete KMIP object during one exchange.
///
/// Each fact is optional: `None` means unknown and profile validation fails
/// closed if that fact is needed. Values and the object identifier are redacted
/// from Debug output.
#[derive(Clone, Eq, PartialEq)]
pub struct ProfileObjectState {
    unique_identifier: String,
    original_creation_date_present: Option<bool>,
    usage_limits_present: Option<bool>,
    protection_use_started: Option<bool>,
    get_usage_allocation_performed: Option<bool>,
    get_usage_allocation_changed_count: Option<bool>,
    usage_limits_total: Option<i64>,
    usage_limits_count: Option<i64>,
}

impl ProfileObjectState {
    /// Creates state bound to an exact Text String Unique Identifier.
    #[must_use]
    pub fn new(unique_identifier: impl Into<String>) -> Self {
        Self {
            unique_identifier: unique_identifier.into(),
            original_creation_date_present: None,
            usage_limits_present: None,
            protection_use_started: None,
            get_usage_allocation_performed: None,
            get_usage_allocation_changed_count: None,
            usage_limits_total: None,
            usage_limits_count: None,
        }
    }

    /// Attests whether Original Creation Date is present; `None` is unknown.
    #[must_use]
    pub const fn with_original_creation_date_present(mut self, present: Option<bool>) -> Self {
        self.original_creation_date_present = present;
        self
    }

    /// Attests whether Usage Limits is present; `None` is unknown.
    #[must_use]
    pub const fn with_usage_limits_present(mut self, present: Option<bool>) -> Self {
        self.usage_limits_present = present;
        self
    }

    /// Attests whether protection use has begun; `None` is unknown.
    #[must_use]
    pub const fn with_protection_use_started(mut self, started: Option<bool>) -> Self {
        self.protection_use_started = started;
        self
    }

    /// Attests whether Get Usage Allocation has been performed; `None` is unknown.
    #[must_use]
    pub const fn with_get_usage_allocation_performed(mut self, performed: Option<bool>) -> Self {
        self.get_usage_allocation_performed = performed;
        self
    }

    /// Attests whether Get Usage Allocation changed Count; `None` is unknown.
    #[must_use]
    pub const fn with_get_usage_allocation_changed_count(mut self, changed: Option<bool>) -> Self {
        self.get_usage_allocation_changed_count = changed;
        self
    }

    /// Supplies the current Usage Limits Total when known.
    #[must_use]
    pub const fn with_usage_limits_total(mut self, total: Option<i64>) -> Self {
        self.usage_limits_total = total;
        self
    }

    /// Supplies the current Usage Limits Count when known.
    #[must_use]
    pub const fn with_usage_limits_count(mut self, count: Option<i64>) -> Self {
        self.usage_limits_count = count;
        self
    }
}

impl fmt::Debug for ProfileObjectState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProfileObjectState")
            .field("unique_identifier", &"[REDACTED]")
            .field("state", &"[REDACTED]")
            .finish()
    }
}

/// Caller-attested object state for one profile-validated exchange.
///
/// State is never cached, fetched, or inferred by `KMIPKit`. A context entry is
/// matched to the exact request Unique Identifier; missing or duplicate
/// entries fail closed when a rule needs them.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct ProfileValidationContext {
    object_states: Vec<ProfileObjectStateEntry>,
}

#[derive(Clone, Eq, PartialEq)]
struct ProfileObjectStateEntry {
    batch_item_index: usize,
    state: ProfileObjectState,
}

impl ProfileValidationContext {
    /// Creates an empty context. Any request requiring remote state fails closed.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            object_states: Vec::new(),
        }
    }

    /// Adds caller-attested state for one zero-based request batch item.
    #[must_use]
    pub fn with_object_state(mut self, batch_item_index: usize, state: ProfileObjectState) -> Self {
        self.object_states.push(ProfileObjectStateEntry {
            batch_item_index,
            state,
        });
        self
    }

    fn state_for(
        &self,
        batch_item_index: usize,
        unique_identifier: &str,
    ) -> Result<&ProfileObjectState, StateBinding> {
        let mut matching = self
            .object_states
            .iter()
            .filter(|entry| entry.batch_item_index == batch_item_index);
        let Some(entry) = matching.next() else {
            return Err(StateBinding::Missing);
        };
        if matching.next().is_some() {
            return Err(StateBinding::Duplicate);
        }
        if entry.state.unique_identifier != unique_identifier {
            return Err(StateBinding::Mismatch);
        }
        Ok(&entry.state)
    }
}

impl fmt::Debug for ProfileValidationContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProfileValidationContext")
            .field("object_state_count", &self.object_states.len())
            .field("object_states", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StateBinding {
    Missing,
    Duplicate,
    Mismatch,
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
    /// Returns a stable source-backed error when a selected rule requires an
    /// incompatible transport or its mapping is no longer present.
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
                let satisfied = match constraint.key {
                    "transport" => constraint.value == transport.as_rule_value(),
                    "transport_security" => {
                        transport.satisfies_transport_security(constraint.value)
                    }
                    "tls13_cipher_suites" => {
                        transport.satisfies_tls13_cipher_suite_policy(constraint.value)
                    }
                    _ => true,
                };
                if !satisfied {
                    return Err(ProfileValidationError::for_rule_requirement(
                        ProfileValidationErrorCode::TransportConstraintRejected,
                        profile_id,
                        requirement_id,
                        mapping.source_clause_id,
                        mapping.source_gap_id,
                    ));
                }
            }
        }
        Ok(())
    }

    /// Evaluates selected rules that can be observed in one outbound KMIP
    /// [`RequestMessage`].
    ///
    /// This convenience method uses an empty caller-state context and fails
    /// closed if the request requires remote object facts. Use
    /// [`Self::validate_request_with_context`] to supply those facts. An `Ok`
    /// result alone does not establish requirements that remain caller-owned.
    /// The typed
    /// `Client::execute_with_profiles` path validates operation-local request
    /// rules before an exchange, but it cannot infer remote object attributes
    /// or other context that the caller has not supplied. In particular, it
    /// does not confirm that IV/Counter/Nonce is present when the selected
    /// wrapping method requires it, or inspect whether opaque wrapped Key
    /// Value plaintext was TTLV encoded. The caller remains responsible for
    /// those KMIP Specification v2.1 §3.3, Tables 14–15 obligations. It also
    /// cannot enforce workflows spanning exchanges: for archived Locate
    /// results, the caller issues Recover and then Get after successful
    /// preceding results (KMIP Specification v2.1 §6.1.28). If Recover returns
    /// Pending, the caller may explicitly issue Poll with its correlation
    /// value under §6.1.42; this method does not poll implicitly.
    ///
    /// # Errors
    ///
    /// Returns a stable requirement and source-reference error when a mapped
    /// evaluator rejects the request or the validated registry is no longer
    /// complete.
    pub fn validate_request(&self, request: &RequestMessage) -> Result<(), ProfileValidationError> {
        self.validate_request_with_context(request, &ProfileValidationContext::new())
    }

    /// Evaluates the outbound request and rules that require caller-attested
    /// object state for this exchange.
    ///
    /// Context entries are matched to concrete Text String Unique Identifiers
    /// in their exact batch items. Missing/unknown facts fail closed, and no
    /// state is inferred from other items or exchanges.
    ///
    /// # Errors
    ///
    /// Returns a stable source-backed error for an invalid request, missing or
    /// mismatched state context, or a state-conditioned prohibited action.
    pub fn validate_request_with_context(
        &self,
        request: &RequestMessage,
        context: &ProfileValidationContext,
    ) -> Result<(), ProfileValidationError> {
        for requirement_id in &self.requirement_ids {
            if let Some(mapping) = CONTEXTUAL_RULE_MAPPINGS
                .iter()
                .find(|mapping| mapping.requirement_id == requirement_id)
            {
                let profile_id = self.profile_id_for_requirement(requirement_id);
                (mapping.evaluator)(request, context, profile_id)?;
            }
        }
        self.evaluate_rules(
            Some(request),
            None,
            TYPED_RULE_MAPPINGS,
            ProfileValidationErrorCode::RequestRuleRejected,
        )
    }

    /// Evaluates selected rules observable in a received response and its
    /// original [`RequestMessage`].
    ///
    /// As with [`Self::validate_request`], this standalone method cannot
    /// recover caller-owned context that is absent from TTLV. The typed client
    /// exchange path validates that context before sending the request. This
    /// method checks profile-specific rules and does not replace core protocol
    /// validation or request-response batch association.
    ///
    /// # Errors
    ///
    /// Returns a stable requirement and source-reference error when a mapped
    /// evaluator rejects the response or the validated registry is no longer
    /// complete.
    pub fn validate_response(
        &self,
        request: &RequestMessage,
        response: &ResponseMessage,
    ) -> Result<(), ProfileValidationError> {
        self.validate_response_with_context(request, response, &ProfileValidationContext::new())
    }

    /// Evaluates response behavior and the original request using the same
    /// caller-attested state supplied for the exchange.
    ///
    /// # Errors
    ///
    /// Returns a stable source-backed error when request context or a mapped
    /// response rule is invalid.
    pub fn validate_response_with_context(
        &self,
        request: &RequestMessage,
        response: &ResponseMessage,
        context: &ProfileValidationContext,
    ) -> Result<(), ProfileValidationError> {
        for requirement_id in &self.requirement_ids {
            if let Some(mapping) = CONTEXTUAL_RULE_MAPPINGS
                .iter()
                .find(|mapping| mapping.requirement_id == requirement_id)
            {
                let profile_id = self.profile_id_for_requirement(requirement_id);
                (mapping.evaluator)(request, context, profile_id)?;
            }
        }
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
                if is_configuration_only_mapping(mapping)
                    || is_contextual_rule_mapping(mapping.requirement_id)
                {
                    continue;
                }
                return Err(ProfileValidationError::for_rule_requirement(
                    ProfileValidationErrorCode::IncompleteRuleMapping,
                    profile_id,
                    requirement_id,
                    mapping.source_clause_id,
                    mapping.source_gap_id,
                ));
            };
            if !evaluator(request, response) {
                return Err(ProfileValidationError::for_rule_requirement(
                    rejection_code,
                    profile_id,
                    requirement_id,
                    mapping.source_clause_id,
                    mapping.source_gap_id,
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
/// assert!(readiness.runtime_validation_available());
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

fn profile_dependency_closure(
    profile_id: &'static str,
) -> (Vec<&'static GeneratedClientProfile>, BTreeSet<&'static str>) {
    let mut profiles = Vec::new();
    let mut visited_profile_ids = BTreeSet::new();
    let mut pending_profile_ids = vec![profile_id];
    let mut unresolved_dependency_ids = BTreeSet::new();
    while let Some(current_profile_id) = pending_profile_ids.pop() {
        if !visited_profile_ids.insert(current_profile_id) {
            continue;
        }
        let Some(current_profile) = find_profile(CLIENT_PROFILES, current_profile_id) else {
            unresolved_dependency_ids.insert(current_profile_id);
            continue;
        };
        profiles.push(current_profile);
        pending_profile_ids.extend(current_profile.dependency_profile_ids.iter().copied());
    }
    (profiles, unresolved_dependency_ids)
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
        let (profile_closure, unresolved_dependency_ids) =
            profile_dependency_closure(profile.profile_id);
        let required_ids: BTreeSet<_> = profile_closure
            .iter()
            .flat_map(|current_profile| current_profile.requirement_ids.iter().copied())
            .collect();
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
        for current_profile in &profile_closure {
            blockers.extend(current_profile.unresolved_scope_obligations.iter().map(
                |obligation| ProfileReadinessBlocker::UnresolvedRequirementScope {
                    requirement_id: obligation.requirement_id,
                    source_clause_id: obligation.source_clause_id,
                },
            ));
            if current_profile.profile_id != profile_id
                && current_profile.claim_state != "evidence_complete"
            {
                blockers.push(ProfileReadinessBlocker::DependencyEvidenceIncomplete {
                    profile_id: current_profile.profile_id,
                });
            }
            if current_profile.applicability == "conditional" {
                blockers.push(ProfileReadinessBlocker::ConditionalProfileUnsatisfied);
            } else if current_profile.applicability != "client_1_0" {
                blockers.push(ProfileReadinessBlocker::ProfileUnavailable);
            }
            blockers.extend(current_profile.test_cases.iter().filter_map(|test_case| {
                (test_case.mandatory_status == "mandatory"
                    && test_case.fixture_availability != "available")
                    .then_some(ProfileReadinessBlocker::MandatoryFixtureUnavailable {
                        test_id: test_case.test_id,
                    })
            }));
        }
        blockers.extend(unresolved_dependency_ids.into_iter().map(|profile_id| {
            ProfileReadinessBlocker::DependencyEvidenceIncomplete { profile_id }
        }));
        let mut requirement_records_complete = true;
        for requirement_id in &required_ids {
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
    source_gap_id: Option<&'static str>,
    verification_category: GeneratedVerificationCategory,
    evaluator: Option<RuleEvaluator>,
    test_evidence: &'static [RuleTestEvidence],
    constraints: &'static [RuleConstraint],
}

type ContextRuleEvaluator =
    fn(&RequestMessage, &ProfileValidationContext, &str) -> Result<(), ProfileValidationError>;

#[derive(Clone, Copy)]
struct ContextualRuleMapping {
    requirement_id: &'static str,
    evaluator: ContextRuleEvaluator,
}

const CONTEXTUAL_RULE_MAPPINGS: &[ContextualRuleMapping] = &[
    ContextualRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.38-003",
        evaluator: validate_original_creation_date_context,
    },
    ContextualRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.59-002",
        evaluator: validate_usage_limits_protection_context,
    },
    ContextualRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.59-003-002",
        evaluator: validate_usage_limits_count_context,
    },
    ContextualRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.59-003-001",
        evaluator: validate_usage_limits_allocation_context,
    },
];

fn is_contextual_rule_mapping(requirement_id: &str) -> bool {
    CONTEXTUAL_RULE_MAPPINGS
        .iter()
        .any(|mapping| mapping.requirement_id == requirement_id)
}

fn validate_original_creation_date_context(
    request: &RequestMessage,
    context: &ProfileValidationContext,
    profile_id: &str,
) -> Result<(), ProfileValidationError> {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.38-003";
    const CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-4.38-003";

    for (index, batch_item) in request.batch_items().enumerate() {
        let Some(operation) = batch_item.operation() else {
            continue;
        };
        if !matches!(
            operation,
            ADD_ATTRIBUTE_OPERATION
                | ADJUST_ATTRIBUTE_OPERATION
                | DELETE_ATTRIBUTE_OPERATION
                | MODIFY_ATTRIBUTE_OPERATION
                | SET_ATTRIBUTE_OPERATION
        ) {
            continue;
        }

        let targets_attribute = batch_item.with_request_payload(|payload| {
            payload_targets_attribute(&payload, operation, ORIGINAL_CREATION_DATE)
        });
        let Some(Ok(true)) = targets_attribute else {
            if matches!(targets_attribute, Some(Err(()))) {
                return Err(context_rule_error(
                    ProfileValidationErrorCode::RequestRuleRejected,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                ));
            }
            continue;
        };

        let result = batch_item.with_request_payload(|payload| {
            let state = bound_object_state(
                &payload,
                context,
                profile_id,
                REQUIREMENT_ID,
                CLAUSE_ID,
                index,
            )?;
            match state.original_creation_date_present {
                Some(false) if operation == ADD_ATTRIBUTE_OPERATION => Ok(()),
                Some(true) if operation == ADD_ATTRIBUTE_OPERATION => Err(context_rule_error(
                    ProfileValidationErrorCode::RequestRuleRejected,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                )),
                Some(true | false) => Err(context_rule_error(
                    ProfileValidationErrorCode::RequestRuleRejected,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                )),
                None => Err(context_rule_error(
                    ProfileValidationErrorCode::ProfileStateContextRequired,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                )),
            }
        });
        match result {
            Some(result) => result?,
            None => {
                return Err(context_rule_error(
                    ProfileValidationErrorCode::ProfileStateContextRequired,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                ));
            }
        }
    }
    Ok(())
}

fn validate_usage_limits_protection_context(
    request: &RequestMessage,
    context: &ProfileValidationContext,
    profile_id: &str,
) -> Result<(), ProfileValidationError> {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.59-002";
    const CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-4.59-002";

    for (index, batch_item) in request.batch_items().enumerate() {
        let Some(operation) = batch_item.operation() else {
            continue;
        };
        let result = batch_item.with_request_payload(|payload| {
            let requested = request_usage_total_mutation(&payload, operation).map_err(|()| {
                context_rule_error(
                    ProfileValidationErrorCode::RequestRuleRejected,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                )
            })?;
            let Some(requested) = requested else {
                return Ok(());
            };
            let state = bound_object_state(
                &payload,
                context,
                profile_id,
                REQUIREMENT_ID,
                CLAUSE_ID,
                index,
            )?;
            let changed = usage_total_is_changed(requested, operation, state).map_err(|code| {
                context_rule_error(code, profile_id, REQUIREMENT_ID, CLAUSE_ID, index)
            })?;
            if !changed {
                return Ok(());
            }
            let protection_started = state.protection_use_started.ok_or_else(|| {
                context_rule_error(
                    ProfileValidationErrorCode::ProfileStateContextRequired,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                )
            })?;
            let allocation_performed = state.get_usage_allocation_performed.ok_or_else(|| {
                context_rule_error(
                    ProfileValidationErrorCode::ProfileStateContextRequired,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                )
            })?;
            if allocation_performed && state.get_usage_allocation_changed_count == Some(true) {
                // The narrower paragraph-3 rule owns this exact trigger.
                return Ok(());
            }
            if protection_started || allocation_performed {
                return Err(context_rule_error(
                    ProfileValidationErrorCode::RequestRuleRejected,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                ));
            }
            Ok(())
        });
        match result {
            Some(result) => result?,
            None => {
                return Err(context_rule_error(
                    ProfileValidationErrorCode::ProfileStateContextRequired,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                ));
            }
        }
    }
    Ok(())
}

fn validate_usage_limits_count_context(
    request: &RequestMessage,
    context: &ProfileValidationContext,
    profile_id: &str,
) -> Result<(), ProfileValidationError> {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.59-003-002";
    const CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-4.59-003";

    for (index, batch_item) in request.batch_items().enumerate() {
        if batch_item.operation() != Some(SET_ATTRIBUTE_OPERATION) {
            continue;
        }
        let result = batch_item.with_request_payload(|payload| {
            let requested_count =
                usage_limits_count_in_wrapper(&payload, NEW_ATTRIBUTE).map_err(|()| {
                    context_rule_error(
                        ProfileValidationErrorCode::RequestRuleRejected,
                        profile_id,
                        REQUIREMENT_ID,
                        CLAUSE_ID,
                        index,
                    )
                })?;
            let Some(requested_count) = requested_count else {
                return Ok(());
            };
            let state = bound_object_state(
                &payload,
                context,
                profile_id,
                REQUIREMENT_ID,
                CLAUSE_ID,
                index,
            )?;
            if state.usage_limits_present != Some(true) {
                return Err(context_rule_error(
                    ProfileValidationErrorCode::ProfileStateContextRequired,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                ));
            }
            let current_count = state.usage_limits_count.ok_or_else(|| {
                context_rule_error(
                    ProfileValidationErrorCode::ProfileStateContextRequired,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                )
            })?;
            if requested_count != current_count {
                return Err(context_rule_error(
                    ProfileValidationErrorCode::RequestRuleRejected,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                ));
            }
            Ok(())
        });
        match result {
            Some(result) => result?,
            None => {
                return Err(context_rule_error(
                    ProfileValidationErrorCode::ProfileStateContextRequired,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                ));
            }
        }
    }
    Ok(())
}

fn validate_usage_limits_allocation_context(
    request: &RequestMessage,
    context: &ProfileValidationContext,
    profile_id: &str,
) -> Result<(), ProfileValidationError> {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.59-003-001";
    const CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-4.59-003";

    for (index, batch_item) in request.batch_items().enumerate() {
        let Some(operation) = batch_item.operation() else {
            continue;
        };
        let result = batch_item.with_request_payload(|payload| {
            let requested = request_usage_total_mutation(&payload, operation).map_err(|()| {
                context_rule_error(
                    ProfileValidationErrorCode::RequestRuleRejected,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                )
            })?;
            let Some(requested) = requested else {
                return Ok(());
            };
            let state = bound_object_state(
                &payload,
                context,
                profile_id,
                REQUIREMENT_ID,
                CLAUSE_ID,
                index,
            )?;
            let changed = usage_total_is_changed(requested, operation, state).map_err(|code| {
                context_rule_error(code, profile_id, REQUIREMENT_ID, CLAUSE_ID, index)
            })?;
            if !changed {
                return Ok(());
            }
            if state.get_usage_allocation_changed_count == Some(true) {
                if state.get_usage_allocation_performed != Some(true) {
                    return Err(context_rule_error(
                        ProfileValidationErrorCode::ProfileStateBindingMismatch,
                        profile_id,
                        REQUIREMENT_ID,
                        CLAUSE_ID,
                        index,
                    ));
                }
                return Err(context_rule_error(
                    ProfileValidationErrorCode::RequestRuleRejected,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                ));
            }
            Ok(())
        });
        match result {
            Some(result) => result?,
            None => {
                return Err(context_rule_error(
                    ProfileValidationErrorCode::ProfileStateContextRequired,
                    profile_id,
                    REQUIREMENT_ID,
                    CLAUSE_ID,
                    index,
                ));
            }
        }
    }
    Ok(())
}

fn context_rule_error(
    code: ProfileValidationErrorCode,
    profile_id: &str,
    requirement_id: &str,
    clause_id: &'static str,
    item_index: usize,
) -> ProfileValidationError {
    ProfileValidationError::for_rule_requirement(code, profile_id, requirement_id, clause_id, None)
        .with_batch_item_index(item_index)
}

fn bound_object_state<'a>(
    payload: &StructureView<'_>,
    context: &'a ProfileValidationContext,
    profile_id: &str,
    requirement_id: &str,
    clause_id: &'static str,
    item_index: usize,
) -> Result<&'a ProfileObjectState, ProfileValidationError> {
    let unique_identifier = match field_presence(payload, UNIQUE_IDENTIFIER) {
        FieldPresence::Unique(item) => item.with_value(|value| match value {
            ValueView::TextString(value) if !value.is_empty() => Some(value.to_owned()),
            _ => None,
        }),
        FieldPresence::Missing | FieldPresence::Duplicate => None,
    };
    let Some(unique_identifier) = unique_identifier else {
        return Err(context_rule_error(
            ProfileValidationErrorCode::ProfileStateBindingMismatch,
            profile_id,
            requirement_id,
            clause_id,
            item_index,
        ));
    };
    context
        .state_for(item_index, &unique_identifier)
        .map_err(|binding| {
            let code = match binding {
                StateBinding::Missing => ProfileValidationErrorCode::ProfileStateContextRequired,
                StateBinding::Duplicate | StateBinding::Mismatch => {
                    ProfileValidationErrorCode::ProfileStateBindingMismatch
                }
            };
            context_rule_error(code, profile_id, requirement_id, clause_id, item_index)
        })
}

fn payload_targets_attribute(
    payload: &StructureView<'_>,
    operation: u32,
    attribute_tag: u32,
) -> Result<bool, ()> {
    match operation {
        ADD_ATTRIBUTE_OPERATION | SET_ATTRIBUTE_OPERATION => {
            attribute_wrapper_targets(payload, NEW_ATTRIBUTE, attribute_tag)
        }
        MODIFY_ATTRIBUTE_OPERATION => {
            Ok(
                attribute_wrapper_targets(payload, CURRENT_ATTRIBUTE, attribute_tag)?
                    || attribute_wrapper_targets(payload, NEW_ATTRIBUTE, attribute_tag)?,
            )
        }
        DELETE_ATTRIBUTE_OPERATION => {
            Ok(
                attribute_wrapper_targets(payload, CURRENT_ATTRIBUTE, attribute_tag)?
                    || reference_targets_attribute(payload, attribute_tag)?,
            )
        }
        ADJUST_ATTRIBUTE_OPERATION => reference_targets_attribute(payload, attribute_tag),
        _ => Ok(false),
    }
}

fn attribute_wrapper_targets(
    payload: &StructureView<'_>,
    wrapper_tag: u32,
    attribute_tag: u32,
) -> Result<bool, ()> {
    match field_presence(payload, wrapper_tag) {
        FieldPresence::Missing => Ok(false),
        FieldPresence::Duplicate => Err(()),
        FieldPresence::Unique(wrapper) => wrapper.with_value(|value| match value {
            ValueView::Structure(attributes) => Ok(attributes
                .children()
                .iter()
                .any(|attribute| attribute.tag().raw() == attribute_tag)),
            _ => Err(()),
        }),
    }
}

fn reference_targets_attribute(
    payload: &StructureView<'_>,
    attribute_tag: u32,
) -> Result<bool, ()> {
    match field_presence(payload, ATTRIBUTE_REFERENCE) {
        FieldPresence::Missing => Ok(false),
        FieldPresence::Duplicate => Err(()),
        FieldPresence::Unique(reference) => {
            let reference = AttributeReference::try_from_ttlv_item(reference).map_err(|_| ())?;
            if let Some(tag) = reference.tag_value() {
                return Ok(tag == attribute_tag);
            }
            let Some((vendor, name)) = reference.name_parts() else {
                return Err(());
            };
            Ok(vendor.is_empty()
                && matches!(
                    (attribute_tag, name),
                    (ORIGINAL_CREATION_DATE, "Original Creation Date")
                        | (USAGE_LIMITS_ATTRIBUTE, "Usage Limits")
                ))
        }
    }
}

#[derive(Clone, Copy)]
enum UsageTotalMutation {
    Set(i64),
    Remove,
    Indeterminate,
}

fn request_usage_total_mutation(
    payload: &StructureView<'_>,
    operation: u32,
) -> Result<Option<UsageTotalMutation>, ()> {
    match operation {
        DELETE_ATTRIBUTE_OPERATION => {
            if attribute_wrapper_targets(payload, CURRENT_ATTRIBUTE, USAGE_LIMITS_ATTRIBUTE)?
                || reference_targets_attribute(payload, USAGE_LIMITS_ATTRIBUTE)?
            {
                Ok(Some(UsageTotalMutation::Remove))
            } else {
                Ok(None)
            }
        }
        ADJUST_ATTRIBUTE_OPERATION => {
            if reference_targets_attribute(payload, USAGE_LIMITS_ATTRIBUTE)? {
                Ok(Some(UsageTotalMutation::Indeterminate))
            } else {
                Ok(None)
            }
        }
        ADD_ATTRIBUTE_OPERATION | MODIFY_ATTRIBUTE_OPERATION | SET_ATTRIBUTE_OPERATION => {
            let new_total = usage_total_in_wrapper(payload, NEW_ATTRIBUTE)?;
            if new_total.is_some() {
                return Ok(new_total);
            }
            if operation == MODIFY_ATTRIBUTE_OPERATION
                && attribute_wrapper_targets(payload, CURRENT_ATTRIBUTE, USAGE_LIMITS_ATTRIBUTE)?
            {
                return Ok(Some(UsageTotalMutation::Remove));
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn usage_total_in_wrapper(
    payload: &StructureView<'_>,
    wrapper_tag: u32,
) -> Result<Option<UsageTotalMutation>, ()> {
    let wrapper = match field_presence(payload, wrapper_tag) {
        FieldPresence::Missing => return Ok(None),
        FieldPresence::Duplicate => return Err(()),
        FieldPresence::Unique(wrapper) => wrapper,
    };
    wrapper.with_value(|value| match value {
        ValueView::Structure(attributes) => {
            let mut matching = attributes
                .children()
                .iter()
                .filter(|attribute| attribute.tag().raw() == USAGE_LIMITS_ATTRIBUTE);
            let Some(usage_limits) = matching.next() else {
                return Ok(None);
            };
            if matching.next().is_some() {
                return Err(());
            }
            usage_limits.with_value(|value| match value {
                ValueView::Structure(fields) => {
                    let FieldPresence::Unique(total) = field_presence(&fields, USAGE_LIMITS_TOTAL)
                    else {
                        return Err(());
                    };
                    total.with_value(|value| match value {
                        ValueView::LongInteger(total) => Ok(Some(UsageTotalMutation::Set(*total))),
                        _ => Err(()),
                    })
                }
                _ => Err(()),
            })
        }
        _ => Err(()),
    })
}

fn usage_limits_count_in_wrapper(
    payload: &StructureView<'_>,
    wrapper_tag: u32,
) -> Result<Option<i64>, ()> {
    let wrapper = match field_presence(payload, wrapper_tag) {
        FieldPresence::Missing => return Ok(None),
        FieldPresence::Duplicate => return Err(()),
        FieldPresence::Unique(wrapper) => wrapper,
    };
    wrapper.with_value(|value| match value {
        ValueView::Structure(attributes) => {
            let mut matching = attributes
                .children()
                .iter()
                .filter(|attribute| attribute.tag().raw() == USAGE_LIMITS_ATTRIBUTE);
            let Some(usage_limits) = matching.next() else {
                return Ok(None);
            };
            if matching.next().is_some() {
                return Err(());
            }
            usage_limits.with_value(|value| match value {
                ValueView::Structure(fields) => {
                    if !usage_limits_structure_is_well_formed(&fields) {
                        return Err(());
                    }
                    match field_presence(&fields, USAGE_LIMITS_COUNT) {
                        FieldPresence::Unique(count) => count.with_value(|value| match value {
                            ValueView::LongInteger(count) => Ok(Some(*count)),
                            _ => Err(()),
                        }),
                        FieldPresence::Missing | FieldPresence::Duplicate => Err(()),
                    }
                }
                _ => Err(()),
            })
        }
        _ => Err(()),
    })
}

fn usage_total_is_changed(
    requested: UsageTotalMutation,
    operation: u32,
    state: &ProfileObjectState,
) -> Result<bool, ProfileValidationErrorCode> {
    match requested {
        UsageTotalMutation::Indeterminate => {
            Err(ProfileValidationErrorCode::ProfileStateContextRequired)
        }
        UsageTotalMutation::Remove => match state.usage_limits_present {
            Some(true) => Ok(true),
            Some(false) => Ok(false),
            None => Err(ProfileValidationErrorCode::ProfileStateContextRequired),
        },
        UsageTotalMutation::Set(total) => match state.usage_limits_present {
            Some(false) if operation == ADD_ATTRIBUTE_OPERATION => Ok(false),
            Some(false) | None => Err(ProfileValidationErrorCode::ProfileStateContextRequired),
            Some(true) => {
                let current_total = state
                    .usage_limits_total
                    .ok_or(ProfileValidationErrorCode::ProfileStateContextRequired)?;
                Ok(total != current_total)
            }
        },
    }
}

const DISCOVER_VERSIONS_OPERATION: u32 = 0x0000_001E;
const CERTIFY_OPERATION: u32 = 0x0000_0006;
const CREATE_OPERATION: u32 = 0x0000_0001;
const ARCHIVE_OPERATION: u32 = 0x0000_0015;
const REKEY_OPERATION: u32 = 0x0000_0004;
const RECERTIFY_OPERATION: u32 = 0x0000_0007;
const REPROVISION_OPERATION: u32 = 0x0000_0035;
const REKEY_KEY_PAIR_OPERATION: u32 = 0x0000_001D;
const SIGN_OPERATION: u32 = 0x0000_0021;
const SIGNATURE_VERIFY_OPERATION: u32 = 0x0000_0022;
const SIGNATURE_VERIFY_DATA: u32 = 0x0042_00C2;
const RECOVER_OPERATION: u32 = 0x0000_0016;
const CREATE_SPLIT_KEY_OPERATION: u32 = 0x0000_0028;
const JOIN_SPLIT_KEY_OPERATION: u32 = 0x0000_0029;
const JOIN_SPLIT_KEY_OBJECT_TYPE_TAG: u32 = 0x0042_0057;
const CHECK_OPERATION: u32 = 0x0000_0009;
const CHECK_STANDARD_REQUEST_MEMBER_TAGS: [u32; 4] = [
    UNIQUE_IDENTIFIER,
    USAGE_LIMITS_COUNT,
    CRYPTOGRAPHIC_USAGE_MASK,
    LEASE_TIME,
];
const GET_USAGE_ALLOCATION_OPERATION: u32 = 0x0000_0011;
const LOGIN_OPERATION: u32 = 0x0000_002D;
const LOGIN_REQUEST_COUNT: u32 = 0x0042_014C;
const LOGIN_USAGE_LIMITS: u32 = 0x0042_0095;
const CREDENTIAL: u32 = 0x0042_0023;
const CREDENTIAL_VALUE: u32 = 0x0042_0025;
const ATTESTATION_CREDENTIAL_TYPE: u32 = 3;
const DEVICE_CREDENTIAL_PASSWORD: u32 = 0x0042_00A1;
const DELEGATED_LOGIN_OPERATION: u32 = 0x0000_002F;
const LOGOUT_OPERATION: u32 = 0x0000_002E;
const REPROVISION_CERTIFICATE: u32 = 0x0042_0013;
const REPROVISION_CERTIFICATE_REQUEST: u32 = 0x0042_0018;
const REPROVISION_CERTIFICATE_TYPE: u32 = 0x0042_001D;
const REPROVISION_CERTIFICATE_VALUE: u32 = 0x0042_001E;
const REPROVISION_CERTIFICATE_REQUEST_TYPE: u32 = 0x0042_0019;
const REPROVISION_CERTIFICATE_REQUEST_VALUE: u32 = 0x0042_0140;
const SPLIT_KEY_METHOD: u32 = 0x0042_008A;
const PRIME_FIELD_SIZE: u32 = 0x0042_0062;
const POLYNOMIAL_SHARING_PRIME_FIELD: u32 = 3;
const CERTIFICATE_REQUEST_TYPE: u32 = 0x0042_0019;
const CERTIFICATE_REQUEST_VALUE: u32 = 0x0042_0140;
const CERTIFY_UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const SPECIFICATION_TAG_PREFIX: u32 = 0x42;
const EXTENSION_TAG_PREFIX: u32 = 0x54;
const PKCS12_FRIENDLY_NAME_ATTRIBUTE: u32 = 0x0042_00FB;
const VENDOR_EXTENSION: u32 = 0x0042_009C;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const IV_COUNTER_NONCE: u32 = 0x0042_003D;
const MAC_VERIFY_ORIGINAL_DATA: u32 = 0x0042_00C2;
const USAGE_LIMITS_COUNT: u32 = 0x0042_0096;
const USAGE_LIMITS_TOTAL: u32 = 0x0042_0097;
const USAGE_LIMITS_UNIT: u32 = 0x0042_0098;
const USAGE_LIMITS_ATTRIBUTE: u32 = 0x0042_0095;
const LEASE_TIME: u32 = 0x0042_0049;
const TICKET: u32 = 0x0042_0149;
const OFFSET: u32 = 0x0042_0058;
const SIGN_CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const DELEGATED_LOGIN_RIGHTS: u32 = 0x0042_014D;
const DELEGATED_LOGIN_RIGHT: u32 = 0x0042_014E;
const DELEGATED_LOGIN_REQUEST_COUNT: u32 = 0x0042_014C;
const DELEGATED_LOGIN_USAGE_LIMITS: u32 = 0x0042_0095;
const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
const CTR_MODE: u32 = 6;
const CCM_MODE: u32 = 8;
const TAG_LENGTH: u32 = 0x0042_00CE;
const IV_LENGTH: u32 = 0x0042_00CD;
const GCM_MODE: u32 = 9;
const IMPORT_OPERATION: u32 = 0x0000_002A;
const IMPORT_OBJECT_TYPE: u32 = 0x0042_0057;
const DERIVE_KEY_OBJECT_TYPE: u32 = 0x0042_0057;
const DERIVE_KEY_DERIVATION_METHOD: u32 = 0x0042_0031;
const DERIVE_KEY_DERIVATION_PARAMETERS: u32 = 0x0042_0032;
const DERIVE_KEY_HASH_METHOD: u32 = 2;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const DERIVATION_DATA: u32 = 0x0042_0030;
const ASYNCHRONOUS_INDICATOR_MANDATORY: u32 = 1;
const ASYNCHRONOUS_INDICATOR_PROHIBITED: u32 = 3;
const ENUMERATION_EXTENSION_MIN: u32 = 0x8000_0000;
const ENUMERATION_EXTENSION_MAX: u32 = 0x8FFF_FFFF;
const SYMMETRIC_KEY_OBJECT_TYPE: u32 = 2;
const PGP_KEY_OBJECT_TYPE: u32 = 9;
const SECRET_DATA_OBJECT_TYPE: u32 = 7;
const SECRET_DATA: u32 = 0x0042_0085;
const SECRET_DATA_TYPE: u32 = 0x0042_0086;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_MATERIAL: u32 = 0x0042_0043;
const KEY_VALUE: u32 = 0x0042_0045;
const PGP_KEY: u32 = 0x0042_00BD;
const PGP_KEY_VERSION: u32 = 0x0042_00BE;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const PUBLIC_KEY: u32 = 0x0042_006D;
const PRIVATE_KEY: u32 = 0x0042_0064;
const SPLIT_KEY: u32 = 0x0042_0089;
const SPLIT_KEY_PARTS: u32 = 0x0042_008B;
const KEY_PART_IDENTIFIER: u32 = 0x0042_0044;
const KEY_WRAPPING_DATA: u32 = 0x0042_0046;
const RSA_PRIVATE_KEY_FORMAT: u32 = 10;
const RSA_PRIME_P: u32 = 0x0042_005E;
const RSA_PRIME_EXPONENT_P: u32 = 0x0042_0060;
const RSA_PRIME_EXPONENT_Q: u32 = 0x0042_0061;
const RSA_PRIVATE_EXPONENT: u32 = 0x0042_0063;
const RSA_PRIME_Q: u32 = 0x0042_0071;
const KEY_WRAPPING_SPECIFICATION: u32 = 0x0042_0047;
const WRAPPING_METHOD: u32 = 0x0042_009E;
const ENCRYPTION_KEY_INFORMATION: u32 = 0x0042_0036;
const NO_ENCODING_OPTION: u32 = 1;
const GET_ATTRIBUTES_OPERATION: u32 = 0x0000_000B;
const OBTAIN_LEASE_OPERATION: u32 = 0x0000_0010;
const RESULT_STATUS_SUCCESS: u32 = 0;
const ATTRIBUTE_REFERENCE: u32 = 0x0042_013B;
const ADD_ATTRIBUTE_OPERATION: u32 = 0x0000_000D;
const MODIFY_ATTRIBUTE_OPERATION: u32 = 0x0000_000E;
const DELETE_ATTRIBUTE_OPERATION: u32 = 0x0000_000F;
const ADJUST_ATTRIBUTE_OPERATION: u32 = 0x0000_0030;
const SET_ATTRIBUTE_OPERATION: u32 = 0x0000_0031;
const CURRENT_ATTRIBUTE: u32 = 0x0042_013C;
const NEW_ATTRIBUTE: u32 = 0x0042_013D;
const STATE_ATTRIBUTE: u32 = 0x0042_008D;
const APPLICATION_DATA: u32 = 0x0042_0002;
const APPLICATION_NAMESPACE: u32 = 0x0042_0003;
const APPLICATION_SPECIFIC_INFORMATION: u32 = 0x0042_0004;
const LOCATE_OPERATION: u32 = 0x0000_0008;
const REVOKE_OPERATION: u32 = 0x0000_0013;
const VALIDATE_OPERATION: u32 = 0x0000_0017;
const VALIDITY_DATE: u32 = 0x0042_009A;
const VALIDATE_CERTIFICATE: u32 = 0x0042_0013;
const REVOCATION_REASON: u32 = 0x0042_0081;
const REVOCATION_REASON_CODE: u32 = 0x0042_0082;
const REVOCATION_MESSAGE: u32 = 0x0042_0080;
const COMPROMISE_OCCURRENCE_DATE: u32 = 0x0042_0021;
const KEY_COMPROMISE_REASON: u32 = 2;
const CA_COMPROMISE_REASON: u32 = 3;
const MAXIMUM_ITEMS: u32 = 0x0042_004F;
const OFFSET_ITEMS: u32 = 0x0042_00D4;
const QUERY_OPERATION: u32 = 0x0000_0018;
const QUERY_FUNCTION: u32 = 0x0042_0074;
const PROFILE_INFORMATION: u32 = 0x0042_00EB;
const ATTRIBUTES: u32 = 0x0042_0125;
const CERTIFICATE_TYPE_ATTRIBUTE: u32 = 0x0042_001D;
const LINK_ATTRIBUTE: u32 = 0x0042_004A;
const LINK_TYPE: u32 = 0x0042_004B;
const LINKED_OBJECT_IDENTIFIER: u32 = 0x0042_004C;
const ISSUER_ALTERNATIVE_NAME: u32 = 0x0042_00B1;
const SUBJECT_ALTERNATIVE_NAME: u32 = 0x0042_00B3;
const SUBJECT_DISTINGUISHED_NAME: u32 = 0x0042_00B4;
const MAC_SIGNATURE_KEY_INFORMATION: u32 = 0x0042_004E;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ID_PLACEHOLDER_ENUMERATION_VALUE: u32 = 1;
const ENCODING_OPTION: u32 = 0x0042_00A3;
const CREATE_KEY_PAIR_OPERATION: u32 = 0x0000_0002;
const RNG_SEED_OPERATION: u32 = 0x0000_0026;
const RNG_SEED_DATA_LENGTH: u32 = 0x0042_00D5;
const DERIVE_KEY_OPERATION: u32 = 0x0000_0005;
const CREATE_KEY_PAIR_ATTRIBUTE_GROUPS: [u32; 3] = [0x0042_0126, 0x0042_0127, 0x0042_0128];
const CREATE_KEY_PAIR_COMMON_ATTRIBUTES: u32 = 0x0042_0126;
const CREATE_KEY_PAIR_PRIVATE_KEY_ATTRIBUTES: u32 = 0x0042_0127;
const CREATE_KEY_PAIR_PUBLIC_KEY_ATTRIBUTES: u32 = 0x0042_0128;
const CREATE_KEY_PAIR_TABLE_191_ATTRIBUTES: [u32; 4] = [
    0x0042_0028, // Cryptographic Algorithm
    0x0042_002a, // Cryptographic Length
    0x0042_0029, // Cryptographic Domain Parameters
    0x0042_002b, // Cryptographic Parameters
];
const CRYPTOGRAPHIC_DOMAIN_PARAMETERS: u32 = 0x0042_0029;
const QLENGTH_DOMAIN_PARAMETER: u32 = 0x0042_0073;
const REGISTER_OPERATION: u32 = 0x0000_0003;
const POLL_OPERATION: u32 = 0x0000_001A;
const CANCEL_OPERATION: u32 = 0x0000_0019;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const GET_OPERATION: u32 = 0x0000_000A;
const EXPORT_OPERATION: u32 = 0x0000_002B;
const REGISTER_ATTRIBUTES: u32 = 0x0042_0125;
const NIST_KEY_TYPE_ATTRIBUTE: u32 = 0x0042_013A;
const REGISTER_OBJECT_TYPE: u32 = 0x0042_0057;
const CERTIFICATE_OBJECT_TYPE: u32 = 1;
const REGISTER_OPAQUE_OBJECT: u32 = 0x0042_005B;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_VALUE: u32 = 0x0042_001E;
const DIGITAL_SIGNATURE_ALGORITHM: u32 = 0x0042_00AE;
const OPAQUE_DATA_TYPE: u32 = 0x0042_0059;
const OPAQUE_DATA_VALUE: u32 = 0x0042_005A;
const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
const RANDOM_NUMBER_GENERATOR: u32 = 0x0042_00DE;
const RNG_PARAMETERS: u32 = 0x0042_00D9;
const RNG_ALGORITHM: u32 = 0x0042_00DA;
const RNG_HASHING_ALGORITHM: u32 = 0x0042_0038;
const RNG_RECOMMENDED_CURVE: u32 = 0x0042_0075;
const RNG_DRBG_ALGORITHM: u32 = 0x0042_00DB;
const RNG_FIPS186_VARIATION: u32 = 0x0042_00DC;
const RNG_PREDICTION_RESISTANCE: u32 = 0x0042_00DD;
const REGISTER_PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const STORAGE_STATUS_MASK: u32 = 0x0042_008E;
const PROTECTION_STORAGE_MASK: u32 = 0x0042_015E;
const PUBLIC_KEY_OBJECT_TYPE: u32 = 3;
const PRIVATE_KEY_OBJECT_TYPE: u32 = 4;
const SPLIT_KEY_OBJECT_TYPE: u32 = 5;
const OPAQUE_OBJECT_TYPE: u32 = 8;
const CERTIFICATE_REQUEST_OBJECT_TYPE: u32 = 10;
const RESERVED_OBJECT_TYPE_6: u32 = 6;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const VENDOR_ATTRIBUTE: u32 = 0x0042_0008;
const GET_KEY_COMPRESSION_TYPE: u32 = 0x0042_0041;
const GET_KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const GET_KEY_WRAP_TYPE: u32 = 0x0042_00F8;
const EXPORT_KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const EXPORT_KEY_WRAP_TYPE: u32 = 0x0042_00F8;
const KEY_VALUE_LOCATION: u32 = 0x0042_00B8;
const KEY_VALUE_LOCATION_VALUE: u32 = 0x0042_00B9;
const KEY_VALUE_LOCATION_TYPE: u32 = 0x0042_00BA;
const KEY_VALUE_PRESENT: u32 = 0x0042_00BB;
const BATCH_ERROR_CONTINUATION_OPTION_CONTINUE: u32 = 1;
const BATCH_ERROR_CONTINUATION_OPTION_STOP: u32 = 2;
const BATCH_ERROR_CONTINUATION_OPTION_UNDO: u32 = 3;

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

fn client_certify_requests_at_most_one_certificate(
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
        if operation != CERTIFY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == CERTIFICATE_REQUEST_VALUE)
                    .count()
                    <= 1
            })
            .unwrap_or(false)
    })
}

fn client_recertify_requests_at_most_one_certificate(
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
        if operation != RECERTIFY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == UNIQUE_IDENTIFIER)
                    .count()
                    <= 1
            })
            .unwrap_or(false)
    })
}

fn client_recertify_identifier_required_when_request_value_is_omitted(
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
        if operation != RECERTIFY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let fields = payload.children();
                let has_request_value = fields
                    .iter()
                    .any(|field| field.tag().raw() == CERTIFICATE_REQUEST_VALUE);
                if has_request_value {
                    return true;
                }

                let mut identifiers = fields
                    .iter()
                    .filter(|field| field.tag().raw() == UNIQUE_IDENTIFIER);
                let Some(identifier) = identifiers.next() else {
                    return false;
                };
                if identifiers.next().is_some() {
                    return false;
                }

                !identifier.with_value(|value| {
                    matches!(
                        value,
                        ValueView::Enumeration(value)
                            if *value == ID_PLACEHOLDER_ENUMERATION_VALUE
                    )
                })
            })
            .unwrap_or(false)
    })
}

fn client_certify_certificate_request_value_is_omittable(
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
        if operation != CERTIFY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == CERTIFICATE_REQUEST_VALUE)
                    .all(|field| {
                        field.with_value(|value| matches!(value, ValueView::ByteString(_)))
                    })
            })
            .unwrap_or(false)
    })
}

fn client_can_retrieve_certified_certificate_with_get_in_batch(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    let batch_ordered = request.header().effective_batch_order_option();
    let mut preceding_placeholder_producer = false;
    for batch_item in request.batch_items() {
        let Some(operation) = batch_item.operation() else {
            return false;
        };

        if operation == GET_OPERATION {
            let Some(uses_placeholder) = batch_item.with_request_payload(|payload| {
                let mut has_identifier = false;
                let mut selects_placeholder = false;
                for field in payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == UNIQUE_IDENTIFIER)
                {
                    has_identifier = true;
                    selects_placeholder |= field.with_value(|value| {
                        matches!(
                            value,
                            ValueView::Enumeration(value)
                                if *value == ID_PLACEHOLDER_ENUMERATION_VALUE
                        )
                    });
                }
                !has_identifier || selects_placeholder
            }) else {
                return false;
            };

            if uses_placeholder && (!batch_ordered || !preceding_placeholder_producer) {
                return false;
            }
        }

        if matches!(
            operation,
            CREATE_OPERATION
                | CREATE_KEY_PAIR_OPERATION
                | REGISTER_OPERATION
                | REKEY_OPERATION
                | DERIVE_KEY_OPERATION
                | CERTIFY_OPERATION
                | RECERTIFY_OPERATION
                | LOCATE_OPERATION
                | RECOVER_OPERATION
                | REKEY_KEY_PAIR_OPERATION
                | CREATE_SPLIT_KEY_OPERATION
                | JOIN_SPLIT_KEY_OPERATION
                | IMPORT_OPERATION
                | EXPORT_OPERATION
        ) {
            preceding_placeholder_producer = true;
        }
    }

    true
}

fn client_certify_request_type_required_with_request_value(
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
        if operation != CERTIFY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let fields = payload.children();
                let has_request_value = fields
                    .iter()
                    .any(|field| field.tag().raw() == CERTIFICATE_REQUEST_VALUE);
                !has_request_value
                    || fields.iter().any(|field| {
                        field.tag().raw() == CERTIFICATE_REQUEST_TYPE
                            && field.with_value(|value| matches!(value, ValueView::Enumeration(_)))
                    })
            })
            .unwrap_or(false)
    })
}

fn client_recertify_request_type_required_with_request_value(
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
        if operation != RECERTIFY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let fields = payload.children();
                let has_request_value = fields
                    .iter()
                    .any(|field| field.tag().raw() == CERTIFICATE_REQUEST_VALUE);
                !has_request_value
                    || fields.iter().any(|field| {
                        field.tag().raw() == CERTIFICATE_REQUEST_TYPE
                            && field.with_value(|value| matches!(value, ValueView::Enumeration(_)))
                    })
            })
            .unwrap_or(false)
    })
}

fn client_certify_omitted_request_requires_unique_identifier(
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
        if operation != CERTIFY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let fields = payload.children();
                let has_certificate_request = fields
                    .iter()
                    .any(|field| field.tag().raw() == CERTIFICATE_REQUEST_VALUE);
                let mut identifier_count = 0;
                let mut has_direct_identifier = false;
                for field in fields
                    .iter()
                    .filter(|field| field.tag().raw() == CERTIFY_UNIQUE_IDENTIFIER)
                {
                    identifier_count += 1;
                    has_direct_identifier = field.with_value(|value| {
                        matches!(value, ValueView::TextString(_))
                            || matches!(
                                value,
                                ValueView::Enumeration(value)
                                    if *value != ID_PLACEHOLDER_ENUMERATION_VALUE
                            )
                    });
                }
                has_certificate_request || (identifier_count == 1 && has_direct_identifier)
            })
            .unwrap_or(false)
    })
}

fn client_certify_omitted_request_requires_certificate_type_attribute(
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
        if operation != CERTIFY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let fields = payload.children();
                let both_request_fields_omitted = !fields.iter().any(|field| {
                    matches!(
                        field.tag().raw(),
                        CERTIFICATE_REQUEST_TYPE | CERTIFICATE_REQUEST_VALUE
                    )
                });
                if both_request_fields_omitted {
                    fields
                        .iter()
                        .filter(|field| field.tag().raw() == ATTRIBUTES)
                        .any(|field| {
                            field.with_value(|value| match value {
                                ValueView::Structure(attributes) => {
                                    attributes.children().iter().any(|attribute| {
                                        attribute.tag().raw() == CERTIFICATE_TYPE_ATTRIBUTE
                                            && attribute.with_value(|value| {
                                                matches!(value, ValueView::Enumeration(_))
                                            })
                                    })
                                }
                                _ => false,
                            })
                        })
                } else {
                    true
                }
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

fn client_gcm_cryptographic_parameters_include_tag_length(
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
                    .all(item_and_nested_cryptographic_parameters_are_valid)
            })
            .unwrap_or(false)
    })
}

fn client_variable_iv_cryptographic_parameters_include_iv_length(
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
                    .all(item_and_nested_variable_iv_parameters_are_valid)
            })
            .unwrap_or(false)
    })
}

fn item_and_nested_variable_iv_parameters_are_valid(item: &Item) -> bool {
    let item_is_valid = if item.tag().raw() == CRYPTOGRAPHIC_PARAMETERS {
        item.with_value(|value| match value {
            ValueView::Structure(parameters) => {
                let mut variable_iv_mode_is_selected = false;
                let mut has_iv_length = false;
                let fields_are_valid =
                    parameters
                        .children()
                        .iter()
                        .all(|parameter| match parameter.tag().raw() {
                            BLOCK_CIPHER_MODE => parameter.with_value(|value| {
                                if let ValueView::Enumeration(mode) = value {
                                    variable_iv_mode_is_selected |=
                                        matches!(*mode, CTR_MODE | CCM_MODE | GCM_MODE);
                                    true
                                } else {
                                    false
                                }
                            }),
                            IV_LENGTH => parameter.with_value(|value| {
                                let valid = matches!(value, ValueView::Integer(_));
                                has_iv_length |= valid;
                                valid
                            }),
                            _ => true,
                        });

                fields_are_valid && (!variable_iv_mode_is_selected || has_iv_length)
            }
            _ => false,
        })
    } else {
        true
    };

    item_is_valid
        && item.with_value(|value| match value {
            ValueView::Structure(nested) => nested
                .children()
                .iter()
                .all(item_and_nested_variable_iv_parameters_are_valid),
            _ => true,
        })
}

fn item_and_nested_cryptographic_parameters_are_valid(item: &Item) -> bool {
    let item_is_valid = if item.tag().raw() == CRYPTOGRAPHIC_PARAMETERS {
        item.with_value(|value| match value {
            ValueView::Structure(parameters) => {
                let mut block_cipher_mode_is_gcm = false;
                let mut has_tag_length = false;
                let fields_are_valid =
                    parameters
                        .children()
                        .iter()
                        .all(|parameter| match parameter.tag().raw() {
                            BLOCK_CIPHER_MODE => parameter.with_value(|value| {
                                if let ValueView::Enumeration(mode) = value {
                                    block_cipher_mode_is_gcm |= *mode == GCM_MODE;
                                    true
                                } else {
                                    false
                                }
                            }),
                            TAG_LENGTH => parameter.with_value(|value| {
                                let valid = matches!(value, ValueView::Integer(_));
                                has_tag_length |= valid;
                                valid
                            }),
                            _ => true,
                        });

                fields_are_valid && (!block_cipher_mode_is_gcm || has_tag_length)
            }
            _ => false,
        })
    } else {
        true
    };

    item_is_valid
        && item.with_value(|value| match value {
            ValueView::Structure(nested) => nested
                .children()
                .iter()
                .all(item_and_nested_cryptographic_parameters_are_valid),
            _ => true,
        })
}

fn client_get_or_export_key_compression_type_is_enumeration(
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
        if !matches!(operation, GET_OPERATION | EXPORT_OPERATION) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let mut selector_seen = false;
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == GET_KEY_COMPRESSION_TYPE)
                    .all(|field| {
                        !std::mem::replace(&mut selector_seen, true)
                            && field.with_value(|value| matches!(value, ValueView::Enumeration(_)))
                    })
            })
            .unwrap_or(false)
    })
}

// §6.1.19-002 is conditional on requesting a particular format. Request-local
// validation can preserve that selector, but cannot inspect the server's
// registered format or establish response-format support.
fn client_get_key_format_type_is_enumeration(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_get_optional_enumeration_is_well_formed(request, GET_KEY_FORMAT_TYPE)
}

fn client_get_or_export_key_format_type_is_enumeration(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_optional_enumeration_is_well_formed(
        request,
        &[GET_OPERATION, EXPORT_OPERATION],
        GET_KEY_FORMAT_TYPE,
    )
}

fn client_get_key_wrap_type_is_enumeration(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_get_optional_enumeration_is_well_formed(request, GET_KEY_WRAP_TYPE)
}

fn client_export_key_format_type_is_enumeration(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_export_optional_enumeration_is_well_formed(request, EXPORT_KEY_FORMAT_TYPE)
}

fn client_export_key_wrap_type_is_enumeration(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_export_optional_enumeration_is_well_formed(request, EXPORT_KEY_WRAP_TYPE)
}

fn client_export_controls_have_get_semantics(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_export_key_format_type_is_enumeration(request, response)
        && client_export_key_wrap_type_is_enumeration(request, response)
        && client_get_or_export_key_compression_type_is_enumeration(request, response)
        && client_get_or_export_optional_wrapping_specification_is_structure(request, response)
        && client_get_or_export_wrapping_specification_has_key_information(request, response)
}

fn client_get_optional_enumeration_is_well_formed(
    request: Option<&RequestMessage>,
    option_tag: u32,
) -> bool {
    client_optional_enumeration_is_well_formed(request, &[GET_OPERATION], option_tag)
}

fn client_export_optional_enumeration_is_well_formed(
    request: Option<&RequestMessage>,
    option_tag: u32,
) -> bool {
    client_optional_enumeration_is_well_formed(request, &[EXPORT_OPERATION], option_tag)
}

fn client_optional_enumeration_is_well_formed(
    request: Option<&RequestMessage>,
    selected_operations: &[u32],
    option_tag: u32,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if !selected_operations.contains(&operation) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let mut option_seen = false;
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == option_tag)
                    .all(|field| {
                        !std::mem::replace(&mut option_seen, true)
                            && field.with_value(|value| matches!(value, ValueView::Enumeration(_)))
                    })
            })
            .unwrap_or(false)
    })
}

fn client_check_optional_usage_limits_count_is_long_integer(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_check_optional_field_is_well_formed(
        request,
        USAGE_LIMITS_COUNT,
        CheckOptionalFieldType::LongInteger,
    )
}

fn client_pkcs12_friendly_name_uses_lowercase_text(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        batch_item
            .with_request_payload(|payload| {
                structure_uses_lowercase_pkcs12_friendly_name_text(&payload)
            })
            .unwrap_or(false)
    })
}

fn structure_uses_lowercase_pkcs12_friendly_name_text(structure: &StructureView<'_>) -> bool {
    structure.children().iter().all(|field| {
        let tag = field.tag().raw();
        if tag == PKCS12_FRIENDLY_NAME_ATTRIBUTE {
            return field.with_value(|value| match value {
                ValueView::TextString(text) => text.to_lowercase() == *text,
                _ => false,
            });
        }

        if tag == VENDOR_ATTRIBUTE || tag == VENDOR_EXTENSION || tag_is_extension(tag) {
            return true;
        }

        field.with_value(|value| match value {
            ValueView::Structure(nested) => {
                structure_uses_lowercase_pkcs12_friendly_name_text(&nested)
            }
            _ => true,
        })
    })
}

fn tag_is_extension(raw_tag: u32) -> bool {
    ((raw_tag >> 16) & 0xFF) == EXTENSION_TAG_PREFIX
}

fn client_check_additional_attributes_are_limited_to_listed_set(
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
        if operation != CHECK_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload.children().iter().all(|field| {
                    let tag = field.tag();
                    CHECK_STANDARD_REQUEST_MEMBER_TAGS.contains(&tag.raw())
                })
            })
            .unwrap_or(false)
    })
}

/// Returns `None` when any Batch Item operation is unavailable.
fn client_request_contains_check(request: &RequestMessage) -> Option<bool> {
    let mut check_present = false;
    for batch_item in request.batch_items() {
        let operation = batch_item.operation()?;
        check_present |= operation == CHECK_OPERATION;
    }
    Some(check_present)
}

fn client_check_requires_multi_item_batch_when_check_is_present(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };
    let Some(check_present) = client_request_contains_check(request) else {
        return false;
    };
    !check_present || request.batch_items().len() >= 2
}

fn client_check_batch_order_option_is_true_when_check_is_present(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };
    let Some(check_present) = client_request_contains_check(request) else {
        return false;
    };
    !check_present || request.header().effective_batch_order_option()
}

fn client_check_batch_error_continuation_option_is_stop_or_undo_when_check_is_present(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };
    let Some(check_present) = client_request_contains_check(request) else {
        return false;
    };
    !check_present
        || matches!(
            request.header().effective_batch_error_continuation_option(),
            BATCH_ERROR_CONTINUATION_OPTION_STOP | BATCH_ERROR_CONTINUATION_OPTION_UNDO
        )
}

fn client_get_usage_allocation_requires_usage_limits_count(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.23 Table 232 marks this field required; §7.40 Table 392 defines
    // its Long Integer representation. This checks request shape only.
    client_operation_field_is_well_formed(
        request,
        GET_USAGE_ALLOCATION_OPERATION,
        USAGE_LIMITS_COUNT,
        OperationFieldType::LongInteger,
        true,
    )
}

fn client_check_optional_lease_time_is_interval(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_check_optional_field_is_well_formed(
        request,
        LEASE_TIME,
        CheckOptionalFieldType::Interval,
    )
}

fn client_check_optional_cryptographic_usage_mask_is_integer(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_check_optional_field_is_well_formed(
        request,
        CRYPTOGRAPHIC_USAGE_MASK,
        CheckOptionalFieldType::Integer,
    )
}

fn client_delegated_login_rights_is_required_structure(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_operation_field_is_well_formed(
        request,
        DELEGATED_LOGIN_OPERATION,
        DELEGATED_LOGIN_RIGHTS,
        OperationFieldType::RightsStructure,
        true,
    )
}

fn client_login_lease_time_is_optional_interval_or_date_time(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_operation_field_is_well_formed(
        request,
        LOGIN_OPERATION,
        LEASE_TIME,
        OperationFieldType::LeaseTime,
        false,
    )
}

fn client_login_request_count_is_optional_integer(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_operation_field_is_well_formed(
        request,
        LOGIN_OPERATION,
        LOGIN_REQUEST_COUNT,
        OperationFieldType::Integer,
        false,
    )
}

fn client_login_usage_limits_is_optional_structure(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_operation_field_is_well_formed(
        request,
        LOGIN_OPERATION,
        LOGIN_USAGE_LIMITS,
        OperationFieldType::UsageLimitsStructure,
        false,
    )
}

fn client_logout_ticket_is_required_structure(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_operation_field_is_well_formed(
        request,
        LOGOUT_OPERATION,
        TICKET,
        OperationFieldType::Structure,
        true,
    )
}

fn client_recertify_offset_is_optional_interval(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_operation_field_is_well_formed(
        request,
        RECERTIFY_OPERATION,
        OFFSET,
        OperationFieldType::Interval,
        false,
    )
}

fn client_rekey_offset_is_optional_interval(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_operation_field_is_well_formed(
        request,
        REKEY_OPERATION,
        OFFSET,
        OperationFieldType::Interval,
        false,
    )
}

fn client_rekey_key_pair_offset_is_optional_interval(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.47 assigns date calculation to the server; this client rule checks
    // only that the optional Table 310 request field is representable.
    client_operation_field_is_well_formed(
        request,
        REKEY_KEY_PAIR_OPERATION,
        OFFSET,
        OperationFieldType::Interval,
        false,
    )
}

fn client_encrypt_cryptographic_parameters_are_optional_structure(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.17 permits omission when managed-object attributes supply the
    // parameters. Client preflight checks representation only; it cannot
    // inspect remote attributes or predict server acceptance.
    client_operation_field_is_well_formed(
        request,
        ENCRYPT_OPERATION,
        CRYPTOGRAPHIC_PARAMETERS,
        OperationFieldType::Structure,
        false,
    )
}

fn client_encrypt_iv_counter_nonce_is_optional_byte_string(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.17 permits omission under the caller-supplied parameters or when
    // the selected algorithm does not need an IV. Preflight checks only the
    // optional outbound field shape; it neither creates an IV nor predicts
    // remote-object attributes or server behavior.
    client_operation_field_is_well_formed(
        request,
        ENCRYPT_OPERATION,
        IV_COUNTER_NONCE,
        OperationFieldType::ByteString,
        false,
    )
}

fn client_decrypt_cryptographic_parameters_are_optional_structure(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.11 permits omission when the managed object's attributes supply
    // parameters. This request-local check cannot inspect those attributes.
    client_operation_field_is_well_formed(
        request,
        DECRYPT_OPERATION,
        CRYPTOGRAPHIC_PARAMETERS,
        OperationFieldType::Structure,
        false,
    )
}

fn client_decrypt_iv_counter_nonce_is_optional_byte_string(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.11 permits omission when the algorithm does not use an IV. The
    // caller supplies that context; this check validates only the wire shape.
    client_operation_field_is_well_formed(
        request,
        DECRYPT_OPERATION,
        IV_COUNTER_NONCE,
        OperationFieldType::ByteString,
        false,
    )
}

fn client_sign_cryptographic_parameters_are_optional_structure(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.55 permits omission when managed-object attributes supply the
    // parameters. This client-side rule validates representation only; it
    // cannot inspect those remote attributes or predict server acceptance.
    client_operation_field_is_well_formed(
        request,
        SIGN_OPERATION,
        SIGN_CRYPTOGRAPHIC_PARAMETERS,
        OperationFieldType::Structure,
        false,
    )
}

fn client_signature_verify_cryptographic_parameters_are_optional_structure(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.56 permits omission when managed-object attributes supply the
    // parameters. This client-side rule checks representation only.
    client_operation_field_is_well_formed(
        request,
        SIGNATURE_VERIFY_OPERATION,
        CRYPTOGRAPHIC_PARAMETERS,
        OperationFieldType::Structure,
        false,
    )
}

fn client_signature_verify_original_data_is_optional_byte_string(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.56 permits original Data when the selected algorithm requires it.
    // The client validates representation and does not infer algorithm needs.
    client_operation_field_is_well_formed(
        request,
        SIGNATURE_VERIFY_OPERATION,
        SIGNATURE_VERIFY_DATA,
        OperationFieldType::ByteString,
        false,
    )
}

fn client_mac_cryptographic_parameters_are_optional_structure(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.32 permits omission when managed-object attributes supply the
    // parameters. This request-local rule validates shape only.
    client_operation_field_is_well_formed(
        request,
        MAC_OPERATION,
        CRYPTOGRAPHIC_PARAMETERS,
        OperationFieldType::Structure,
        false,
    )
}

fn client_mac_verify_cryptographic_parameters_are_optional_structure(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.33 permits omission when managed-object attributes supply the
    // parameters. This request-local rule validates shape only.
    client_operation_field_is_well_formed(
        request,
        MAC_VERIFY_OPERATION,
        CRYPTOGRAPHIC_PARAMETERS,
        OperationFieldType::Structure,
        false,
    )
}

fn client_mac_verify_original_data_is_optional_byte_string(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.33 permits original Data for algorithms that need it. The client
    // validates its representation and does not infer algorithm requirements.
    client_operation_field_is_well_formed(
        request,
        MAC_VERIFY_OPERATION,
        MAC_VERIFY_ORIGINAL_DATA,
        OperationFieldType::ByteString,
        false,
    )
}

// Table 313 marks both request objects optional; §6.1.48 does not say they
// are mutually exclusive. Each present object still has the required §2 shape.
fn client_reprovision_request_fields_are_well_formed(
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
        if operation != REPROVISION_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                optional_reprovision_object_is_well_formed(
                    &payload,
                    REPROVISION_CERTIFICATE_REQUEST,
                    REPROVISION_CERTIFICATE_REQUEST_TYPE,
                    REPROVISION_CERTIFICATE_REQUEST_VALUE,
                ) && optional_reprovision_object_is_well_formed(
                    &payload,
                    REPROVISION_CERTIFICATE,
                    REPROVISION_CERTIFICATE_TYPE,
                    REPROVISION_CERTIFICATE_VALUE,
                )
            })
            .unwrap_or(false)
    })
}

fn optional_reprovision_object_is_well_formed(
    payload: &StructureView<'_>,
    object_tag: u32,
    type_tag: u32,
    value_tag: u32,
) -> bool {
    match field_presence(payload, object_tag) {
        FieldPresence::Missing => true,
        FieldPresence::Unique(field) => field.with_value(|value| match value {
            ValueView::Structure(object) => {
                reprovision_object_fields_are_well_formed(&object, type_tag, value_tag)
            }
            _ => false,
        }),
        FieldPresence::Duplicate => false,
    }
}

fn reprovision_object_fields_are_well_formed(
    object: &StructureView<'_>,
    type_tag: u32,
    value_tag: u32,
) -> bool {
    let FieldPresence::Unique(object_type) = field_presence(object, type_tag) else {
        return false;
    };
    let FieldPresence::Unique(object_value) = field_presence(object, value_tag) else {
        return false;
    };

    object_type.with_value(|value| matches!(value, ValueView::Enumeration(_)))
        && object_value.with_value(|value| matches!(value, ValueView::ByteString(_)))
}

fn client_delegated_login_lease_time_is_optional_interval_or_date_time(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_operation_field_is_well_formed(
        request,
        DELEGATED_LOGIN_OPERATION,
        LEASE_TIME,
        OperationFieldType::LeaseTime,
        false,
    )
}

fn client_delegated_login_request_count_is_optional_integer(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_operation_field_is_well_formed(
        request,
        DELEGATED_LOGIN_OPERATION,
        DELEGATED_LOGIN_REQUEST_COUNT,
        OperationFieldType::Integer,
        false,
    )
}

fn client_delegated_login_usage_limits_is_optional_structure(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_operation_field_is_well_formed(
        request,
        DELEGATED_LOGIN_OPERATION,
        DELEGATED_LOGIN_USAGE_LIMITS,
        OperationFieldType::UsageLimitsStructure,
        false,
    )
}

#[derive(Clone, Copy)]
enum OperationFieldType {
    ByteString,
    Integer,
    LongInteger,
    Interval,
    LeaseTime,
    Structure,
    RightsStructure,
    UsageLimitsStructure,
}

impl OperationFieldType {
    fn accepts(self, value: ValueView<'_>) -> bool {
        match self {
            Self::ByteString => matches!(value, ValueView::ByteString(_)),
            Self::Integer => matches!(value, ValueView::Integer(_)),
            Self::LongInteger => matches!(value, ValueView::LongInteger(_)),
            Self::Interval => matches!(value, ValueView::Interval(_)),
            Self::LeaseTime => matches!(value, ValueView::Interval(_) | ValueView::DateTime(_)),
            Self::Structure => matches!(value, ValueView::Structure(_)),
            Self::RightsStructure => match value {
                ValueView::Structure(value) => delegated_login_rights_are_well_formed(&value),
                _ => false,
            },
            Self::UsageLimitsStructure => match value {
                ValueView::Structure(value) => usage_limits_structure_is_well_formed(&value),
                _ => false,
            },
        }
    }
}

fn client_operation_field_is_well_formed(
    request: Option<&RequestMessage>,
    selected_operation: u32,
    field_tag: u32,
    expected_type: OperationFieldType,
    required: bool,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if operation != selected_operation {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let mut field_seen = false;
                let well_formed = payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == field_tag)
                    .all(|field| {
                        !std::mem::replace(&mut field_seen, true)
                            && field.with_value(|value| expected_type.accepts(value))
                    });
                well_formed && (!required || field_seen)
            })
            .unwrap_or(false)
    })
}

fn client_join_split_key_has_validated_identifier_entries(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // Thresholds are caller-supplied metadata absent from TTLV. The typed client
    // serializer checks their count before this message exists; this rule confirms
    // the encoded identifier entries remain present and well-formed.
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if operation != JOIN_SPLIT_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                join_split_key_identifier_entries_are_well_formed(&payload)
            })
            .unwrap_or(false)
    })
}

fn join_split_key_identifier_entries_are_well_formed(payload: &StructureView<'_>) -> bool {
    let mut identifier_count = 0_usize;
    let valid_entries = payload.children().iter().all(|field| {
        if field.tag().raw() != UNIQUE_IDENTIFIER {
            return true;
        }

        let valid_identifier = field.with_value(|value| {
            matches!(
                value,
                ValueView::TextString(_) | ValueView::Integer(_) | ValueView::Enumeration(_)
            )
        });
        if valid_identifier {
            identifier_count += 1;
        }
        valid_identifier
    });

    valid_entries && identifier_count > 0
}

fn client_join_split_key_secret_data_type_is_valid_when_applicable(
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
        if operation != JOIN_SPLIT_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                join_split_key_secret_data_type_is_well_formed(&payload)
            })
            .unwrap_or(false)
    })
}

fn join_split_key_secret_data_type_is_well_formed(payload: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(object_type) =
        field_presence(payload, JOIN_SPLIT_KEY_OBJECT_TYPE_TAG)
    else {
        // Object Type is a separate operation requirement. This optional rule only
        // checks Secret Data Type when its condition is observable.
        return true;
    };
    let Some(object_type) = object_type.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    }) else {
        return true;
    };
    if object_type != SECRET_DATA_OBJECT_TYPE {
        return true;
    }

    match field_presence(payload, SECRET_DATA_TYPE) {
        FieldPresence::Missing => true,
        FieldPresence::Unique(secret_data_type) => {
            secret_data_type.with_value(|value| matches!(value, ValueView::Enumeration(_)))
        }
        FieldPresence::Duplicate => false,
    }
}

fn delegated_login_rights_are_well_formed(structure: &kmipkit_ttlv::StructureView<'_>) -> bool {
    structure.children().iter().all(|field| {
        field.tag().raw() != DELEGATED_LOGIN_RIGHT
            || field.with_value(|value| matches!(value, ValueView::Structure(_)))
    })
}

fn usage_limits_structure_is_well_formed(structure: &kmipkit_ttlv::StructureView<'_>) -> bool {
    // Table 392 requires one of each standard member; unknown members remain uninterpreted.
    let mut total_seen = false;
    let mut count_seen = false;
    let mut unit_seen = false;

    for field in structure.children() {
        match field.tag().raw() {
            USAGE_LIMITS_TOTAL => {
                if total_seen
                    || !field.with_value(|value| matches!(value, ValueView::LongInteger(_)))
                {
                    return false;
                }
                total_seen = true;
            }
            USAGE_LIMITS_COUNT => {
                if count_seen
                    || !field.with_value(|value| matches!(value, ValueView::LongInteger(_)))
                {
                    return false;
                }
                count_seen = true;
            }
            USAGE_LIMITS_UNIT => {
                if unit_seen
                    || !field.with_value(|value| matches!(value, ValueView::Enumeration(_)))
                {
                    return false;
                }
                unit_seen = true;
            }
            _ => {}
        }
    }

    total_seen && count_seen && unit_seen
}

#[derive(Clone, Copy)]
enum CheckOptionalFieldType {
    Integer,
    LongInteger,
    Interval,
}

fn client_check_optional_field_is_well_formed(
    request: Option<&RequestMessage>,
    field_tag: u32,
    expected_type: CheckOptionalFieldType,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if operation != CHECK_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let mut field_seen = false;
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == field_tag)
                    .all(|field| {
                        !std::mem::replace(&mut field_seen, true)
                            && field.with_value(|value| match expected_type {
                                CheckOptionalFieldType::Integer => {
                                    matches!(value, ValueView::Integer(_))
                                }
                                CheckOptionalFieldType::LongInteger => {
                                    matches!(value, ValueView::LongInteger(_))
                                }
                                CheckOptionalFieldType::Interval => {
                                    matches!(value, ValueView::Interval(_))
                                }
                            })
                    })
            })
            .unwrap_or(false)
    })
}

fn client_get_optional_wrapping_specification_is_structure(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_optional_wrapping_specification_is_structure(request, &[GET_OPERATION], response)
}

fn client_get_or_export_optional_wrapping_specification_is_structure(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_optional_wrapping_specification_is_structure(
        request,
        &[GET_OPERATION, EXPORT_OPERATION],
        response,
    )
}

fn client_optional_wrapping_specification_is_structure(
    request: Option<&RequestMessage>,
    selected_operations: &[u32],
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if !selected_operations.contains(&operation) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let mut specification_seen = false;
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == KEY_WRAPPING_SPECIFICATION)
                    .all(|field| {
                        !std::mem::replace(&mut specification_seen, true)
                            && field.with_value(|value| matches!(value, ValueView::Structure(_)))
                    })
            })
            .unwrap_or(false)
    })
}

fn client_can_include_optional_unique_batch_item_id(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §9.21 makes the ID optional in requests; §8.3 requires it for multi-item
    // requests. Positive evidence proves both single-item caller choices work.
    request.is_some()
}

fn client_can_repeat_request_batch_items(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // Repetition is optional per request; executable evidence proves that a
    // valid client request can contain multiple Batch Items.
    request.is_some()
}

fn client_can_repeat_request_attestation_types(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // Repetition is optional per request; positive evidence proves that a
    // valid Request Header can preserve multiple Attestation Type values.
    request.is_some()
}

fn client_sets_attestation_indicator_true(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §9.3 treats absence as False, so supported clients must send True explicitly.
    request.is_some_and(|request| request.header().attestation_capable_indicator() == Some(true))
}

fn client_can_repeat_request_message_extensions(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §8.3 Table 396 permits repetition; the request model validates field shape.
    request.is_some()
}

fn client_rejects_unknown_critical_response_extensions(
    _request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some((_, response)) = response else {
        return true;
    };

    response.batch_items().all(|batch_item| {
        (0..batch_item.message_extension_count()).all(|extension_index| {
            batch_item
                .message_extension(extension_index)
                .is_some_and(|extension| extension.criticality_indicator() == Some(false))
        })
    })
}

fn client_can_process_unknown_noncritical_response_extensions(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some((_, response)) = response else {
        return request.is_some();
    };

    response.batch_items().all(|batch_item| {
        (0..batch_item.message_extension_count())
            .all(|extension_index| batch_item.message_extension(extension_index).is_some())
    })
}

fn client_can_include_optional_create_split_key_source_identifier(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.10 permits a caller to identify the existing split source.
    request.is_some()
}

fn client_split_key_part_identifier_is_within_declared_parts(
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
        if !matches!(operation, REGISTER_OPERATION | IMPORT_OPERATION) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let FieldPresence::Unique(object_type) =
                    field_presence(&payload, REGISTER_OBJECT_TYPE)
                else {
                    return false;
                };
                let Some(object_type) = object_type.with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                }) else {
                    return false;
                };
                if object_type != SPLIT_KEY_OBJECT_TYPE {
                    return true;
                }

                let FieldPresence::Unique(split_key) = field_presence(&payload, SPLIT_KEY) else {
                    return false;
                };
                split_key.with_value(|value| match value {
                    ValueView::Structure(split_key) => {
                        split_key_part_identifier_is_within_parts(&split_key)
                    }
                    _ => false,
                })
            })
            .unwrap_or(false)
    })
}

fn split_key_part_identifier_is_within_parts(split_key: &StructureView<'_>) -> bool {
    let Some(split_key_parts) = unique_integer_field_value(split_key, SPLIT_KEY_PARTS) else {
        return false;
    };
    let Some(key_part_identifier) = unique_integer_field_value(split_key, KEY_PART_IDENTIFIER)
    else {
        return false;
    };

    (1..=split_key_parts).contains(&key_part_identifier)
}

fn unique_integer_field_value(structure: &StructureView<'_>, tag: u32) -> Option<i32> {
    let FieldPresence::Unique(item) = field_presence(structure, tag) else {
        return None;
    };
    item.with_value(|value| match value {
        ValueView::Integer(value) => Some(*value),
        _ => None,
    })
}

fn client_create_split_key_requires_prime_field_size_for_polynomial_method(
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
        if operation != CREATE_SPLIT_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let FieldPresence::Unique(method) = field_presence(&payload, SPLIT_KEY_METHOD)
                else {
                    return false;
                };
                let Some(method) = method.with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                }) else {
                    return false;
                };

                match field_presence(&payload, PRIME_FIELD_SIZE) {
                    FieldPresence::Missing => method != POLYNOMIAL_SHARING_PRIME_FIELD,
                    FieldPresence::Duplicate => false,
                    FieldPresence::Unique(size) => {
                        size.with_value(|value| matches!(value, ValueView::BigInteger(_)))
                    }
                }
            })
            .unwrap_or(false)
    })
}

fn client_can_represent_optional_delete_attribute_selector(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.13 Table 202 permits the typed request model's optional selector.
    request.is_some()
}

fn client_can_represent_optional_modify_attribute_selector(
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
        if operation != MODIFY_ATTRIBUTE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(
                |payload| match field_presence(&payload, CURRENT_ATTRIBUTE) {
                    FieldPresence::Missing => true,
                    FieldPresence::Duplicate => false,
                    FieldPresence::Unique(current_attribute) => current_attribute
                        .with_value(|value| matches!(value, ValueView::Structure(_))),
                },
            )
            .unwrap_or(false)
    })
}

fn client_can_include_optional_get_attributes_attribute_references(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.20 Table 223 permits zero or more distinct Attribute Reference fields.
    request.is_some()
}

fn client_can_include_optional_locate_maximum_items(
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
        if operation != LOCATE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == MAXIMUM_ITEMS)
                    .all(|field| field.with_value(|value| matches!(value, ValueView::Integer(_))))
            })
            .unwrap_or(false)
    })
}

fn client_can_include_optional_locate_offset_items(
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
        if operation != LOCATE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == OFFSET_ITEMS)
                    .all(|field| field.with_value(|value| matches!(value, ValueView::Integer(_))))
            })
            .unwrap_or(false)
    })
}

fn client_supplies_locate_attributes_structure(
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
        if operation != LOCATE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let mut found_attributes = false;
                let attributes_are_structures = payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == ATTRIBUTES)
                    .all(|field| {
                        found_attributes = true;
                        field.with_value(|value| matches!(value, ValueView::Structure(_)))
                    });
                found_attributes && attributes_are_structures
            })
            .unwrap_or(false)
    })
}

fn client_can_send_partial_structured_locate_attributes(
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
        if operation != LOCATE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let mut found_attributes = false;
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == ATTRIBUTES)
                    .all(|field| {
                        found_attributes = true;
                        field.with_value(|value| {
                            let ValueView::Structure(attributes) = value else {
                                return false;
                            };
                            attributes
                                .children()
                                .iter()
                                .filter(|attribute| attribute.tag().raw() == LINK_ATTRIBUTE)
                                .all(|attribute| {
                                    attribute.with_value(|value| {
                                        matches!(value, ValueView::Structure(_))
                                    })
                                })
                        })
                    })
                    && found_attributes
            })
            .unwrap_or(false)
    })
}

fn client_can_omit_link_when_associated_key_material_is_unavailable(
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
        if operation != REGISTER_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let mut found_attributes = false;
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == ATTRIBUTES)
                    .all(|field| {
                        found_attributes = true;
                        field.with_value(|value| matches!(value, ValueView::Structure(_)))
                    })
                    && found_attributes
            })
            .unwrap_or(false)
    })
}

fn client_link_type_is_required_in_object_state(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_link_members_are_required_in_object_state(
        request,
        response,
        RequiredLinkMember::LinkType,
    )
}

fn client_linked_object_identifier_is_required_in_object_state(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_link_members_are_required_in_object_state(
        request,
        response,
        RequiredLinkMember::LinkedObjectIdentifier,
    )
}

#[derive(Clone, Copy)]
enum RequiredLinkMember {
    LinkType,
    LinkedObjectIdentifier,
}

fn client_link_members_are_required_in_object_state(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
    required_member: RequiredLinkMember,
) -> bool {
    if let Some((request, response)) = response {
        let request_items = request.batch_items().collect::<Vec<_>>();
        let response_items = response.batch_items().collect::<Vec<_>>();
        if request_items.len() != response_items.len() {
            return false;
        }

        let mut consumed_responses = vec![false; response_items.len()];
        for request_item in request_items.iter().copied() {
            if request_item.operation() != Some(GET_ATTRIBUTES_OPERATION) {
                continue;
            }

            let request_id = request_item.with_unique_batch_item_id(<[u8]>::to_vec);
            let response_index = if let Some(request_id) = request_id {
                let matching_indices = response_items
                    .iter()
                    .enumerate()
                    .filter_map(|(index, item)| {
                        item.with_unique_batch_item_id(|response_id| {
                            response_id == request_id.as_slice()
                        })
                        .filter(|matches| *matches)
                        .map(|_| index)
                    })
                    .collect::<Vec<_>>();
                if matching_indices.len() != 1 {
                    return false;
                }
                matching_indices[0]
            } else {
                if request_items.len() != 1
                    || response_items.len() != 1
                    || response_items[0]
                        .with_unique_batch_item_id(|_| ())
                        .is_some()
                {
                    return false;
                }
                0
            };

            if consumed_responses[response_index] {
                return false;
            }
            consumed_responses[response_index] = true;

            let response_item = response_items[response_index];
            if response_item.operation() != Some(GET_ATTRIBUTES_OPERATION) {
                return false;
            }

            let Some(status) = response_item.result_status() else {
                return false;
            };
            if status.raw() != RESULT_STATUS_SUCCESS {
                continue;
            }

            let links_are_valid = response_item
                .with_response_payload(|payload| {
                    payload
                        .children()
                        .iter()
                        .filter(|field| field.tag().raw() == ATTRIBUTES)
                        .all(|field| {
                            field.with_value(|value| match value {
                                ValueView::Structure(attributes) => {
                                    object_state_links_have_required_members(
                                        &attributes,
                                        required_member,
                                    )
                                }
                                _ => false,
                            })
                        })
                })
                .unwrap_or(true);
            if !links_are_valid {
                return false;
            }
        }
        return true;
    }

    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if operation == LOCATE_OPERATION {
            // §6.1.28 permits partial structured Link search criteria.
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                object_state_links_have_required_members(&payload, required_member)
            })
            .unwrap_or(false)
    })
}

fn object_state_links_have_required_members(
    structure: &StructureView<'_>,
    required_member: RequiredLinkMember,
) -> bool {
    structure.children().iter().all(|field| {
        if field.tag().raw() == LINK_ATTRIBUTE {
            field.with_value(|value| match value {
                ValueView::Structure(link) => match required_member {
                    RequiredLinkMember::LinkType => link_has_required_type(&link),
                    RequiredLinkMember::LinkedObjectIdentifier => {
                        link_has_required_object_identifier(&link)
                    }
                },
                _ => false,
            })
        } else {
            field.with_value(|value| match value {
                ValueView::Structure(nested) => {
                    object_state_links_have_required_members(&nested, required_member)
                }
                _ => true,
            })
        }
    })
}

fn link_has_required_type(link: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(link_type) = field_presence(link, LINK_TYPE) else {
        return false;
    };
    link_type.with_value(|value| matches!(value, ValueView::Enumeration(_)))
}

fn link_has_required_object_identifier(link: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(identifier) = field_presence(link, LINKED_OBJECT_IDENTIFIER) else {
        return false;
    };
    identifier.with_value(|value| {
        matches!(
            value,
            ValueView::TextString(_) | ValueView::Enumeration(_) | ValueView::Integer(_)
        )
    })
}

fn get_attributes_references_are_not_repeated(
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
        if operation != GET_ATTRIBUTES_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let mut seen = HashSet::new();
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == ATTRIBUTE_REFERENCE)
                    .all(|field| {
                        AttributeReference::try_from_ttlv_item(field)
                            .is_ok_and(|reference| seen.insert(reference))
                    })
            })
            .unwrap_or(false)
    })
}

fn client_query_contains_one_or_more_functions(
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
        if operation != QUERY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let mut query_function_count = 0;
                let all_query_functions_are_enumerations = payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == QUERY_FUNCTION)
                    .all(|field| {
                        query_function_count += 1;
                        field.with_value(|value| matches!(value, ValueView::Enumeration(_)))
                    });
                query_function_count > 0 && all_query_functions_are_enumerations
            })
            .unwrap_or(false)
    })
}

fn client_query_profile_information_is_well_formed(
    _request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some((request, response)) = response else {
        return true;
    };
    let query_profiles_requested = request.batch_items().any(|batch_item| {
        if batch_item.operation() != Some(QUERY_OPERATION) {
            return false;
        }

        batch_item
            .with_request_payload(|payload| {
                payload.children().iter().any(|field| {
                    field.tag().raw() == QUERY_FUNCTION
                        && field.with_value(
                            |value| matches!(value, ValueView::Enumeration(raw) if *raw == 10),
                        )
                })
            })
            .unwrap_or(false)
    });
    if !query_profiles_requested {
        return true;
    }

    response
        .batch_items()
        .filter(|batch_item| batch_item.operation() == Some(QUERY_OPERATION))
        .all(|batch_item| {
            batch_item
                .with_response_payload(|payload| {
                    payload
                        .children()
                        .iter()
                        .filter(|field| field.tag().raw() == PROFILE_INFORMATION)
                        .all(|field| {
                            field.with_value(|value| {
                                crate::query::profile_information_is_well_formed(&value)
                            })
                        })
                })
                .unwrap_or(true)
        })
}

fn client_can_repeat_query_functions(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.40 Table 282 permits repetition; positive evidence verifies the
    // typed Query request preserves repeated functions in caller order.
    request.is_some()
}

fn client_can_repeat_get_or_export_wrapping_attribute_names(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §7.18 Table 370 permits repeated Attribute Name Text Strings in a Key
    // Wrapping Specification; §6.1.19 Table 220 carries that structure in Get.
    request.is_some()
}

fn client_get_or_export_wrapping_specification_has_key_information(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        if !matches!(
            batch_item.operation(),
            Some(GET_OPERATION | EXPORT_OPERATION)
        ) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload.children().iter().all(|field| {
                    if field.tag().raw() != KEY_WRAPPING_SPECIFICATION {
                        return true;
                    }

                    field.with_value(|value| match value {
                        ValueView::Structure(specification) => {
                            wrapping_structure_has_key_information(&specification)
                        }
                        _ => false,
                    })
                })
            })
            .unwrap_or(false)
    })
}

fn client_supports_get_or_export_wrapped_key_request(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §6.1.18 gives Export's wrapping controls Get semantics. A request without
    // the optional structure does not assert that the client requests wrapping.
    client_get_or_export_optional_wrapping_specification_is_structure(request, response)
}

fn client_can_include_request_time_stamp(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §9.20 makes a client request Time Stamp optional; positive evidence
    // proves that the request model accepts and preserves the supplied value.
    request.is_some()
}

fn client_authentication_contains_a_credential_if_present(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request
        .header()
        .with_authentication(|authentication| {
            authentication.children().iter().any(|field| {
                field.tag().raw() == CREDENTIAL
                    && field.with_value(|value| matches!(value, ValueView::Structure(_)))
            })
        })
        .unwrap_or(true)
}

fn client_can_omit_authentication_when_not_required(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §9.4 leaves whether Authentication is needed to the caller and server policy.
    request.is_some()
}

fn client_can_represent_multi_instance_attributes(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §4 defines a client-side representation capability; object storage remains server-owned.
    request.is_some() || response.is_some()
}

fn client_preserves_single_instance_attribute_cardinality(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };
    if !request_attribute_sets_preserve_single_instance_cardinality(request) {
        return false;
    }
    let Some((_, response)) = response else {
        return true;
    };

    response.batch_items().all(|item| {
        if item.operation() == Some(GET_OPERATION) {
            let Ok(typed) = crate::GetResponse::try_from_response_item(item) else {
                return false;
            };
            let object_type = typed.object_type().map(crate::ObjectType::raw);
            return typed.object().is_none_or(|object| {
                crate::attribute::validate_key_value_attribute_cardinality(
                    object,
                    object_type,
                    true,
                )
                .is_ok()
            });
        }
        if item.operation() == Some(EXPORT_OPERATION) {
            let Ok(typed) = crate::ExportResponse::try_from_response_item(item) else {
                return false;
            };
            let object_type = typed.object_type().map(crate::ObjectType::raw);
            let attributes_valid = typed.attributes().is_none_or(|attributes| {
                attributes.with_value(|value| match value {
                    ValueView::Structure(attributes) => {
                        crate::attribute::validate_single_instance_attribute_items(
                            attributes.children(),
                            object_type,
                            true,
                        )
                        .is_ok()
                    }
                    _ => false,
                })
            });
            let object_valid = typed.object().is_none_or(|object| {
                crate::attribute::validate_key_value_attribute_cardinality(
                    object,
                    object_type,
                    true,
                )
                .is_ok()
            });
            return attributes_valid && object_valid;
        }
        if item.operation() != Some(GET_ATTRIBUTES_OPERATION) {
            return true;
        }
        let Ok(typed) = crate::GetAttributesResponse::try_from_response_item(item) else {
            return false;
        };
        typed.attributes().is_none_or(|attributes| {
            crate::attribute::validate_single_instance_attribute_items(
                attributes.as_items(),
                None,
                true,
            )
            .is_ok()
        })
    })
}

fn request_attribute_sets_preserve_single_instance_cardinality(request: &RequestMessage) -> bool {
    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if !operation_contains_attribute_sets(operation) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let object_type = request_payload_object_type(&payload, operation);
                request_direct_attribute_groups_preserve_cardinality(
                    &payload,
                    operation,
                    object_type,
                ) && request_key_value_attributes_preserve_cardinality(
                    &payload,
                    operation,
                    object_type,
                )
            })
            .unwrap_or(false)
    })
}

fn operation_contains_attribute_sets(operation: u32) -> bool {
    matches!(
        operation,
        CREATE_OPERATION
            | REGISTER_OPERATION
            | IMPORT_OPERATION
            | CREATE_KEY_PAIR_OPERATION
            | CERTIFY_OPERATION
            | REKEY_OPERATION
            | RECERTIFY_OPERATION
            | REKEY_KEY_PAIR_OPERATION
            | CREATE_SPLIT_KEY_OPERATION
            | JOIN_SPLIT_KEY_OPERATION
            | DERIVE_KEY_OPERATION
            | ADD_ATTRIBUTE_OPERATION
            | MODIFY_ATTRIBUTE_OPERATION
            | SET_ATTRIBUTE_OPERATION
    )
}

fn request_payload_object_type(payload: &StructureView<'_>, operation: u32) -> Option<u32> {
    if matches!(
        operation,
        CREATE_OPERATION
            | REGISTER_OPERATION
            | IMPORT_OPERATION
            | CREATE_SPLIT_KEY_OPERATION
            | JOIN_SPLIT_KEY_OPERATION
            | DERIVE_KEY_OPERATION
    ) {
        let mut fields = payload
            .children()
            .iter()
            .filter(|field| field.tag().raw() == REGISTER_OBJECT_TYPE);
        return match (fields.next(), fields.next()) {
            (Some(field), None) => field.with_value(|value| match value {
                ValueView::Enumeration(value) => Some(*value),
                _ => None,
            }),
            _ => None,
        };
    }
    matches!(operation, CERTIFY_OPERATION | RECERTIFY_OPERATION).then_some(CERTIFICATE_OBJECT_TYPE)
}

fn request_direct_attribute_groups_preserve_cardinality(
    payload: &StructureView<'_>,
    operation: u32,
    object_type: Option<u32>,
) -> bool {
    let mut seen_groups = HashSet::new();
    payload.children().iter().all(|field| {
        let is_attribute_group = match operation {
            CREATE_OPERATION
            | REGISTER_OPERATION
            | IMPORT_OPERATION
            | CERTIFY_OPERATION
            | REKEY_OPERATION
            | RECERTIFY_OPERATION
            | CREATE_SPLIT_KEY_OPERATION
            | JOIN_SPLIT_KEY_OPERATION
            | DERIVE_KEY_OPERATION => field.tag().raw() == ATTRIBUTES,
            CREATE_KEY_PAIR_OPERATION | REKEY_KEY_PAIR_OPERATION => matches!(
                field.tag().raw(),
                CREATE_KEY_PAIR_COMMON_ATTRIBUTES
                    | CREATE_KEY_PAIR_PRIVATE_KEY_ATTRIBUTES
                    | CREATE_KEY_PAIR_PUBLIC_KEY_ATTRIBUTES
            ),
            ADD_ATTRIBUTE_OPERATION | MODIFY_ATTRIBUTE_OPERATION | SET_ATTRIBUTE_OPERATION => {
                field.tag().raw() == NEW_ATTRIBUTE
            }
            _ => false,
        };
        if !is_attribute_group {
            return true;
        }
        if !seen_groups.insert(field.tag().raw()) {
            return false;
        }
        field.with_value(|value| match value {
            ValueView::Structure(attributes) => {
                crate::attribute::validate_single_instance_attribute_items(
                    attributes.children(),
                    object_type,
                    true,
                )
                .is_ok()
            }
            _ => false,
        })
    })
}

fn request_key_value_attributes_preserve_cardinality(
    payload: &StructureView<'_>,
    operation: u32,
    object_type: Option<u32>,
) -> bool {
    if !matches!(operation, REGISTER_OPERATION | IMPORT_OPERATION) {
        return true;
    }
    payload.children().iter().all(|field| {
        crate::attribute::validate_key_value_attribute_cardinality(field, object_type, true).is_ok()
    })
}

fn client_can_repeat_credentials_within_authentication(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §9.4 Table 403 permits repeating the Credential structure.
    request.is_some()
}

fn client_device_credential_has_identifier_if_present(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request
        .header()
        .with_authentication(|authentication| {
            authentication.children().iter().all(|field| {
                if field.tag().raw() != CREDENTIAL {
                    return true;
                }

                field.with_value(|value| match value {
                    ValueView::Structure(credential) => {
                        let Ok(credential_type) =
                            crate::credential::credential_type_from_view(&credential)
                        else {
                            return false;
                        };
                        if credential_type != CredentialType::Device {
                            return true;
                        }

                        credential
                            .children()
                            .iter()
                            .find(|child| child.tag().raw() == CREDENTIAL_VALUE)
                            .is_some_and(|child| {
                                child.with_value(|value| match value {
                                    ValueView::Structure(structure) => {
                                        crate::credential::validate_device_credential_value_view(
                                            &structure,
                                        )
                                        .is_ok()
                                    }
                                    _ => false,
                                })
                            })
                    }
                    _ => false,
                })
            })
        })
        .unwrap_or(true)
}

fn client_device_credential_password_is_text_string_if_present(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request
        .header()
        .with_authentication(|authentication| {
            authentication.children().iter().all(|field| {
                if field.tag().raw() != CREDENTIAL {
                    return true;
                }

                field.with_value(|value| match value {
                    ValueView::Structure(credential) => {
                        let Ok(credential_type) =
                            crate::credential::credential_type_from_view(&credential)
                        else {
                            return false;
                        };
                        if credential_type != CredentialType::Device {
                            return true;
                        }

                        credential
                            .children()
                            .iter()
                            .find(|child| child.tag().raw() == CREDENTIAL_VALUE)
                            .is_some_and(|child| {
                                child.with_value(|value| match value {
                                    ValueView::Structure(device) => {
                                        let mut password_seen = false;
                                        device
                                            .children()
                                            .iter()
                                            .filter(|field| {
                                                field.tag().raw() == DEVICE_CREDENTIAL_PASSWORD
                                            })
                                            .all(|field| {
                                                if password_seen {
                                                    return false;
                                                }
                                                password_seen = true;
                                                field.with_value(|value| {
                                                    matches!(value, ValueView::TextString(_))
                                                })
                                            })
                                    }
                                    _ => false,
                                })
                            })
                    }
                    _ => false,
                })
            })
        })
        .unwrap_or(true)
}

fn client_attestation_credential_has_evidence_if_present(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request
        .header()
        .with_authentication(|authentication| {
            authentication.children().iter().all(|field| {
                if field.tag().raw() != CREDENTIAL {
                    return true;
                }

                field.with_value(|value| match value {
                    ValueView::Structure(credential) => {
                        let Ok(credential_type) =
                            crate::credential::credential_type_from_view(&credential)
                        else {
                            return false;
                        };
                        if credential_type.raw() != ATTESTATION_CREDENTIAL_TYPE {
                            return true;
                        }

                        credential
                            .children()
                            .iter()
                            .find(|child| child.tag().raw() == CREDENTIAL_VALUE)
                            .is_some_and(|child| {
                                child.with_value(|value| match value {
                                    ValueView::Structure(attestation) => {
                                        crate::credential::validate_attestation_value_view(
                                            &attestation,
                                        )
                                        .is_ok()
                                    }
                                    _ => false,
                                })
                            })
                    }
                    _ => false,
                })
            })
        })
        .unwrap_or(true)
}

fn client_can_include_maximum_response_size(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §9.12 makes this request-header Integer optional; positive evidence
    // verifies that the model preserves a caller-supplied value.
    request.is_some()
}

fn client_sends_maximum_response_size_only_for_possible_large_replies(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    if request.header().maximum_response_size().is_none() {
        return true;
    }

    // KMIPKIT-0007 classifies Discover Versions as not likely to return a
    // large reply. Other operations are left to their operation specifications.
    request
        .batch_items()
        .any(|item| item.operation() != Some(DISCOVER_VERSIONS_OPERATION))
}

fn client_can_include_client_correlation_value(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §9.9 permits a caller-provided request string and does not require it to
    // be unique. Request-header validation enforces the Table 408 Text String.
    request.is_some()
}

fn client_uses_async_correlation_value_for_poll_or_cancel(
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
        if operation != POLL_OPERATION && operation != CANCEL_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let FieldPresence::Unique(correlation) =
                    field_presence(&payload, ASYNCHRONOUS_CORRELATION_VALUE)
                else {
                    return false;
                };
                correlation.with_value(|value| matches!(value, ValueView::ByteString(_)))
            })
            .unwrap_or(false)
    })
}

fn client_includes_unique_batch_item_id_for_multi_item_request(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    let mut batch_items = request.batch_items();
    batch_items.len() <= 1
        || batch_items.all(|item| item.with_unique_batch_item_id(|_| ()).is_some())
}

fn client_sets_batch_count_to_request_item_count(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    usize::try_from(request.header().batch_count()).ok() == Some(request.batch_items().len())
}

fn client_sets_batch_count_to_one_for_single_operation(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().len() != 1 || request.header().batch_count() == 1
}

fn client_defaults_batch_error_continuation_to_stop(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    let header = request.header();
    header.batch_error_continuation_option().is_some()
        || header.effective_batch_error_continuation_option()
            == BATCH_ERROR_CONTINUATION_OPTION_STOP
}

fn client_uses_assigned_batch_error_continuation_values(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request
        .header()
        .batch_error_continuation_option()
        .is_none_or(|value| {
            matches!(
                value,
                BATCH_ERROR_CONTINUATION_OPTION_CONTINUE
                    | BATCH_ERROR_CONTINUATION_OPTION_STOP
                    | BATCH_ERROR_CONTINUATION_OPTION_UNDO
            )
        })
}

fn client_includes_batch_error_continuation_only_for_multi_item_request(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    let header = request.header();
    header.batch_error_continuation_option().is_none() || header.batch_count() > 1
}

fn client_can_inspect_optional_result_message(
    _request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    // §9.17's client-use MAY adds no request or response validation rule.
    // Exact text access and redacted Debug output are covered by result tests;
    // caller-controlled logging stays outside this protocol profile validator.
    true
}

fn client_understands_response_operations(
    _request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some((request, response)) = response else {
        return true;
    };

    let request_items = request.batch_items().collect::<Vec<_>>();
    let response_items = response.batch_items().collect::<Vec<_>>();
    if response_items.is_empty() || response_items.len() > request_items.len() {
        return false;
    }

    let mut consumed_requests = vec![false; request_items.len()];
    // Index IDs once so response matching remains bounded for large batches.
    let mut request_indices_by_id = HashMap::new();
    for (index, request_item) in request_items.iter().copied().enumerate() {
        if let Some(request_id) = request_item.with_unique_batch_item_id(<[u8]>::to_vec)
            && request_indices_by_id.insert(request_id, index).is_some()
        {
            return false;
        }
    }

    for (response_index, response_item) in response_items.iter().copied().enumerate() {
        let Some(response_operation) = response_item.operation() else {
            return false;
        };
        if response_item.result_status().is_none() {
            return false;
        }

        let response_id = response_item.with_unique_batch_item_id(<[u8]>::to_vec);
        // Batch Order Option constrains execution order (§9.8); a supplied
        // Unique Batch Item ID remains the response correlation key (§9.21).
        let request_index = if let Some(response_id) = response_id {
            let Some(request_index) = request_indices_by_id.get(&response_id).copied() else {
                return false;
            };
            request_index
        } else if request_items.len() == 1
            && response_items.len() == 1
            && request_items[0]
                .with_unique_batch_item_id(<[u8]>::to_vec)
                .is_none()
        {
            0
        } else if request.header().effective_batch_order_option() {
            let request_item = request_items[response_index];
            if request_item
                .with_unique_batch_item_id(<[u8]>::to_vec)
                .is_some()
            {
                return false;
            }
            response_index
        } else {
            return false;
        };

        if consumed_requests[request_index] {
            return false;
        }
        consumed_requests[request_index] = true;

        let request_item = request_items[request_index];
        if request_item.operation() != Some(response_operation) {
            return false;
        }
    }

    true
}

fn client_response_preserves_x509_optional_values(
    _request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some((request, response)) = response else {
        return true;
    };

    request
        .batch_items()
        .zip(response.batch_items())
        .all(|(request_item, response_item)| {
            if request_item.operation() != Some(GET_ATTRIBUTES_OPERATION) {
                return true;
            }
            if response_item.operation() != Some(GET_ATTRIBUTES_OPERATION) {
                return false;
            }

            response_item
                .with_response_payload(|payload| {
                    payload
                        .children()
                        .iter()
                        .filter(|field| field.tag().raw() == ATTRIBUTES)
                        .all(|field| {
                            field.with_value(|value| match value {
                                ValueView::Structure(attributes) => attributes
                                    .children()
                                    .iter()
                                    .filter(|attribute| {
                                        matches!(
                                            attribute.tag().raw(),
                                            ISSUER_ALTERNATIVE_NAME
                                                | SUBJECT_ALTERNATIVE_NAME
                                                | SUBJECT_DISTINGUISHED_NAME
                                        )
                                    })
                                    .all(|attribute| {
                                        attribute.with_value(|value| {
                                            matches!(value, ValueView::ByteString(_))
                                        })
                                    }),
                                _ => false,
                            })
                        })
                })
                .unwrap_or(true)
        })
}

fn client_obtain_lease_response_values_are_typed(
    _request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some((request, response)) = response else {
        return true;
    };

    let request_items = request.batch_items().collect::<Vec<_>>();
    let response_items = response.batch_items().collect::<Vec<_>>();
    if response_items.is_empty() || response_items.len() > request_items.len() {
        return false;
    }

    let mut consumed_requests = vec![false; request_items.len()];
    let mut request_indices_by_id = HashMap::new();
    for (index, request_item) in request_items.iter().copied().enumerate() {
        if let Some(request_id) = request_item.with_unique_batch_item_id(<[u8]>::to_vec)
            && request_indices_by_id.insert(request_id, index).is_some()
        {
            return false;
        }
    }

    for response_item in response_items {
        let Some(response_operation) = response_item.operation() else {
            return false;
        };
        let Some(result_status) = response_item.result_status() else {
            return false;
        };

        let response_id = response_item.with_unique_batch_item_id(<[u8]>::to_vec);
        let request_index = if let Some(response_id) = response_id {
            let Some(request_index) = request_indices_by_id.get(&response_id).copied() else {
                return false;
            };
            request_index
        } else if request_items.len() == 1
            && request_items[0]
                .with_unique_batch_item_id(<[u8]>::to_vec)
                .is_none()
        {
            // A single-item request may omit the optional ID. For multiple
            // items, §8.3 Table 396 requires one and §9.21 requires echoing it.
            0
        } else {
            return false;
        };

        if consumed_requests
            .get(request_index)
            .copied()
            .unwrap_or(true)
        {
            return false;
        }
        consumed_requests[request_index] = true;
        let Some(request_operation) = request_items[request_index].operation() else {
            return false;
        };
        if request_operation != response_operation {
            return false;
        }
        if request_operation != OBTAIN_LEASE_OPERATION
            || result_status.raw() != RESULT_STATUS_SUCCESS
        {
            continue;
        }

        if crate::ObtainLeaseResponse::try_from_response_item(response_item).is_err() {
            return false;
        }
    }

    true
}
fn client_accepts_rng_seed_zero_data_length(
    _request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some((request, response)) = response else {
        return true;
    };

    let request_items = request.batch_items().collect::<Vec<_>>();
    if !request_items
        .iter()
        .any(|request_item| request_item.operation() == Some(RNG_SEED_OPERATION))
    {
        return true;
    }
    let response_items = response.batch_items().collect::<Vec<_>>();
    let mut consumed_responses = vec![false; response_items.len()];

    for (request_index, request_item) in request_items.iter().copied().enumerate() {
        if request_item.operation() != Some(RNG_SEED_OPERATION) {
            continue;
        }

        let response_index =
            if let Some(request_id) = request_item.with_unique_batch_item_id(<[u8]>::to_vec) {
                let matching_indices = response_items
                    .iter()
                    .enumerate()
                    .filter_map(|(index, item)| {
                        item.with_unique_batch_item_id(|response_id| {
                            response_id == request_id.as_slice()
                        })
                        .filter(|matches| *matches)
                        .map(|_| index)
                    })
                    .collect::<Vec<_>>();
                match matching_indices.as_slice() {
                    [] => continue,
                    [index] => *index,
                    _ => return false,
                }
            } else if request.header().effective_batch_order_option() {
                if request_index >= response_items.len() {
                    continue;
                }
                request_index
            } else {
                return false;
            };

        if consumed_responses[response_index] {
            return false;
        }
        consumed_responses[response_index] = true;
        let response_item = response_items[response_index];
        if response_item.operation() != Some(RNG_SEED_OPERATION) {
            return false;
        }
        let Some(status) = response_item.result_status() else {
            return false;
        };
        if status.raw() != RESULT_STATUS_SUCCESS {
            continue;
        }

        let data_length_is_well_formed = response_item
            .with_response_payload(|payload| {
                // §6.1.50 Table 320 requires Data Length; the paragraph above
                // the table permits a successful value of zero.
                let FieldPresence::Unique(data_length) =
                    field_presence(&payload, RNG_SEED_DATA_LENGTH)
                else {
                    return false;
                };
                data_length.with_value(|value| matches!(value, ValueView::Integer(_)))
            })
            .unwrap_or(false);
        if !data_length_is_well_formed {
            return false;
        }
    }
    true
}

fn client_create_key_pair_domain_parameters_have_valid_wire_types(
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
                                    attribute.with_value(|value| match value {
                                        ValueView::Structure(parameters) => parameters
                                            .children()
                                            .iter()
                                            .filter(|parameter| {
                                                parameter.tag().raw() == QLENGTH_DOMAIN_PARAMETER
                                            })
                                            .all(|parameter| {
                                                parameter.with_value(|value| {
                                                    matches!(value, ValueView::Integer(_))
                                                })
                                            }),
                                        _ => false,
                                    })
                                }),
                            _ => false,
                        })
                    })
            })
            .unwrap_or(false)
    })
}

fn client_repeated_attributes_are_representable(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    if let Some((request, response)) = response {
        return request.batch_items().zip(response.batch_items()).all(
            |(request_item, response_item)| {
                if request_item.operation() != Some(GET_ATTRIBUTES_OPERATION) {
                    return true;
                }
                if response_item.operation() != Some(GET_ATTRIBUTES_OPERATION) {
                    return false;
                }

                response_item
                    .with_response_payload(|payload| {
                        attribute_group_is_representable(&payload, ATTRIBUTES)
                    })
                    .unwrap_or(false)
            },
        );
    }

    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|request_item| {
        if request_item.operation() != Some(REGISTER_OPERATION) {
            return true;
        }

        request_item
            .with_request_payload(|payload| attribute_group_is_representable(&payload, ATTRIBUTES))
            .unwrap_or(false)
    })
}

fn client_object_group_names_are_representable(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    const OBJECT_GROUP_ATTRIBUTE_TAG: u32 = 0x0042_0056;
    const OBJECT_GROUP_ASSIGNMENT_OPERATIONS: [u32; 7] = [
        CREATE_OPERATION,
        REGISTER_OPERATION,
        DERIVE_KEY_OPERATION,
        CERTIFY_OPERATION,
        RECERTIFY_OPERATION,
        REKEY_OPERATION,
        REKEY_KEY_PAIR_OPERATION,
    ];

    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if !OBJECT_GROUP_ASSIGNMENT_OPERATIONS.contains(&operation)
            && operation != CREATE_KEY_PAIR_OPERATION
            && operation != ADD_ATTRIBUTE_OPERATION
            && operation != MODIFY_ATTRIBUTE_OPERATION
            && operation != SET_ATTRIBUTE_OPERATION
        {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                if OBJECT_GROUP_ASSIGNMENT_OPERATIONS.contains(&operation) {
                    return object_group_attribute_group_is_text(
                        &payload,
                        ATTRIBUTES,
                        OBJECT_GROUP_ATTRIBUTE_TAG,
                    );
                }

                match operation {
                    CREATE_KEY_PAIR_OPERATION => {
                        CREATE_KEY_PAIR_ATTRIBUTE_GROUPS
                            .into_iter()
                            .all(|attribute_group| {
                                object_group_attribute_group_is_text(
                                    &payload,
                                    attribute_group,
                                    OBJECT_GROUP_ATTRIBUTE_TAG,
                                )
                            })
                    }
                    ADD_ATTRIBUTE_OPERATION | SET_ATTRIBUTE_OPERATION => {
                        object_group_attribute_group_is_text(
                            &payload,
                            NEW_ATTRIBUTE,
                            OBJECT_GROUP_ATTRIBUTE_TAG,
                        )
                    }
                    MODIFY_ATTRIBUTE_OPERATION => {
                        object_group_attribute_group_is_text(
                            &payload,
                            CURRENT_ATTRIBUTE,
                            OBJECT_GROUP_ATTRIBUTE_TAG,
                        ) && object_group_attribute_group_is_text(
                            &payload,
                            NEW_ATTRIBUTE,
                            OBJECT_GROUP_ATTRIBUTE_TAG,
                        )
                    }
                    _ => true,
                }
            })
            .unwrap_or(false)
    })
}

fn object_group_attribute_group_is_text(
    payload: &StructureView<'_>,
    attribute_group_tag: u32,
    object_group_attribute_tag: u32,
) -> bool {
    match field_presence(payload, attribute_group_tag) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(attributes) => attributes.with_value(|value| match value {
            ValueView::Structure(attributes) => attributes.children().iter().all(|attribute| {
                attribute.tag().raw() != object_group_attribute_tag
                    || attribute.item_type() == ItemType::TextString
            }),
            _ => false,
        }),
    }
}

fn client_create_key_pair_attribute_group_is_representable(
    request: Option<&RequestMessage>,
    attribute_group_tag: u32,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|request_item| {
        let Some(operation) = request_item.operation() else {
            return false;
        };
        if operation != CREATE_KEY_PAIR_OPERATION {
            return true;
        }

        request_item
            .with_request_payload(|payload| {
                attribute_group_is_representable(&payload, attribute_group_tag)
            })
            .unwrap_or(false)
    })
}

fn client_create_key_pair_common_attributes_are_representable(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_create_key_pair_attribute_group_is_representable(
        request,
        CREATE_KEY_PAIR_COMMON_ATTRIBUTES,
    )
}

fn client_create_key_pair_private_key_attributes_are_representable(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_create_key_pair_attribute_group_is_representable(
        request,
        CREATE_KEY_PAIR_PRIVATE_KEY_ATTRIBUTES,
    )
}

fn client_create_key_pair_public_key_attributes_are_representable(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_create_key_pair_attribute_group_is_representable(
        request,
        CREATE_KEY_PAIR_PUBLIC_KEY_ATTRIBUTES,
    )
}

fn client_create_key_pair_differing_attributes_are_representable(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_create_key_pair_attribute_group_is_representable(
        request,
        CREATE_KEY_PAIR_PRIVATE_KEY_ATTRIBUTES,
    ) && client_create_key_pair_attribute_group_is_representable(
        request,
        CREATE_KEY_PAIR_PUBLIC_KEY_ATTRIBUTES,
    )
}

fn client_create_key_pair_table_191_values_match(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|request_item| {
        let Some(operation) = request_item.operation() else {
            return false;
        };
        if operation != CREATE_KEY_PAIR_OPERATION {
            return true;
        }

        request_item
            .with_request_payload(|payload| {
                create_key_pair_table_191_payload_is_consistent(&payload)
            })
            .unwrap_or(false)
    })
}

fn create_key_pair_table_191_payload_is_consistent(payload: &StructureView<'_>) -> bool {
    let mut groups = [None; 3];
    for (group, tag) in groups.iter_mut().zip(CREATE_KEY_PAIR_ATTRIBUTE_GROUPS) {
        *group = match field_presence(payload, tag) {
            FieldPresence::Missing => None,
            FieldPresence::Unique(group) if group.item_type() == ItemType::Structure => Some(group),
            FieldPresence::Unique(_) | FieldPresence::Duplicate => return false,
        };
    }
    let common = groups[0];
    let private = groups[1];
    let public = groups[2];

    CREATE_KEY_PAIR_TABLE_191_ATTRIBUTES.into_iter().all(|tag| {
        with_create_key_pair_attribute_group(common, |common| {
            with_create_key_pair_attribute_group(private, |private| {
                with_create_key_pair_attribute_group(public, |public| {
                    match (
                        effective_table_191_attribute(common.as_ref(), private.as_ref(), tag),
                        effective_table_191_attribute(common.as_ref(), public.as_ref(), tag),
                    ) {
                        (FieldPresence::Missing, FieldPresence::Missing) => true,
                        (FieldPresence::Unique(private), FieldPresence::Unique(public)) => {
                            crate::create_key_pair::values_equal(private, public)
                        }
                        _ => false,
                    }
                })
            })
        })
    })
}

fn with_create_key_pair_attribute_group(
    group: Option<&Item>,
    callback: impl for<'a> FnOnce(Option<StructureView<'a>>) -> bool,
) -> bool {
    match group {
        None => callback(None),
        Some(group) => group.with_value(|value| match value {
            ValueView::Structure(attributes) => callback(Some(attributes)),
            _ => false,
        }),
    }
}

fn effective_table_191_attribute<'a>(
    common: Option<&'a StructureView<'_>>,
    specific: Option<&'a StructureView<'_>>,
    tag: u32,
) -> FieldPresence<'a> {
    match (
        table_191_attribute_in_group(specific, tag),
        table_191_attribute_in_group(common, tag),
    ) {
        (FieldPresence::Duplicate, _) | (_, FieldPresence::Duplicate) => FieldPresence::Duplicate,
        (FieldPresence::Unique(attribute), _) => FieldPresence::Unique(attribute),
        (FieldPresence::Missing, FieldPresence::Unique(attribute)) => {
            FieldPresence::Unique(attribute)
        }
        (FieldPresence::Missing, FieldPresence::Missing) => FieldPresence::Missing,
    }
}

fn table_191_attribute_in_group<'a>(
    group: Option<&'a StructureView<'_>>,
    tag: u32,
) -> FieldPresence<'a> {
    let Some(group) = group else {
        return FieldPresence::Missing;
    };
    field_presence(group, tag)
}

fn attribute_group_is_representable(payload: &StructureView<'_>, group_tag: u32) -> bool {
    match field_presence(payload, group_tag) {
        FieldPresence::Missing => true,
        FieldPresence::Unique(group) => {
            group.with_value(|value| matches!(value, ValueView::Structure(_)))
        }
        FieldPresence::Duplicate => false,
    }
}

fn derive_key_specifies_cryptographic_length(
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
        if operation != DERIVE_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let FieldPresence::Unique(attributes) = field_presence(&payload, ATTRIBUTES) else {
                    return false;
                };
                attributes.with_value(|value| match value {
                    ValueView::Structure(attributes) => {
                        let FieldPresence::Unique(length) =
                            field_presence(&attributes, CRYPTOGRAPHIC_LENGTH)
                        else {
                            return false;
                        };
                        length.with_value(|value| matches!(value, ValueView::Integer(_)))
                    }
                    _ => false,
                })
            })
            .unwrap_or(false)
    })
}

fn payload_has_explicit_unique_identifier(payload: &StructureView<'_>) -> bool {
    // Table 205 permits repeats, requires a source ID, and prohibits the
    // ID Placeholder enumeration in every Unique Identifier occurrence.
    let mut found_identifier = false;
    for field in payload
        .children()
        .iter()
        .filter(|field| field.tag().raw() == UNIQUE_IDENTIFIER)
    {
        found_identifier = true;
        let is_id_placeholder = field.with_value(|value| {
            matches!(
                value,
                ValueView::Enumeration(value) if *value == ID_PLACEHOLDER_ENUMERATION_VALUE
            )
        });
        if is_id_placeholder {
            return false;
        }
    }
    found_identifier
}

fn derive_key_specifies_explicit_source_identifiers(
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
        if operation != DERIVE_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| payload_has_explicit_unique_identifier(&payload))
            .unwrap_or(false)
    })
}

fn derive_key_repeated_unique_identifiers_are_usable(
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
        if operation != DERIVE_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == UNIQUE_IDENTIFIER)
                    .all(|field| {
                        field.with_value(|value| unique_identifier_has_supported_encoding(&value))
                    })
            })
            .unwrap_or(false)
    })
}

fn unique_identifier_has_supported_encoding(value: &ValueView<'_>) -> bool {
    matches!(
        value,
        ValueView::TextString(_) | ValueView::Enumeration(_) | ValueView::Integer(_)
    )
}

fn derive_key_output_object_type(payload: &StructureView<'_>) -> Option<u32> {
    let FieldPresence::Unique(object_type) = field_presence(payload, DERIVE_KEY_OBJECT_TYPE) else {
        return None;
    };
    object_type.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    })
}

fn derive_key_includes_required_object_type(
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
        if operation != DERIVE_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| derive_key_output_object_type(&payload).is_some())
            .unwrap_or(false)
    })
}

fn derive_key_includes_required_derivation_method(
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
        if operation != DERIVE_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let FieldPresence::Unique(method) =
                    field_presence(&payload, DERIVE_KEY_DERIVATION_METHOD)
                else {
                    return false;
                };
                method.with_value(|value| matches!(value, ValueView::Enumeration(_)))
            })
            .unwrap_or(false)
    })
}

fn derive_key_includes_required_derivation_parameters(
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
        if operation != DERIVE_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let FieldPresence::Unique(parameters) =
                    field_presence(&payload, DERIVE_KEY_DERIVATION_PARAMETERS)
                else {
                    return false;
                };
                parameters.with_value(|value| matches!(value, ValueView::Structure(_)))
            })
            .unwrap_or(false)
    })
}

fn derive_key_hash_method_selects_explicit_prf(
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
        if operation != DERIVE_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let FieldPresence::Unique(method) =
                    field_presence(&payload, DERIVE_KEY_DERIVATION_METHOD)
                else {
                    // The separate §6.1.14 method rule owns missing, duplicate, or malformed method fields.
                    return true;
                };
                method.with_value(|value| match value {
                    ValueView::Enumeration(method) if *method == DERIVE_KEY_HASH_METHOD => {
                        derive_key_hash_method_has_explicit_prf(&payload)
                    }
                    // §7.12's AES-CBC condition depends on the server-held derivation key; do not
                    // infer that condition from the requested result attributes or ENCRYPT method.
                    _ => true,
                })
            })
            .unwrap_or(false)
    })
}

fn derive_key_hash_method_has_explicit_prf(payload: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(parameters) =
        field_presence(payload, DERIVE_KEY_DERIVATION_PARAMETERS)
    else {
        return false;
    };
    parameters.with_value(|value| {
        let ValueView::Structure(parameters) = value else {
            return false;
        };
        let FieldPresence::Unique(cryptographic_parameters) =
            field_presence(&parameters, CRYPTOGRAPHIC_PARAMETERS)
        else {
            return false;
        };
        cryptographic_parameters.with_value(|value| {
            let ValueView::Structure(cryptographic_parameters) = value else {
                return false;
            };
            let FieldPresence::Unique(hashing_algorithm) =
                field_presence(&cryptographic_parameters, HASHING_ALGORITHM)
            else {
                return false;
            };
            hashing_algorithm.with_value(|value| matches!(value, ValueView::Enumeration(_)))
        })
    })
}

fn derive_key_optional_input_forms_are_representable(
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
        if operation != DERIVE_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let identifiers_have_supported_encodings = payload.children().iter().all(|field| {
                    field.tag().raw() != UNIQUE_IDENTIFIER
                        || field
                            .with_value(|value| unique_identifier_has_supported_encoding(&value))
                });
                if !identifiers_have_supported_encodings {
                    return false;
                }

                match field_presence(&payload, DERIVE_KEY_DERIVATION_PARAMETERS) {
                    FieldPresence::Missing => true,
                    FieldPresence::Duplicate => false,
                    FieldPresence::Unique(parameters) => parameters.with_value(|value| {
                        let ValueView::Structure(parameters) = value else {
                            return false;
                        };
                        parameters.children().iter().all(|field| {
                            field.tag().raw() != DERIVATION_DATA
                                || field
                                    .with_value(|value| matches!(value, ValueView::ByteString(_)))
                        })
                    }),
                }
            })
            .unwrap_or(false)
    })
}

fn client_can_signal_asynchronous_response_support(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    match request.header().asynchronous_indicator() {
        None
        | Some(
            ASYNCHRONOUS_INDICATOR_MANDATORY..=ASYNCHRONOUS_INDICATOR_PROHIBITED
            | ENUMERATION_EXTENSION_MIN..=ENUMERATION_EXTENSION_MAX,
        ) => true,
        Some(_) => false,
    }
}

fn derive_key_includes_required_attributes(
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
        if operation != DERIVE_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let FieldPresence::Unique(attributes) = field_presence(&payload, ATTRIBUTES) else {
                    return false;
                };
                attributes.with_value(|value| matches!(value, ValueView::Structure(_)))
            })
            .unwrap_or(false)
    })
}

fn derive_key_specifies_cryptographic_algorithm_for_key_output(
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
        if operation != DERIVE_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let Some(object_type) = derive_key_output_object_type(&payload) else {
                    return false;
                };
                if object_type == SECRET_DATA_OBJECT_TYPE {
                    return true;
                }
                if object_type != SYMMETRIC_KEY_OBJECT_TYPE {
                    return false;
                }

                let FieldPresence::Unique(attributes) = field_presence(&payload, ATTRIBUTES) else {
                    return false;
                };
                attributes.with_value(|value| match value {
                    ValueView::Structure(attributes) => {
                        let FieldPresence::Unique(algorithm) =
                            field_presence(&attributes, CRYPTOGRAPHIC_ALGORITHM)
                        else {
                            return false;
                        };
                        algorithm.with_value(|value| matches!(value, ValueView::Enumeration(_)))
                    }
                    _ => false,
                })
            })
            .unwrap_or(false)
    })
}

fn derive_key_symmetric_output_specifies_cryptographic_attributes(
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
        if operation != DERIVE_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                match derive_key_output_object_type(&payload) {
                    Some(SYMMETRIC_KEY_OBJECT_TYPE) => {
                        let FieldPresence::Unique(attributes) =
                            field_presence(&payload, ATTRIBUTES)
                        else {
                            return false;
                        };
                        attributes.with_value(|value| match value {
                            ValueView::Structure(attributes) => {
                                let FieldPresence::Unique(length) =
                                    field_presence(&attributes, CRYPTOGRAPHIC_LENGTH)
                                else {
                                    return false;
                                };
                                let FieldPresence::Unique(algorithm) =
                                    field_presence(&attributes, CRYPTOGRAPHIC_ALGORITHM)
                                else {
                                    return false;
                                };
                                length.with_value(|value| matches!(value, ValueView::Integer(_)))
                                    && algorithm.with_value(|value| {
                                        matches!(value, ValueView::Enumeration(_))
                                    })
                            }
                            _ => false,
                        })
                    }
                    // This conditional requirement applies only to a known
                    // Symmetric Key output. Preserve unknown Object Type values
                    // without interpreting them as symmetric keys.
                    Some(_) => true,
                    // A malformed or absent Object Type cannot establish the
                    // requirement's applicability, so reject it fail-closed.
                    None => false,
                }
            })
            .unwrap_or(false)
    })
}

fn derive_key_secret_data_uses_total_cryptographic_length(
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
        if operation != DERIVE_KEY_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let Some(object_type) = derive_key_output_object_type(&payload) else {
                    return false;
                };
                if object_type != SECRET_DATA_OBJECT_TYPE {
                    return true;
                }

                let FieldPresence::Unique(attributes) = field_presence(&payload, ATTRIBUTES) else {
                    return false;
                };
                attributes.with_value(|value| match value {
                    ValueView::Structure(attributes) => {
                        let FieldPresence::Unique(length) =
                            field_presence(&attributes, CRYPTOGRAPHIC_LENGTH)
                        else {
                            return false;
                        };
                        length.with_value(|value| matches!(value, ValueView::Integer(_)))
                    }
                    _ => false,
                })
            })
            .unwrap_or(false)
    })
}

fn client_opaque_object_context_custom_attributes(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    if let Some((request, response)) = response {
        return request.batch_items().zip(response.batch_items()).all(
            |(request_item, response_item)| {
                if request_item.operation() != Some(GET_ATTRIBUTES_OPERATION) {
                    return true;
                }
                if response_item.operation() != Some(GET_ATTRIBUTES_OPERATION) {
                    return false;
                }

                response_item
                    .with_response_payload(|payload| match field_presence(&payload, ATTRIBUTES) {
                        FieldPresence::Missing => true,
                        FieldPresence::Unique(attributes) => attributes.with_value(|value| {
                            matches!(value, ValueView::Structure(attributes)
                                    if vendor_attributes_are_well_formed(&attributes))
                        }),
                        FieldPresence::Duplicate => false,
                    })
                    .unwrap_or(true)
            },
        );
    }

    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if operation != REGISTER_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| register_opaque_context_is_well_formed(&payload))
            .unwrap_or(false)
    })
}

fn client_register_opaque_object_has_required_data_fields(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    if response.is_some() {
        return true;
    }

    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        if batch_item.operation() != Some(REGISTER_OPERATION) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let FieldPresence::Unique(object_type) =
                    field_presence(&payload, REGISTER_OBJECT_TYPE)
                else {
                    return true;
                };
                let Some(object_type) = object_type.with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                }) else {
                    return true;
                };
                if object_type != OPAQUE_OBJECT_TYPE {
                    return true;
                }

                let FieldPresence::Unique(opaque_object) =
                    field_presence(&payload, REGISTER_OPAQUE_OBJECT)
                else {
                    return false;
                };
                opaque_object.with_value(|value| match value {
                    ValueView::Structure(opaque_object) => {
                        let FieldPresence::Unique(data_type) =
                            field_presence(&opaque_object, OPAQUE_DATA_TYPE)
                        else {
                            return false;
                        };
                        let FieldPresence::Unique(data_value) =
                            field_presence(&opaque_object, OPAQUE_DATA_VALUE)
                        else {
                            return false;
                        };
                        data_type.with_value(|value| matches!(value, ValueView::Enumeration(_)))
                            && data_value
                                .with_value(|value| matches!(value, ValueView::ByteString(_)))
                    }
                    _ => false,
                })
            })
            .unwrap_or(false)
    })
}

fn client_register_has_required_object_type_and_attributes(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    if response.is_some() {
        return true;
    }

    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        if batch_item.operation() != Some(REGISTER_OPERATION) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                register_payload_has_required_table_291_fields(&payload)
            })
            .unwrap_or(false)
    })
}

fn register_payload_has_required_table_291_fields(payload: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(object_type) = field_presence(payload, REGISTER_OBJECT_TYPE) else {
        return false;
    };
    let FieldPresence::Unique(attributes) = field_presence(payload, REGISTER_ATTRIBUTES) else {
        return false;
    };

    object_type.with_value(|value| matches!(value, ValueView::Enumeration(_)))
        && attributes.with_value(|value| matches!(value, ValueView::Structure(_)))
}

fn register_opaque_context_is_well_formed(payload: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(object_type) = field_presence(payload, REGISTER_OBJECT_TYPE) else {
        return true;
    };
    let Some(object_type) = object_type.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    }) else {
        return true;
    };
    if object_type != OPAQUE_OBJECT_TYPE {
        return true;
    }

    match field_presence(payload, REGISTER_ATTRIBUTES) {
        FieldPresence::Missing => true,
        FieldPresence::Unique(attributes) => attributes.with_value(|value| match value {
            ValueView::Structure(attributes) => vendor_attributes_are_well_formed(&attributes),
            _ => false,
        }),
        FieldPresence::Duplicate => false,
    }
}

fn client_opaque_object_context_managed_attributes(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    if response.is_some() {
        return true;
    }

    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        if batch_item.operation() != Some(REGISTER_OPERATION) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                register_managed_opaque_attributes_are_well_formed(&payload)
            })
            .unwrap_or(false)
    })
}

fn register_managed_opaque_attributes_are_well_formed(payload: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(object_type) = field_presence(payload, REGISTER_OBJECT_TYPE) else {
        return false;
    };
    let Some(object_type) = object_type.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    }) else {
        return false;
    };
    if object_type != OPAQUE_OBJECT_TYPE {
        return true;
    }

    match field_presence(payload, REGISTER_ATTRIBUTES) {
        FieldPresence::Missing => true,
        FieldPresence::Unique(attributes) => attributes.with_value(|value| match value {
            ValueView::Structure(attributes) => {
                managed_opaque_attribute_types_are_well_formed(&attributes)
            }
            _ => false,
        }),
        FieldPresence::Duplicate => false,
    }
}

fn managed_opaque_attribute_types_are_well_formed(attributes: &StructureView<'_>) -> bool {
    [
        (CRYPTOGRAPHIC_USAGE_MASK, false),
        (CRYPTOGRAPHIC_ALGORITHM, true),
        (CRYPTOGRAPHIC_LENGTH, false),
    ]
    .into_iter()
    .all(|(tag, enumeration)| match field_presence(attributes, tag) {
        FieldPresence::Missing => true,
        FieldPresence::Unique(field) => field.with_value(|value| {
            if enumeration {
                matches!(value, ValueView::Enumeration(_))
            } else {
                matches!(value, ValueView::Integer(_))
            }
        }),
        FieldPresence::Duplicate => false,
    })
}

fn vendor_attributes_are_well_formed(attributes: &StructureView<'_>) -> bool {
    attributes
        .children()
        .iter()
        .filter(|attribute| attribute.tag().raw() == VENDOR_ATTRIBUTE)
        .all(|attribute| {
            attribute.with_value(|value| match value {
                ValueView::Structure(vendor_attribute) => {
                    crate::attribute::validate_vendor_attribute(&vendor_attribute).is_ok()
                }
                _ => false,
            })
        })
}

fn import_preserves_wrapped_secret_data_shape(
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
        if operation != IMPORT_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| import_payload_preserves_secret_data_wrapping(&payload))
            .unwrap_or(false)
    })
}

fn client_import_preserves_wrapped_object_and_attributes(
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
        if operation != IMPORT_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                import_payload_preserves_wrapped_object_and_attributes(&payload)
            })
            .unwrap_or(false)
    })
}

fn client_archive_request_is_well_formed(
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
        if operation != ARCHIVE_OPERATION {
            return true;
        }
        batch_item
            .with_request_payload(
                |payload| match field_presence(&payload, UNIQUE_IDENTIFIER) {
                    FieldPresence::Missing => true,
                    FieldPresence::Unique(identifier) => identifier.with_value(|value| {
                        matches!(
                            value,
                            ValueView::Enumeration(_)
                                | ValueView::Integer(_)
                                | ValueView::TextString(_)
                        )
                    }),
                    FieldPresence::Duplicate => false,
                },
            )
            .unwrap_or(false)
    })
}

fn import_payload_preserves_wrapped_object_and_attributes(payload: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(attributes) = field_presence(payload, ATTRIBUTES) else {
        return false;
    };
    if !attributes.with_value(|value| matches!(value, ValueView::Structure(_))) {
        return false;
    }

    let FieldPresence::Unique(object_type) = field_presence(payload, IMPORT_OBJECT_TYPE) else {
        return false;
    };
    let Some(object_type) = object_type.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    }) else {
        return false;
    };
    let Some(object_tag) = key_block_object_tag(object_type) else {
        return true;
    };

    let FieldPresence::Unique(object) = field_presence(payload, object_tag) else {
        return false;
    };
    object.with_value(|value| match value {
        ValueView::Structure(object) => match field_presence(&object, KEY_BLOCK) {
            FieldPresence::Unique(key_block) => key_block.with_value(|value| match value {
                ValueView::Structure(key_block) => key_block_preserves_wrapping(&key_block),
                _ => false,
            }),
            FieldPresence::Missing | FieldPresence::Duplicate => false,
        },
        _ => false,
    })
}

fn client_key_wrapping_data_is_absent_or_has_required_key_information(
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
        if !matches!(operation, REGISTER_OPERATION | IMPORT_OPERATION) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                key_wrapping_data_has_required_key_information(&payload)
            })
            .unwrap_or(false)
    })
}

fn client_no_encoding_excludes_key_value_attributes(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_key_wrapping_rule_holds(request, KeyWrappingRule::NoEncodingExcludesAttributes)
}

fn client_wrapped_key_value_requires_key_wrapping_data(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_key_wrapping_rule_holds(request, KeyWrappingRule::WrappedValueRequiresData)
}

fn client_key_wrapping_data_is_only_present_for_wrapped_key_value(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_key_wrapping_rule_holds(request, KeyWrappingRule::DataOnlyForWrappedValue)
}

fn client_key_wrapping_rule_holds(request: Option<&RequestMessage>, rule: KeyWrappingRule) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if !matches!(operation, REGISTER_OPERATION | IMPORT_OPERATION) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| key_wrapping_rule_holds_for_payload(&payload, rule))
            .unwrap_or(false)
    })
}

fn key_wrapping_rule_holds_for_payload(payload: &StructureView<'_>, rule: KeyWrappingRule) -> bool {
    let object_type_tag = match field_presence(payload, REGISTER_OBJECT_TYPE) {
        FieldPresence::Missing | FieldPresence::Duplicate => return true,
        FieldPresence::Unique(object_type) => object_type.with_value(|value| match value {
            ValueView::Enumeration(value) => Some(*value),
            _ => None,
        }),
    };
    let Some(object_type_tag) = object_type_tag else {
        return true;
    };
    let Some(object_tag) = key_block_object_tag(object_type_tag) else {
        return true;
    };

    match field_presence(payload, object_tag) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(object) => object.with_value(|value| match value {
            ValueView::Structure(object) => match field_presence(&object, KEY_BLOCK) {
                FieldPresence::Missing => true,
                FieldPresence::Duplicate => false,
                FieldPresence::Unique(key_block) => key_block.with_value(|value| match value {
                    ValueView::Structure(key_block) => {
                        key_wrapping_rule_holds_for_key_block(&key_block, rule)
                    }
                    _ => false,
                }),
            },
            _ => false,
        }),
    }
}

fn key_block_object_tag(object_type: u32) -> Option<u32> {
    match object_type {
        SYMMETRIC_KEY_OBJECT_TYPE => Some(SYMMETRIC_KEY),
        PUBLIC_KEY_OBJECT_TYPE => Some(PUBLIC_KEY),
        PRIVATE_KEY_OBJECT_TYPE => Some(PRIVATE_KEY),
        SPLIT_KEY_OBJECT_TYPE => Some(SPLIT_KEY),
        SECRET_DATA_OBJECT_TYPE => Some(SECRET_DATA),
        PGP_KEY_OBJECT_TYPE => Some(PGP_KEY),
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum KeyWrappingRule {
    WrappedValueRequiresData,
    DataOnlyForWrappedValue,
    NoEncodingExcludesAttributes,
}

fn key_wrapping_rule_holds_for_key_block(
    key_block: &StructureView<'_>,
    rule: KeyWrappingRule,
) -> bool {
    let wrapping_data = field_presence(key_block, KEY_WRAPPING_DATA);
    if matches!(wrapping_data, FieldPresence::Duplicate) {
        return false;
    }

    match field_presence(key_block, KEY_VALUE) {
        FieldPresence::Missing => match rule {
            KeyWrappingRule::DataOnlyForWrappedValue => {
                matches!(wrapping_data, FieldPresence::Missing)
            }
            KeyWrappingRule::WrappedValueRequiresData
            | KeyWrappingRule::NoEncodingExcludesAttributes => true,
        },
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(key_value) => key_value.with_value(|value| match value {
            ValueView::ByteString(_) => match rule {
                KeyWrappingRule::WrappedValueRequiresData => {
                    matches!(wrapping_data, FieldPresence::Unique(_))
                }
                KeyWrappingRule::DataOnlyForWrappedValue
                | KeyWrappingRule::NoEncodingExcludesAttributes => true,
            },
            ValueView::Structure(key_value) => match rule {
                KeyWrappingRule::WrappedValueRequiresData => true,
                KeyWrappingRule::DataOnlyForWrappedValue => {
                    matches!(wrapping_data, FieldPresence::Missing)
                }
                KeyWrappingRule::NoEncodingExcludesAttributes => {
                    no_encoding_key_value_has_no_attributes(key_block, &key_value)
                }
            },
            _ => false,
        }),
    }
}

fn no_encoding_key_value_has_no_attributes(
    key_block: &StructureView<'_>,
    key_value: &StructureView<'_>,
) -> bool {
    match field_presence(key_block, KEY_WRAPPING_DATA) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(wrapping_data) => wrapping_data.with_value(|value| {
            let ValueView::Structure(wrapping_data) = value else {
                return false;
            };
            match field_presence(&wrapping_data, ENCODING_OPTION) {
                FieldPresence::Missing => true,
                FieldPresence::Duplicate => false,
                FieldPresence::Unique(encoding_option) => {
                    encoding_option.with_value(|value| match value {
                        ValueView::Enumeration(value) if *value == NO_ENCODING_OPTION => {
                            matches!(
                                field_presence(key_value, ATTRIBUTES),
                                FieldPresence::Missing
                            )
                        }
                        ValueView::Enumeration(_) => true,
                        _ => false,
                    })
                }
            }
        }),
    }
}

fn import_preserves_pgp_key_form_and_opaque_bytes(
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
        if operation != IMPORT_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| import_payload_preserves_pgp_key(&payload))
            .unwrap_or(false)
    })
}

fn import_payload_preserves_pgp_key(payload: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(object_type) = field_presence(payload, IMPORT_OBJECT_TYPE) else {
        return false;
    };
    let Some(object_type) = object_type.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    }) else {
        return false;
    };
    if object_type != PGP_KEY_OBJECT_TYPE {
        return true;
    }

    let FieldPresence::Unique(pgp_key) = field_presence(payload, PGP_KEY) else {
        return false;
    };
    pgp_key.with_value(|value| match value {
        ValueView::Structure(pgp_key) => pgp_key_preserves_required_fields(&pgp_key),
        _ => false,
    })
}

fn client_key_block_algorithm_requires_length(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_key_block_direct_field_requires_companion(
        request,
        CRYPTOGRAPHIC_ALGORITHM,
        CRYPTOGRAPHIC_LENGTH,
    )
}

fn client_transparent_rsa_private_key_has_material_alternative(
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
        if !matches!(operation, REGISTER_OPERATION | IMPORT_OPERATION) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let FieldPresence::Unique(object_type) =
                    field_presence(&payload, REGISTER_OBJECT_TYPE)
                else {
                    return false;
                };
                let Some(object_type) = object_type.with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                }) else {
                    return false;
                };
                if object_type != PRIVATE_KEY_OBJECT_TYPE {
                    return true;
                }

                let FieldPresence::Unique(private_key) = field_presence(&payload, PRIVATE_KEY)
                else {
                    return false;
                };
                private_key.with_value(|value| match value {
                    ValueView::Structure(private_key) => {
                        match field_presence(&private_key, KEY_BLOCK) {
                            FieldPresence::Unique(key_block) => {
                                key_block.with_value(|value| match value {
                                    ValueView::Structure(key_block) => {
                                        transparent_rsa_key_block_has_material_alternative(
                                            &key_block,
                                        )
                                    }
                                    _ => false,
                                })
                            }
                            FieldPresence::Missing | FieldPresence::Duplicate => false,
                        }
                    }
                    _ => false,
                })
            })
            .unwrap_or(false)
    })
}

fn transparent_rsa_key_block_has_material_alternative(key_block: &StructureView<'_>) -> bool {
    let key_format_type = match field_presence(key_block, KEY_FORMAT_TYPE) {
        FieldPresence::Missing | FieldPresence::Duplicate => return false,
        FieldPresence::Unique(item) => item.with_value(|value| match value {
            ValueView::Enumeration(value) => Some(*value),
            _ => None,
        }),
    };
    let Some(key_format_type) = key_format_type else {
        return false;
    };
    if key_format_type != RSA_PRIVATE_KEY_FORMAT {
        return true;
    }

    let FieldPresence::Unique(key_value) = field_presence(key_block, KEY_VALUE) else {
        return false;
    };
    key_value.with_value(|value| match value {
        ValueView::ByteString(_) => match field_presence(key_block, KEY_WRAPPING_DATA) {
            FieldPresence::Unique(wrapping_data) => {
                wrapping_data.with_value(|value| matches!(value, ValueView::Structure(_)))
            }
            FieldPresence::Missing | FieldPresence::Duplicate => false,
        },
        ValueView::Structure(key_value) => {
            let FieldPresence::Unique(key_material) = field_presence(&key_value, KEY_MATERIAL)
            else {
                return false;
            };
            key_material.with_value(|value| match value {
                ValueView::Structure(key_material) => {
                    transparent_rsa_private_key_material_has_alternative(&key_material)
                }
                _ => false,
            })
        }
        _ => false,
    })
}

fn transparent_rsa_private_key_material_has_alternative(key_material: &StructureView<'_>) -> bool {
    let has_private_exponent = has_unique_big_integer(key_material, RSA_PRIVATE_EXPONENT);
    let has_prime_factors = has_unique_big_integer(key_material, RSA_PRIME_P)
        && has_unique_big_integer(key_material, RSA_PRIME_Q);
    let has_prime_exponents = has_unique_big_integer(key_material, RSA_PRIME_EXPONENT_P)
        && has_unique_big_integer(key_material, RSA_PRIME_EXPONENT_Q);

    has_private_exponent || has_prime_factors || has_prime_exponents
}

fn has_unique_big_integer(structure: &StructureView<'_>, tag: u32) -> bool {
    match field_presence(structure, tag) {
        FieldPresence::Unique(item) => {
            item.with_value(|value| matches!(value, ValueView::BigInteger(_)))
        }
        FieldPresence::Missing | FieldPresence::Duplicate => false,
    }
}

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const HASH_OPERATION: u32 = 0x0000_0027;
const MAC_OPERATION: u32 = 0x0000_0023;
const MAC_VERIFY_OPERATION: u32 = 0x0000_0024;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const INITIAL_INDICATOR: u32 = 0x0042_00D7;
const FINAL_INDICATOR: u32 = 0x0042_00D8;
const AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA: u32 = 0x0042_00FE;
const AUTHENTICATED_ENCRYPTION_TAG: u32 = 0x0042_00FF;

fn client_multipart_continuation_has_server_correlation(
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
        if !matches!(
            operation,
            HASH_OPERATION
                | MAC_OPERATION
                | MAC_VERIFY_OPERATION
                | ENCRYPT_OPERATION
                | DECRYPT_OPERATION
                | SIGN_OPERATION
                | SIGNATURE_VERIFY_OPERATION
        ) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let correlation_value = match field_presence(&payload, CORRELATION_VALUE) {
                    FieldPresence::Missing => None,
                    FieldPresence::Duplicate => return false,
                    FieldPresence::Unique(value) => {
                        Some(value.with_value(|value| matches!(value, ValueView::ByteString(_))))
                    }
                };
                let initial_indicator = match field_presence(&payload, INITIAL_INDICATOR) {
                    FieldPresence::Missing => None,
                    FieldPresence::Duplicate => return false,
                    FieldPresence::Unique(value) => match value.with_value(|value| match value {
                        ValueView::Boolean(value) => Some(*value),
                        _ => None,
                    }) {
                        Some(value) => Some(value),
                        None => return false,
                    },
                };
                let final_indicator = match field_presence(&payload, FINAL_INDICATOR) {
                    FieldPresence::Missing => None,
                    FieldPresence::Duplicate => return false,
                    FieldPresence::Unique(value) => match value.with_value(|value| match value {
                        ValueView::Boolean(value) => Some(*value),
                        _ => None,
                    }) {
                        Some(value) => Some(value),
                        None => return false,
                    },
                };

                if initial_indicator == Some(true) {
                    return correlation_value.is_none();
                }

                let is_continuation = correlation_value.is_some()
                    || initial_indicator == Some(false)
                    || final_indicator.is_some();
                !is_continuation || correlation_value == Some(true)
            })
            .unwrap_or(false)
    })
}

fn client_encrypt_aad_is_on_initial_request(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_multipart_field_is_initial_only(
        request,
        ENCRYPT_OPERATION,
        AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA,
    )
}

fn client_decrypt_aad_is_on_initial_request(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_multipart_field_is_initial_only(
        request,
        DECRYPT_OPERATION,
        AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA,
    )
}

fn client_decrypt_tag_is_on_initial_request(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_multipart_field_is_initial_only(request, DECRYPT_OPERATION, AUTHENTICATED_ENCRYPTION_TAG)
}

fn client_multipart_field_is_initial_only(
    request: Option<&RequestMessage>,
    operation: u32,
    field_tag: u32,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(batch_operation) = batch_item.operation() else {
            return false;
        };
        if batch_operation != operation {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                match field_presence(&payload, CORRELATION_VALUE) {
                    FieldPresence::Unique(_) => {
                        return matches!(
                            field_presence(&payload, field_tag),
                            FieldPresence::Missing
                        );
                    }
                    FieldPresence::Duplicate => return false,
                    FieldPresence::Missing => {}
                }

                let initial_indicator = match field_presence(&payload, INITIAL_INDICATOR) {
                    FieldPresence::Missing => return true,
                    FieldPresence::Duplicate => return false,
                    FieldPresence::Unique(indicator) => indicator.with_value(|value| match value {
                        ValueView::Boolean(value) => Some(*value),
                        _ => None,
                    }),
                };
                let Some(is_initial) = initial_indicator else {
                    return false;
                };

                match field_presence(&payload, field_tag) {
                    FieldPresence::Missing => true,
                    FieldPresence::Duplicate => false,
                    FieldPresence::Unique(_) => is_initial,
                }
            })
            .unwrap_or(false)
    })
}

fn client_key_block_length_requires_algorithm(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_key_block_direct_field_requires_companion(
        request,
        CRYPTOGRAPHIC_LENGTH,
        CRYPTOGRAPHIC_ALGORITHM,
    )
}

fn client_key_block_algorithm_may_be_omitted_if_value_present(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_key_block_value_is_direct_or_in_plaintext_key_value(request, CRYPTOGRAPHIC_ALGORITHM)
}

fn client_key_block_length_may_be_omitted_if_value_present(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_key_block_value_is_direct_or_in_plaintext_key_value(request, CRYPTOGRAPHIC_LENGTH)
}

fn client_key_block_value_is_direct_or_in_plaintext_key_value(
    request: Option<&RequestMessage>,
    attribute_tag: u32,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if !matches!(operation, REGISTER_OPERATION | IMPORT_OPERATION) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let FieldPresence::Unique(object_type) =
                    field_presence(&payload, REGISTER_OBJECT_TYPE)
                else {
                    return false;
                };
                let Some(object_type) = object_type.with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                }) else {
                    return false;
                };
                if matches!(object_type, SECRET_DATA_OBJECT_TYPE | OPAQUE_OBJECT_TYPE) {
                    return true;
                }
                let Some(object_tag) = key_block_object_tag(object_type) else {
                    // Preserve future Object Type values without applying a known-object rule.
                    return true;
                };
                let FieldPresence::Unique(object) = field_presence(&payload, object_tag) else {
                    return false;
                };
                object.with_value(|value| match value {
                    ValueView::Structure(object) => match field_presence(&object, KEY_BLOCK) {
                        FieldPresence::Unique(key_block) => key_block.with_value(|value| {
                            matches!(value, ValueView::Structure(key_block)
                                if register_key_block_has_table293_attribute(&key_block, attribute_tag))
                        }),
                        FieldPresence::Missing | FieldPresence::Duplicate => false,
                    },
                    _ => false,
                })
            })
            .unwrap_or(false)
    })
}

fn client_key_block_direct_field_requires_companion(
    request: Option<&RequestMessage>,
    field_tag: u32,
    companion_tag: u32,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if !matches!(operation, REGISTER_OPERATION | IMPORT_OPERATION) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|object| {
                        matches!(
                            object.tag().raw(),
                            SYMMETRIC_KEY | PUBLIC_KEY | PRIVATE_KEY | SPLIT_KEY | PGP_KEY
                        )
                    })
                    .all(|object| {
                        object.with_value(|value| match value {
                            ValueView::Structure(object) => {
                                match field_presence(&object, KEY_BLOCK) {
                                    FieldPresence::Missing => true,
                                    FieldPresence::Duplicate => false,
                                    FieldPresence::Unique(key_block) => {
                                        key_block.with_value(|value| match value {
                                            ValueView::Structure(key_block) => {
                                                match field_presence(&key_block, field_tag) {
                                                    FieldPresence::Missing => true,
                                                    FieldPresence::Duplicate => false,
                                                    FieldPresence::Unique(_) => matches!(
                                                        field_presence(&key_block, companion_tag),
                                                        FieldPresence::Unique(_)
                                                    ),
                                                }
                                            }
                                            _ => false,
                                        })
                                    }
                                }
                            }
                            _ => false,
                        })
                    })
            })
            .unwrap_or(false)
    })
}

fn client_register_algorithm_requires_length(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_register_table293_outer_attribute_pair(
        request,
        CRYPTOGRAPHIC_ALGORITHM,
        CRYPTOGRAPHIC_LENGTH,
    )
}

fn client_register_length_requires_algorithm(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    client_register_table293_outer_attribute_pair(
        request,
        CRYPTOGRAPHIC_LENGTH,
        CRYPTOGRAPHIC_ALGORITHM,
    )
}

fn client_register_nist_key_type_is_enumeration(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        if batch_item.operation() != Some(REGISTER_OPERATION) {
            return true;
        }

        batch_item
            .with_request_payload(|payload| register_nist_key_type_is_enumeration(&payload))
            .unwrap_or(false)
    })
}

fn register_nist_key_type_is_enumeration(payload: &StructureView<'_>) -> bool {
    match field_presence(payload, REGISTER_ATTRIBUTES) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(attributes) => attributes.with_value(|value| match value {
            ValueView::Structure(attributes) => {
                register_nist_key_type_attribute_is_well_formed(payload, &attributes)
            }
            _ => false,
        }),
    }
}

fn register_nist_key_type_attribute_is_well_formed(
    payload: &StructureView<'_>,
    attributes: &StructureView<'_>,
) -> bool {
    match field_presence(attributes, NIST_KEY_TYPE_ATTRIBUTE) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(attribute) => {
            if !attribute.with_value(|value| matches!(value, ValueView::Enumeration(_))) {
                return false;
            }

            let FieldPresence::Unique(object_type) = field_presence(payload, REGISTER_OBJECT_TYPE)
            else {
                return false;
            };
            let Some(object_type) = object_type.with_value(|value| match value {
                ValueView::Enumeration(value) => Some(*value),
                _ => None,
            }) else {
                return false;
            };

            register_object_type_allows_nist_key_type(object_type)
        }
    }
}

fn register_object_type_allows_nist_key_type(object_type: u32) -> bool {
    !matches!(
        object_type,
        CERTIFICATE_OBJECT_TYPE
            | RESERVED_OBJECT_TYPE_6
            | OPAQUE_OBJECT_TYPE
            | CERTIFICATE_REQUEST_OBJECT_TYPE
    )
}

fn client_register_table293_pair_may_be_in_key_block(
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
        if operation != REGISTER_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                register_table293_pair_omission_is_backed_by_key_block(&payload)
            })
            .unwrap_or(false)
    })
}

fn register_table293_pair_omission_is_backed_by_key_block(payload: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(object_type) = field_presence(payload, REGISTER_OBJECT_TYPE) else {
        return false;
    };
    let Some(object_type) = object_type.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    }) else {
        return false;
    };
    if object_type == SECRET_DATA_OBJECT_TYPE || !register_table293_object_type_applies(object_type)
    {
        return true;
    }

    let FieldPresence::Unique(attributes) = field_presence(payload, REGISTER_ATTRIBUTES) else {
        return false;
    };
    attributes.with_value(|value| match value {
        ValueView::Structure(attributes) => {
            match (
                field_presence(&attributes, CRYPTOGRAPHIC_ALGORITHM),
                field_presence(&attributes, CRYPTOGRAPHIC_LENGTH),
            ) {
                (FieldPresence::Unique(algorithm), FieldPresence::Unique(length)) => {
                    register_table293_attribute_has_type(algorithm, CRYPTOGRAPHIC_ALGORITHM)
                        && register_table293_attribute_has_type(length, CRYPTOGRAPHIC_LENGTH)
                }
                (FieldPresence::Missing, FieldPresence::Missing) => {
                    register_payload_key_block_has_attribute(payload, CRYPTOGRAPHIC_ALGORITHM)
                        && register_payload_key_block_has_attribute(payload, CRYPTOGRAPHIC_LENGTH)
                }
                _ => false,
            }
        }
        _ => false,
    })
}

fn register_payload_key_block_has_attribute(payload: &StructureView<'_>, tag: u32) -> bool {
    let mut object_fields = payload.children().iter().filter(|field| {
        !matches!(
            field.tag().raw(),
            REGISTER_OBJECT_TYPE | REGISTER_ATTRIBUTES | REGISTER_PROTECTION_STORAGE_MASKS
        )
    });
    let Some(object) = object_fields.next() else {
        return false;
    };
    if object_fields.next().is_some() {
        return false;
    }

    object.with_value(|value| match value {
        ValueView::Structure(object) => match field_presence(&object, KEY_BLOCK) {
            FieldPresence::Unique(key_block) => key_block.with_value(|value| match value {
                ValueView::Structure(key_block) => {
                    register_key_block_has_table293_attribute(&key_block, tag)
                }
                _ => false,
            }),
            FieldPresence::Missing | FieldPresence::Duplicate => false,
        },
        _ => false,
    })
}

fn register_key_block_has_table293_attribute(key_block: &StructureView<'_>, tag: u32) -> bool {
    match field_presence(key_block, tag) {
        FieldPresence::Unique(attribute) => {
            return register_table293_attribute_has_type(attribute, tag);
        }
        FieldPresence::Duplicate => return false,
        FieldPresence::Missing => {}
    }

    let FieldPresence::Unique(key_value) = field_presence(key_block, KEY_VALUE) else {
        return false;
    };
    key_value.with_value(|value| match value {
        ValueView::Structure(key_value) => {
            let FieldPresence::Unique(attributes) = field_presence(&key_value, ATTRIBUTES) else {
                return false;
            };
            attributes.with_value(|value| match value {
                ValueView::Structure(attributes) => match field_presence(&attributes, tag) {
                    FieldPresence::Unique(attribute) => {
                        register_table293_attribute_has_type(attribute, tag)
                    }
                    FieldPresence::Missing | FieldPresence::Duplicate => false,
                },
                _ => false,
            })
        }
        _ => false,
    })
}

fn client_register_signature_algorithm_may_be_in_certificate(
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
        if operation != REGISTER_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                register_certificate_signature_algorithm_is_covered(&payload)
            })
            .unwrap_or(false)
    })
}

fn register_certificate_signature_algorithm_is_covered(payload: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(object_type) = field_presence(payload, REGISTER_OBJECT_TYPE) else {
        return false;
    };
    let Some(object_type) = object_type.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    }) else {
        return false;
    };
    if object_type != CERTIFICATE_OBJECT_TYPE {
        return true;
    }

    let FieldPresence::Unique(attributes) = field_presence(payload, REGISTER_ATTRIBUTES) else {
        return false;
    };
    attributes.with_value(|value| match value {
        ValueView::Structure(attributes) => {
            match field_presence(&attributes, DIGITAL_SIGNATURE_ALGORITHM) {
                FieldPresence::Unique(algorithm) => {
                    algorithm.with_value(|value| matches!(value, ValueView::Enumeration(_)))
                }
                FieldPresence::Duplicate => false,
                FieldPresence::Missing => register_certificate_value_is_present(payload),
            }
        }
        _ => false,
    })
}

fn register_certificate_value_is_present(payload: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(certificate) = field_presence(payload, CERTIFICATE) else {
        return false;
    };
    certificate.with_value(|value| match value {
        ValueView::Structure(certificate) => {
            match field_presence(&certificate, CERTIFICATE_VALUE) {
                FieldPresence::Unique(value) => {
                    value.with_value(|value| matches!(value, ValueView::ByteString(_)))
                }
                FieldPresence::Missing | FieldPresence::Duplicate => false,
            }
        }
        _ => false,
    })
}

fn client_register_table293_outer_attribute_pair(
    request: Option<&RequestMessage>,
    trigger_tag: u32,
    companion_tag: u32,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        if operation != REGISTER_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let FieldPresence::Unique(object_type) =
                    field_presence(&payload, REGISTER_OBJECT_TYPE)
                else {
                    return false;
                };
                let Some(object_type) = object_type.with_value(|value| match value {
                    ValueView::Enumeration(value) => Some(*value),
                    _ => None,
                }) else {
                    return false;
                };

                // KMIP v2.1 §2.3 makes Opaque's managed-cryptographic status
                // depend on client context, which is not carried in TTLV.
                // Secret Data is explicitly exempted by §6.1.43, Table 293.
                // Unknown/vendor object types remain forward-compatible and
                // are outside the base specification's typed applicability.
                if !register_table293_object_type_applies(object_type) {
                    return true;
                }

                match field_presence(&payload, REGISTER_ATTRIBUTES) {
                    FieldPresence::Missing => true,
                    FieldPresence::Duplicate => false,
                    FieldPresence::Unique(attributes) => {
                        attributes.with_value(|value| match value {
                            ValueView::Structure(attributes) => {
                                match field_presence(&attributes, trigger_tag) {
                                    FieldPresence::Missing => true,
                                    FieldPresence::Duplicate => false,
                                    FieldPresence::Unique(trigger) => {
                                        if !register_table293_attribute_has_type(
                                            trigger,
                                            trigger_tag,
                                        ) {
                                            return false;
                                        }

                                        match field_presence(&attributes, companion_tag) {
                                            FieldPresence::Unique(companion) => {
                                                register_table293_attribute_has_type(
                                                    companion,
                                                    companion_tag,
                                                )
                                            }
                                            FieldPresence::Duplicate | FieldPresence::Missing => {
                                                false
                                            }
                                        }
                                    }
                                }
                            }
                            _ => false,
                        })
                    }
                }
            })
            .unwrap_or(false)
    })
}

fn register_table293_object_type_applies(object_type: u32) -> bool {
    matches!(
        object_type,
        CERTIFICATE_OBJECT_TYPE
            | SYMMETRIC_KEY_OBJECT_TYPE
            | PUBLIC_KEY_OBJECT_TYPE
            | PRIVATE_KEY_OBJECT_TYPE
            | SPLIT_KEY_OBJECT_TYPE
            | PGP_KEY_OBJECT_TYPE
            | CERTIFICATE_REQUEST_OBJECT_TYPE
    )
}

fn register_table293_attribute_has_type(attribute: &Item, tag: u32) -> bool {
    attribute.with_value(|value| match tag {
        CRYPTOGRAPHIC_ALGORITHM => matches!(value, ValueView::Enumeration(_)),
        CRYPTOGRAPHIC_LENGTH => matches!(value, ValueView::Integer(_)),
        _ => false,
    })
}

fn pgp_key_preserves_required_fields(pgp_key: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(version) = field_presence(pgp_key, PGP_KEY_VERSION) else {
        return false;
    };
    if !version.with_value(|value| matches!(value, ValueView::Integer(_))) {
        return false;
    }

    let FieldPresence::Unique(key_block) = field_presence(pgp_key, KEY_BLOCK) else {
        return false;
    };
    key_block.with_value(|value| match value {
        ValueView::Structure(key_block) => pgp_key_block_preserves_opaque_value(&key_block),
        _ => false,
    })
}

fn pgp_key_block_preserves_opaque_value(key_block: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(key_format) = field_presence(key_block, KEY_FORMAT_TYPE) else {
        return false;
    };
    if !key_format.with_value(|value| matches!(value, ValueView::Enumeration(_))) {
        return false;
    }

    let FieldPresence::Unique(key_value) = field_presence(key_block, KEY_VALUE) else {
        return false;
    };
    key_value.with_value(|value| matches!(value, ValueView::ByteString(_)))
}

fn client_register_original_creation_date_is_datetime_when_used(
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
        if operation != REGISTER_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(
                |payload| match field_presence(&payload, REGISTER_ATTRIBUTES) {
                    FieldPresence::Missing => true,
                    FieldPresence::Duplicate => false,
                    FieldPresence::Unique(attributes) => {
                        attributes.with_value(|value| match value {
                            ValueView::Structure(attributes) => {
                                optional_date_time_attribute_is_well_formed(
                                    &attributes,
                                    ORIGINAL_CREATION_DATE,
                                )
                            }
                            _ => false,
                        })
                    }
                },
            )
            .unwrap_or(false)
    })
}

fn client_add_original_creation_date_is_datetime_when_used(
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
        if operation != ADD_ATTRIBUTE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| match field_presence(&payload, NEW_ATTRIBUTE) {
                FieldPresence::Missing => true,
                FieldPresence::Duplicate => false,
                FieldPresence::Unique(new_attribute) => {
                    new_attribute.with_value(|value| match value {
                        ValueView::Structure(new_attribute) => {
                            optional_date_time_attribute_is_well_formed(
                                &new_attribute,
                                ORIGINAL_CREATION_DATE,
                            )
                        }
                        _ => false,
                    })
                }
            })
            .unwrap_or(false)
    })
}

fn optional_date_time_attribute_is_well_formed(attributes: &StructureView<'_>, tag: u32) -> bool {
    match field_presence(attributes, tag) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(date) => {
            date.with_value(|value| matches!(value, ValueView::DateTime(_)))
        }
    }
}

fn client_register_random_number_generator_is_well_formed(
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
        if operation != REGISTER_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(
                |payload| match field_presence(&payload, REGISTER_ATTRIBUTES) {
                    FieldPresence::Missing => true,
                    FieldPresence::Duplicate => false,
                    FieldPresence::Unique(attributes) => {
                        attributes.with_value(|value| match value {
                            ValueView::Structure(attributes) => {
                                rng_parameters_attribute_is_well_formed(&attributes)
                            }
                            _ => false,
                        })
                    }
                },
            )
            .unwrap_or(false)
    })
}

fn client_add_attribute_random_number_generator_is_well_formed(
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
        if operation != ADD_ATTRIBUTE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| match field_presence(&payload, NEW_ATTRIBUTE) {
                FieldPresence::Missing => true,
                FieldPresence::Duplicate => false,
                FieldPresence::Unique(new_attribute) => {
                    new_attribute.with_value(|value| match value {
                        ValueView::Structure(new_attribute) => {
                            rng_parameters_attribute_is_well_formed(&new_attribute)
                        }
                        _ => false,
                    })
                }
            })
            .unwrap_or(false)
    })
}

fn client_rng_parameters_are_well_formed(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    const RNG_ATTRIBUTE_CREATION_OPERATIONS: [u32; 5] = [
        CREATE_OPERATION,
        CREATE_KEY_PAIR_OPERATION,
        DERIVE_KEY_OPERATION,
        REKEY_OPERATION,
        REKEY_KEY_PAIR_OPERATION,
    ];

    if let Some((request, response)) = response {
        return client_get_attributes_rng_response_is_well_formed(request, response);
    }

    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        let attribute_groups: &[u32] = if operation == REGISTER_OPERATION {
            &[REGISTER_ATTRIBUTES]
        } else if operation == ADD_ATTRIBUTE_OPERATION {
            &[NEW_ATTRIBUTE]
        } else if operation == CREATE_KEY_PAIR_OPERATION || operation == REKEY_KEY_PAIR_OPERATION {
            &CREATE_KEY_PAIR_ATTRIBUTE_GROUPS
        } else if RNG_ATTRIBUTE_CREATION_OPERATIONS.contains(&operation) {
            &[ATTRIBUTES]
        } else {
            return true;
        };

        batch_item
            .with_request_payload(|payload| {
                attribute_groups
                    .iter()
                    .all(|group| rng_attribute_group_is_well_formed(&payload, *group))
            })
            .unwrap_or(false)
    })
}

fn client_get_attributes_rng_response_is_well_formed(
    request: &RequestMessage,
    response: &ResponseMessage,
) -> bool {
    let request_items = request.batch_items().collect::<Vec<_>>();
    let response_items = response.batch_items().collect::<Vec<_>>();
    if request_items.len() != response_items.len() {
        return false;
    }

    let mut response_indices_by_id = HashMap::new();
    for (index, response_item) in response_items.iter().copied().enumerate() {
        if let Some(response_id) = response_item.with_unique_batch_item_id(<[u8]>::to_vec)
            && response_indices_by_id.insert(response_id, index).is_some()
        {
            return false;
        }
    }

    let mut consumed_responses = vec![false; response_items.len()];
    for (request_index, request_item) in request_items.iter().copied().enumerate() {
        if request_item.operation() != Some(GET_ATTRIBUTES_OPERATION) {
            continue;
        }

        let request_id = request_item.with_unique_batch_item_id(<[u8]>::to_vec);
        let response_index = if let Some(request_id) = request_id {
            let Some(response_index) = response_indices_by_id.get(&request_id).copied() else {
                return false;
            };
            response_index
        } else if request_items.len() == 1 && response_items.len() == 1 {
            if response_items[0]
                .with_unique_batch_item_id(|_| ())
                .is_some()
            {
                return false;
            }
            0
        } else if request.header().effective_batch_order_option() {
            if response_items[request_index]
                .with_unique_batch_item_id(|_| ())
                .is_some()
            {
                return false;
            }
            request_index
        } else {
            return false;
        };

        let Some(consumed_response) = consumed_responses.get_mut(response_index) else {
            return false;
        };
        if *consumed_response {
            return false;
        }
        *consumed_response = true;

        let response_item = response_items[response_index];
        if response_item.operation() != Some(GET_ATTRIBUTES_OPERATION) {
            return false;
        }
        let Some(result_status) = response_item.result_status() else {
            return false;
        };
        if result_status.raw() != RESULT_STATUS_SUCCESS {
            continue;
        }

        if !response_item
            .with_response_payload(|payload| {
                rng_attribute_group_is_well_formed(&payload, ATTRIBUTES)
            })
            .unwrap_or(false)
        {
            return false;
        }
    }

    true
}

fn rng_attribute_group_is_well_formed(payload: &StructureView<'_>, group_tag: u32) -> bool {
    match field_presence(payload, group_tag) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(attributes) => attributes.with_value(|value| match value {
            ValueView::Structure(attributes) => {
                rng_parameters_attribute_is_well_formed(&attributes)
            }
            _ => false,
        }),
    }
}

fn client_application_specific_information_can_omit_data(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    const ATTRIBUTE_CREATION_OPERATIONS: [u32; 11] = [
        CREATE_OPERATION,
        CREATE_KEY_PAIR_OPERATION,
        REGISTER_OPERATION,
        REKEY_OPERATION,
        DERIVE_KEY_OPERATION,
        CERTIFY_OPERATION,
        RECERTIFY_OPERATION,
        REKEY_KEY_PAIR_OPERATION,
        CREATE_SPLIT_KEY_OPERATION,
        JOIN_SPLIT_KEY_OPERATION,
        IMPORT_OPERATION,
    ];

    let Some(request) = request else {
        return false;
    };

    request.batch_items().all(|batch_item| {
        let Some(operation) = batch_item.operation() else {
            return false;
        };
        let attribute_groups: &[u32] = if ATTRIBUTE_CREATION_OPERATIONS.contains(&operation) {
            if matches!(
                operation,
                CREATE_KEY_PAIR_OPERATION | REKEY_KEY_PAIR_OPERATION
            ) {
                &CREATE_KEY_PAIR_ATTRIBUTE_GROUPS
            } else {
                &[ATTRIBUTES]
            }
        } else if matches!(
            operation,
            ADD_ATTRIBUTE_OPERATION | MODIFY_ATTRIBUTE_OPERATION | SET_ATTRIBUTE_OPERATION
        ) {
            &[NEW_ATTRIBUTE]
        } else {
            return true;
        };

        batch_item
            .with_request_payload(|payload| {
                attribute_groups.iter().all(|group| {
                    application_attribute_group_is_well_formed(
                        &payload,
                        *group,
                        *group != NEW_ATTRIBUTE,
                    )
                })
            })
            .unwrap_or(false)
    })
}

fn application_attribute_group_is_well_formed(
    payload: &StructureView<'_>,
    group_tag: u32,
    allow_multiple_instances: bool,
) -> bool {
    match field_presence(payload, group_tag) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(group) => group.with_value(|value| match value {
            ValueView::Structure(attributes) => application_specific_information_is_well_formed(
                &attributes,
                allow_multiple_instances,
            ),
            _ => false,
        }),
    }
}

fn application_specific_information_is_well_formed(
    attributes: &StructureView<'_>,
    allow_multiple_instances: bool,
) -> bool {
    let contains_application_specific_information = attributes
        .children()
        .iter()
        .any(|field| field.tag().raw() == APPLICATION_SPECIFIC_INFORMATION);
    if !allow_multiple_instances
        && contains_application_specific_information
        && attributes.children().len() != 1
    {
        return false;
    }

    for (instance_index, application_information) in attributes
        .children()
        .iter()
        .filter(|field| field.tag().raw() == APPLICATION_SPECIFIC_INFORMATION)
        .enumerate()
    {
        if !allow_multiple_instances && instance_index > 0 {
            return false;
        }

        let is_well_formed = application_information.with_value(|value| match value {
            ValueView::Structure(fields) => application_namespace_is_well_formed(&fields),
            _ => false,
        });
        if !is_well_formed {
            return false;
        }
    }
    true
}

fn application_namespace_is_well_formed(fields: &StructureView<'_>) -> bool {
    let namespace_is_valid = matches!(
        field_presence(fields, APPLICATION_NAMESPACE),
        FieldPresence::Unique(namespace)
            if namespace.with_value(|value| matches!(value, ValueView::TextString(_)))
    );
    let data_is_valid = match field_presence(fields, APPLICATION_DATA) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(data) => {
            data.with_value(|value| matches!(value, ValueView::TextString(_)))
        }
    };
    namespace_is_valid && data_is_valid
}

fn client_server_generated_key_format_type_is_enumeration(
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
        let attribute_groups: &[u32] = match operation {
            CREATE_OPERATION
            | CREATE_SPLIT_KEY_OPERATION
            | REKEY_OPERATION
            | DERIVE_KEY_OPERATION => &[ATTRIBUTES],
            CREATE_KEY_PAIR_OPERATION | REKEY_KEY_PAIR_OPERATION => {
                &CREATE_KEY_PAIR_ATTRIBUTE_GROUPS
            }
            _ => return true,
        };

        batch_item
            .with_request_payload(|payload| {
                attribute_groups
                    .iter()
                    .all(|group| attribute_group_key_format_type_is_enumeration(&payload, *group))
            })
            .unwrap_or(false)
    })
}

fn attribute_group_key_format_type_is_enumeration(
    payload: &StructureView<'_>,
    group_tag: u32,
) -> bool {
    match field_presence(payload, group_tag) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(group) => group.with_value(|value| match value {
            ValueView::Structure(attributes) => {
                key_format_type_attribute_is_enumeration(&attributes)
            }
            _ => false,
        }),
    }
}

fn key_format_type_attribute_is_enumeration(attributes: &StructureView<'_>) -> bool {
    match field_presence(attributes, KEY_FORMAT_TYPE) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(key_format_type) => {
            key_format_type.with_value(|value| matches!(value, ValueView::Enumeration(_)))
        }
    }
}

fn rng_parameters_attribute_is_well_formed(attributes: &StructureView<'_>) -> bool {
    match field_presence(attributes, RANDOM_NUMBER_GENERATOR) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(rng) => rng.with_value(|value| match value {
            ValueView::Structure(parameters) => {
                rng_parameters_structure_is_well_formed(&parameters)
            }
            _ => false,
        }),
    }
}

fn rng_parameters_structure_is_well_formed(parameters: &StructureView<'_>) -> bool {
    if !matches!(
        field_presence(parameters, RNG_PARAMETERS),
        FieldPresence::Missing
    ) {
        return false;
    }

    let algorithm_is_well_formed = match field_presence(parameters, RNG_ALGORITHM) {
        FieldPresence::Unique(algorithm) => {
            algorithm.with_value(|value| matches!(value, ValueView::Enumeration(_)))
        }
        FieldPresence::Missing | FieldPresence::Duplicate => false,
    };

    algorithm_is_well_formed
        && [
            CRYPTOGRAPHIC_ALGORITHM,
            CRYPTOGRAPHIC_LENGTH,
            RNG_HASHING_ALGORITHM,
            RNG_DRBG_ALGORITHM,
            RNG_RECOMMENDED_CURVE,
            RNG_FIPS186_VARIATION,
            RNG_PREDICTION_RESISTANCE,
        ]
        .into_iter()
        .all(|tag| rng_optional_parameter_is_well_formed(parameters, tag))
}

fn rng_optional_parameter_is_well_formed(parameters: &StructureView<'_>, tag: u32) -> bool {
    match field_presence(parameters, tag) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(field) => field.with_value(|value| match tag {
            CRYPTOGRAPHIC_LENGTH => matches!(value, ValueView::Integer(_)),
            RNG_PREDICTION_RESISTANCE => matches!(value, ValueView::Boolean(_)),
            CRYPTOGRAPHIC_ALGORITHM
            | RNG_HASHING_ALGORITHM
            | RNG_DRBG_ALGORITHM
            | RNG_RECOMMENDED_CURVE
            | RNG_FIPS186_VARIATION => matches!(value, ValueView::Enumeration(_)),
            _ => false,
        }),
    }
}

fn client_register_key_value_location_requires_omitted_key_value(
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
        if operation != REGISTER_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| register_key_value_location_is_valid(&payload))
            .unwrap_or(false)
    })
}

fn client_register_does_not_specify_key_value_present(
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
        if operation != REGISTER_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(
                |payload| match field_presence(&payload, REGISTER_ATTRIBUTES) {
                    FieldPresence::Missing => true,
                    FieldPresence::Unique(attributes) => {
                        attributes.with_value(|value| match value {
                            ValueView::Structure(attributes) => attributes
                                .children()
                                .iter()
                                .all(|attribute| attribute.tag().raw() != KEY_VALUE_PRESENT),
                            _ => false,
                        })
                    }
                    FieldPresence::Duplicate => false,
                },
            )
            .unwrap_or(false)
    })
}

fn client_key_value_present_is_not_modified(
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
        let Some(mutation) = (match operation {
            ADD_ATTRIBUTE_OPERATION => Some(ClientAttributeMutation::Add),
            ADJUST_ATTRIBUTE_OPERATION => Some(ClientAttributeMutation::Adjust),
            DELETE_ATTRIBUTE_OPERATION => Some(ClientAttributeMutation::Delete),
            MODIFY_ATTRIBUTE_OPERATION => Some(ClientAttributeMutation::Modify),
            SET_ATTRIBUTE_OPERATION => Some(ClientAttributeMutation::Set),
            _ => None,
        }) else {
            return true;
        };

        batch_item
            .with_request_payload(|payload| match operation {
                ADD_ATTRIBUTE_OPERATION | SET_ATTRIBUTE_OPERATION => {
                    attribute_fields_do_not_mutate_key_value_present(
                        &payload,
                        NEW_ATTRIBUTE,
                        mutation,
                    )
                }
                ADJUST_ATTRIBUTE_OPERATION => {
                    attribute_references_do_not_mutate_key_value_present(&payload, mutation)
                }
                DELETE_ATTRIBUTE_OPERATION => {
                    attribute_fields_do_not_mutate_key_value_present(
                        &payload,
                        CURRENT_ATTRIBUTE,
                        mutation,
                    ) && attribute_references_do_not_mutate_key_value_present(&payload, mutation)
                }
                MODIFY_ATTRIBUTE_OPERATION => {
                    attribute_fields_do_not_mutate_key_value_present(
                        &payload,
                        CURRENT_ATTRIBUTE,
                        mutation,
                    ) && attribute_fields_do_not_mutate_key_value_present(
                        &payload,
                        NEW_ATTRIBUTE,
                        mutation,
                    )
                }
                _ => true,
            })
            .unwrap_or(false)
    })
}

fn client_set_attribute_does_not_mutate_read_only_attributes(
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
        if operation != SET_ATTRIBUTE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == NEW_ATTRIBUTE)
                    .all(|new_attribute| {
                        new_attribute.with_value(|value| match value {
                            ValueView::Structure(attributes) => {
                                attributes.children().iter().all(|attribute| {
                                    !client_attribute_mutation_is_prohibited(
                                        attribute.tag().raw(),
                                        ClientAttributeMutation::Set,
                                    )
                                })
                            }
                            _ => false,
                        })
                    })
            })
            .unwrap_or(false)
    })
}

fn client_read_only_attribute_modification_is_not_requested(
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
        let Some(mutation) = (match operation {
            ADD_ATTRIBUTE_OPERATION => Some(ClientAttributeMutation::Add),
            ADJUST_ATTRIBUTE_OPERATION => Some(ClientAttributeMutation::Adjust),
            MODIFY_ATTRIBUTE_OPERATION => Some(ClientAttributeMutation::Modify),
            SET_ATTRIBUTE_OPERATION => Some(ClientAttributeMutation::Set),
            _ => None,
        }) else {
            return true;
        };

        batch_item
            .with_request_payload(|payload| match operation {
                ADD_ATTRIBUTE_OPERATION => {
                    attribute_fields_allow_client_add_with_creation_date_exception(
                        &payload,
                        NEW_ATTRIBUTE,
                        mutation,
                    )
                }
                SET_ATTRIBUTE_OPERATION => {
                    attribute_fields_allow_client_mutation(&payload, NEW_ATTRIBUTE, mutation)
                }
                ADJUST_ATTRIBUTE_OPERATION => {
                    attribute_references_allow_client_mutation(&payload, mutation)
                }
                MODIFY_ATTRIBUTE_OPERATION => {
                    attribute_fields_allow_client_mutation(&payload, CURRENT_ATTRIBUTE, mutation)
                        && attribute_fields_allow_client_mutation(&payload, NEW_ATTRIBUTE, mutation)
                }
                _ => true,
            })
            .unwrap_or(false)
    })
}

fn client_state_modify_attribute_is_not_requested(
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
        if operation != MODIFY_ATTRIBUTE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                [CURRENT_ATTRIBUTE, NEW_ATTRIBUTE]
                    .into_iter()
                    .all(|wrapper_tag| {
                        payload
                            .children()
                            .iter()
                            .filter(|field| field.tag().raw() == wrapper_tag)
                            .all(|field| {
                                field.with_value(|value| match value {
                                    ValueView::Structure(attributes) => attributes
                                        .children()
                                        .iter()
                                        .all(|attribute| attribute.tag().raw() != STATE_ATTRIBUTE),
                                    _ => false,
                                })
                            })
                    })
            })
            .unwrap_or(false)
    })
}

fn client_read_only_attribute_deletion_is_not_requested(
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
        if operation != DELETE_ATTRIBUTE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                attribute_fields_allow_client_mutation(
                    &payload,
                    CURRENT_ATTRIBUTE,
                    ClientAttributeMutation::Delete,
                ) && attribute_references_allow_client_mutation(
                    &payload,
                    ClientAttributeMutation::Delete,
                )
            })
            .unwrap_or(false)
    })
}

fn attribute_fields_allow_client_mutation(
    payload: &StructureView<'_>,
    wrapper_tag: u32,
    mutation: ClientAttributeMutation,
) -> bool {
    payload
        .children()
        .iter()
        .filter(|field| field.tag().raw() == wrapper_tag)
        .all(|field| {
            field.with_value(|value| match value {
                ValueView::Structure(attributes) => attributes.children().iter().all(|attribute| {
                    !client_attribute_mutation_is_prohibited(attribute.tag().raw(), mutation)
                }),
                _ => false,
            })
        })
}

fn attribute_fields_allow_client_add_with_creation_date_exception(
    payload: &StructureView<'_>,
    wrapper_tag: u32,
    mutation: ClientAttributeMutation,
) -> bool {
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;

    payload
        .children()
        .iter()
        .filter(|field| field.tag().raw() == wrapper_tag)
        .all(|field| {
            field.with_value(|value| match value {
                ValueView::Structure(attributes) => attributes.children().iter().all(|attribute| {
                    attribute.tag().raw() == ORIGINAL_CREATION_DATE
                        || !client_attribute_mutation_is_prohibited(attribute.tag().raw(), mutation)
                }),
                _ => false,
            })
        })
}

fn attribute_references_allow_client_mutation(
    payload: &StructureView<'_>,
    mutation: ClientAttributeMutation,
) -> bool {
    payload
        .children()
        .iter()
        .filter(|field| field.tag().raw() == ATTRIBUTE_REFERENCE)
        .all(|field| {
            let Ok(reference) = AttributeReference::try_from_ttlv_item(field) else {
                return false;
            };
            if let Some(tag) = reference.tag_value() {
                return !client_attribute_mutation_is_prohibited(tag, mutation);
            }
            let Some((vendor_identification, attribute_name)) = reference.name_parts() else {
                return false;
            };
            if !vendor_identification.is_empty() {
                return true;
            }
            crate::attribute_policy::ATTRIBUTE_POLICIES
                .iter()
                .find(|policy| policy.name == attribute_name)
                .is_none_or(|policy| !client_attribute_mutation_is_prohibited(policy.tag, mutation))
        })
}

fn client_usage_limits_count_is_not_set_by_add_or_modify(
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
        let Some(mutation) = (match operation {
            ADD_ATTRIBUTE_OPERATION => Some(ClientAttributeMutation::Add),
            MODIFY_ATTRIBUTE_OPERATION => Some(ClientAttributeMutation::Modify),
            _ => None,
        }) else {
            return true;
        };

        batch_item
            .with_request_payload(|payload| match field_presence(&payload, NEW_ATTRIBUTE) {
                FieldPresence::Missing => true,
                FieldPresence::Duplicate => false,
                FieldPresence::Unique(new_attribute) => {
                    new_attribute.with_value(|value| match value {
                        ValueView::Structure(attributes) => {
                            attributes.children().iter().all(|attribute| {
                                attribute.tag().raw() != USAGE_LIMITS_ATTRIBUTE
                                    || attribute.with_value(|value| match value {
                                        ValueView::Structure(usage_limits) => {
                                            !usage_limits.children().iter().any(|field| {
                                                field.tag().raw() == USAGE_LIMITS_COUNT
                                            }) || !client_attribute_mutation_is_prohibited(
                                                USAGE_LIMITS_ATTRIBUTE,
                                                mutation,
                                            )
                                        }
                                        _ => true,
                                    })
                            })
                        }
                        _ => false,
                    })
                }
            })
            .unwrap_or(false)
    })
}

fn client_compromise_occurrence_date_only_for_key_or_ca_compromise(
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
        if operation != REVOKE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                if !payload
                    .children()
                    .iter()
                    .any(|field| field.tag().raw() == COMPROMISE_OCCURRENCE_DATE)
                {
                    return true;
                }

                let FieldPresence::Unique(reason) = field_presence(&payload, REVOCATION_REASON)
                else {
                    return false;
                };
                reason.with_value(|value| {
                    let ValueView::Structure(reason) = value else {
                        return false;
                    };
                    let FieldPresence::Unique(reason_code) =
                        field_presence(&reason, REVOCATION_REASON_CODE)
                    else {
                        return false;
                    };
                    reason_code.with_value(|value| {
                        matches!(
                            value,
                            ValueView::Enumeration(value)
                                if *value == KEY_COMPROMISE_REASON
                                    || *value == CA_COMPROMISE_REASON
                        )
                    })
                })
            })
            .unwrap_or(false)
    })
}

fn client_revoke_compromise_occurrence_date_recommended_for_compromise_reasons(
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
        if operation != REVOKE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let is_compromise_reason = match field_presence(&payload, REVOCATION_REASON) {
                    FieldPresence::Unique(reason) => reason.with_value(|value| {
                        let ValueView::Structure(reason) = value else {
                            return false;
                        };
                        let FieldPresence::Unique(reason_code) =
                            field_presence(&reason, REVOCATION_REASON_CODE)
                        else {
                            return false;
                        };
                        reason_code.with_value(|value| {
                            matches!(
                                value,
                                ValueView::Enumeration(value)
                                    if *value == KEY_COMPROMISE_REASON
                                        || *value == CA_COMPROMISE_REASON
                            )
                        })
                    }),
                    FieldPresence::Missing | FieldPresence::Duplicate => false,
                };
                if !is_compromise_reason {
                    return true;
                }
                let FieldPresence::Unique(date) =
                    field_presence(&payload, COMPROMISE_OCCURRENCE_DATE)
                else {
                    return false;
                };
                date.with_value(|value| matches!(value, ValueView::DateTime(_)))
            })
            .unwrap_or(false)
    })
}

fn client_revoke_revocation_message_is_text_when_used(
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
        if operation != REVOKE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(
                |payload| match field_presence(&payload, REVOCATION_REASON) {
                    FieldPresence::Missing => true,
                    FieldPresence::Duplicate => false,
                    FieldPresence::Unique(reason) => reason.with_value(|value| match value {
                        ValueView::Structure(reason) => {
                            revoke_reason_has_valid_optional_message(&reason)
                        }
                        _ => false,
                    }),
                },
            )
            .unwrap_or(false)
    })
}

fn revoke_reason_has_valid_optional_message(reason: &StructureView<'_>) -> bool {
    match field_presence(reason, REVOCATION_MESSAGE) {
        FieldPresence::Missing => true,
        FieldPresence::Duplicate => false,
        FieldPresence::Unique(message) => {
            message.with_value(|value| matches!(value, ValueView::TextString(_)))
        }
    }
}

fn client_validate_validity_date_is_usable(
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
        if operation != VALIDATE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| match field_presence(&payload, VALIDITY_DATE) {
                FieldPresence::Missing => true,
                FieldPresence::Unique(date) => {
                    date.with_value(|value| matches!(value, ValueView::DateTime(_)))
                }
                FieldPresence::Duplicate => false,
            })
            .unwrap_or(false)
    })
}

fn client_validate_repeated_certificates_are_usable(
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
        if operation != VALIDATE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == VALIDATE_CERTIFICATE)
                    .all(|field| field.with_value(|value| matches!(value, ValueView::Structure(_))))
            })
            .unwrap_or(false)
    })
}

fn client_validate_repeated_unique_identifiers_are_usable(
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
        if operation != VALIDATE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == UNIQUE_IDENTIFIER)
                    .all(|field| {
                        field.with_value(|value| {
                            matches!(
                                value,
                                ValueView::TextString(_)
                                    | ValueView::Enumeration(_)
                                    | ValueView::Integer(_)
                            )
                        })
                    })
            })
            .unwrap_or(false)
    })
}

// Validate has one payload with two flat, repeatable lists. Either list may be
// empty, but the combined lists need one member; other fields do not supply it.
// The representation cannot identify cryptographic chain boundaries or
// validate the chain itself.
fn client_validate_chain_list_members_are_usable(
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
        if operation != VALIDATE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                let mut has_chain_member = false;
                let fields_are_usable =
                    payload
                        .children()
                        .iter()
                        .all(|field| match field.tag().raw() {
                            VALIDATE_CERTIFICATE => field.with_value(|value| {
                                if !matches!(value, ValueView::Structure(_)) {
                                    return false;
                                }
                                has_chain_member = true;
                                true
                            }),
                            UNIQUE_IDENTIFIER => field.with_value(|value| {
                                if !matches!(
                                    value,
                                    ValueView::TextString(_)
                                        | ValueView::Enumeration(_)
                                        | ValueView::Integer(_)
                                ) {
                                    return false;
                                }
                                has_chain_member = true;
                                true
                            }),
                            _ => true,
                        });
                fields_are_usable && has_chain_member
            })
            .unwrap_or(false)
    })
}

fn attribute_fields_do_not_mutate_key_value_present(
    payload: &StructureView<'_>,
    wrapper_tag: u32,
    mutation: ClientAttributeMutation,
) -> bool {
    payload
        .children()
        .iter()
        .filter(|field| field.tag().raw() == wrapper_tag)
        .all(|field| {
            field.with_value(|value| match value {
                ValueView::Structure(attributes) => attributes.children().iter().all(|attribute| {
                    attribute.tag().raw() != KEY_VALUE_PRESENT
                        || !client_attribute_mutation_is_prohibited(KEY_VALUE_PRESENT, mutation)
                }),
                _ => false,
            })
        })
}

fn attribute_references_do_not_mutate_key_value_present(
    payload: &StructureView<'_>,
    mutation: ClientAttributeMutation,
) -> bool {
    payload
        .children()
        .iter()
        .filter(|field| field.tag().raw() == ATTRIBUTE_REFERENCE)
        .all(|field| {
            AttributeReference::try_from_ttlv_item(field).is_ok_and(|reference| {
                if let Some(tag) = reference.tag_value() {
                    tag != KEY_VALUE_PRESENT
                        || !client_attribute_mutation_is_prohibited(KEY_VALUE_PRESENT, mutation)
                } else if let Some((vendor_identification, attribute_name)) = reference.name_parts()
                {
                    !(vendor_identification.is_empty()
                        && attribute_name == "Key Value Present"
                        && client_attribute_mutation_is_prohibited(KEY_VALUE_PRESENT, mutation))
                } else {
                    false
                }
            })
        })
}

fn client_locate_key_value_present_is_boolean_when_used(
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
        if operation != LOCATE_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| match field_presence(&payload, ATTRIBUTES) {
                FieldPresence::Missing => true,
                FieldPresence::Unique(attributes) => attributes.with_value(|value| match value {
                    ValueView::Structure(attributes) => {
                        match field_presence(&attributes, KEY_VALUE_PRESENT) {
                            FieldPresence::Missing => true,
                            FieldPresence::Unique(attribute) => {
                                attribute.with_value(|value| matches!(value, ValueView::Boolean(_)))
                            }
                            FieldPresence::Duplicate => false,
                        }
                    }
                    _ => false,
                }),
                FieldPresence::Duplicate => false,
            })
            .unwrap_or(false)
    })
}

fn client_register_preserves_optional_wrapped_key_value(
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
        if operation != REGISTER_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                register_payload_preserves_optional_key_wrapping(&payload)
            })
            .unwrap_or(false)
    })
}

fn client_register_key_value_attributes_are_structured(
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
        if operation != REGISTER_OPERATION {
            return true;
        }

        batch_item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .filter(|field| {
                        !matches!(
                            field.tag().raw(),
                            REGISTER_OBJECT_TYPE
                                | REGISTER_ATTRIBUTES
                                | REGISTER_PROTECTION_STORAGE_MASKS
                        )
                    })
                    .all(register_object_key_value_attributes_are_structured)
            })
            .unwrap_or(false)
    })
}

fn register_object_key_value_attributes_are_structured(object: &Item) -> bool {
    object.with_value(|value| match value {
        ValueView::Structure(object) => match field_presence(&object, KEY_BLOCK) {
            FieldPresence::Missing => true,
            FieldPresence::Unique(key_block) => key_block.with_value(|value| match value {
                ValueView::Structure(key_block) => {
                    register_key_block_attributes_are_structured(&key_block)
                }
                _ => false,
            }),
            FieldPresence::Duplicate => false,
        },
        _ => false,
    })
}

fn register_key_block_attributes_are_structured(key_block: &StructureView<'_>) -> bool {
    match field_presence(key_block, KEY_VALUE) {
        FieldPresence::Missing => true,
        FieldPresence::Unique(key_value) => key_value.with_value(|value| match value {
            ValueView::ByteString(_) => true,
            ValueView::Structure(key_value) => match field_presence(&key_value, ATTRIBUTES) {
                FieldPresence::Missing => true,
                FieldPresence::Unique(attributes) => {
                    attributes.with_value(|value| matches!(value, ValueView::Structure(_)))
                }
                FieldPresence::Duplicate => false,
            },
            _ => false,
        }),
        FieldPresence::Duplicate => false,
    }
}

fn register_payload_preserves_optional_key_wrapping(payload: &StructureView<'_>) -> bool {
    for object in payload.children().iter().filter(|field| {
        !matches!(
            field.tag().raw(),
            REGISTER_OBJECT_TYPE | REGISTER_ATTRIBUTES | REGISTER_PROTECTION_STORAGE_MASKS
        )
    }) {
        let object_has_valid_key_block = object.with_value(|value| match value {
            ValueView::Structure(object) => match field_presence(&object, KEY_BLOCK) {
                FieldPresence::Missing => true,
                FieldPresence::Unique(key_block) => key_block.with_value(|value| match value {
                    ValueView::Structure(key_block) => {
                        register_key_block_preserves_optional_wrapping(&key_block)
                    }
                    _ => false,
                }),
                FieldPresence::Duplicate => false,
            },
            _ => true,
        });
        if !object_has_valid_key_block {
            return false;
        }
    }
    true
}

fn register_key_block_preserves_optional_wrapping(key_block: &StructureView<'_>) -> bool {
    match field_presence(key_block, KEY_WRAPPING_DATA) {
        FieldPresence::Missing => true,
        FieldPresence::Unique(wrapping_data) => {
            let FieldPresence::Unique(key_value) = field_presence(key_block, KEY_VALUE) else {
                return false;
            };
            if !key_value.with_value(|value| matches!(value, ValueView::ByteString(_))) {
                return false;
            }
            wrapping_data.with_value(|value| match value {
                ValueView::Structure(data) => wrapping_structure_has_key_information(&data),
                _ => false,
            })
        }
        FieldPresence::Duplicate => false,
    }
}

fn register_key_value_location_is_valid(payload: &StructureView<'_>) -> bool {
    let location = match field_presence(payload, REGISTER_ATTRIBUTES) {
        FieldPresence::Missing => return true,
        FieldPresence::Unique(attributes) => attributes.with_value(|value| match value {
            ValueView::Structure(attributes) => {
                match field_presence(&attributes, KEY_VALUE_LOCATION) {
                    FieldPresence::Missing => None,
                    FieldPresence::Unique(location) => {
                        Some(location.with_value(|value| match value {
                            ValueView::Structure(location) => {
                                key_value_location_members_are_valid(&location)
                            }
                            _ => false,
                        }))
                    }
                    FieldPresence::Duplicate => Some(false),
                }
            }
            _ => Some(false),
        }),
        FieldPresence::Duplicate => return false,
    };
    let Some(location_is_valid) = location else {
        return true;
    };
    location_is_valid && register_object_omits_key_value(payload)
}

fn key_value_location_members_are_valid(location: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(value) = field_presence(location, KEY_VALUE_LOCATION_VALUE) else {
        return false;
    };
    let FieldPresence::Unique(location_type) = field_presence(location, KEY_VALUE_LOCATION_TYPE)
    else {
        return false;
    };

    value.with_value(|value| matches!(value, ValueView::TextString(_)))
        && location_type.with_value(|value| matches!(value, ValueView::Enumeration(_)))
}

fn register_object_omits_key_value(payload: &StructureView<'_>) -> bool {
    let mut key_block_count = 0_u8;
    let mut every_key_block_omits_key_value = true;

    for field in payload.children() {
        if matches!(
            field.tag().raw(),
            REGISTER_OBJECT_TYPE | REGISTER_ATTRIBUTES | REGISTER_PROTECTION_STORAGE_MASKS
        ) {
            continue;
        }

        match register_object_key_value_state(field) {
            RegisterObjectKeyValueState::NoKeyBlock => {}
            RegisterObjectKeyValueState::KeyValueOmitted => {
                key_block_count = key_block_count.saturating_add(1);
            }
            RegisterObjectKeyValueState::KeyValuePresent
            | RegisterObjectKeyValueState::Malformed => {
                key_block_count = key_block_count.saturating_add(1);
                every_key_block_omits_key_value = false;
            }
        }
    }

    key_block_count == 1 && every_key_block_omits_key_value
}

fn register_object_key_value_state(field: &Item) -> RegisterObjectKeyValueState {
    field.with_value(|value| match value {
        ValueView::Structure(object) => match field_presence(&object, KEY_BLOCK) {
            FieldPresence::Missing => RegisterObjectKeyValueState::NoKeyBlock,
            FieldPresence::Duplicate => RegisterObjectKeyValueState::Malformed,
            FieldPresence::Unique(key_block) => key_block.with_value(|value| match value {
                ValueView::Structure(key_block) => match field_presence(&key_block, KEY_VALUE) {
                    FieldPresence::Missing => RegisterObjectKeyValueState::KeyValueOmitted,
                    FieldPresence::Unique(_) => RegisterObjectKeyValueState::KeyValuePresent,
                    FieldPresence::Duplicate => RegisterObjectKeyValueState::Malformed,
                },
                _ => RegisterObjectKeyValueState::Malformed,
            }),
        },
        _ => RegisterObjectKeyValueState::NoKeyBlock,
    })
}

#[derive(Clone, Copy)]
enum RegisterObjectKeyValueState {
    NoKeyBlock,
    KeyValueOmitted,
    KeyValuePresent,
    Malformed,
}

#[derive(Clone, Copy)]
enum FieldPresence<'a> {
    Missing,
    Unique(&'a Item),
    Duplicate,
}

fn field_presence<'a>(structure: &'a StructureView<'_>, tag: u32) -> FieldPresence<'a> {
    let mut found = None;
    for field in structure.children() {
        if field.tag().raw() == tag {
            if found.is_some() {
                return FieldPresence::Duplicate;
            }
            found = Some(field);
        }
    }
    found.map_or(FieldPresence::Missing, FieldPresence::Unique)
}

fn import_payload_preserves_secret_data_wrapping(payload: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(object_type) = field_presence(payload, IMPORT_OBJECT_TYPE) else {
        return false;
    };
    let Some(object_type) = object_type.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    }) else {
        return false;
    };
    if object_type != SECRET_DATA_OBJECT_TYPE {
        return true;
    }

    let FieldPresence::Unique(secret_data) = field_presence(payload, SECRET_DATA) else {
        return false;
    };
    secret_data.with_value(|value| match value {
        ValueView::Structure(secret_data) => secret_data_preserves_wrapping(&secret_data),
        _ => false,
    })
}

fn secret_data_preserves_wrapping(secret_data: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(secret_data_type) = field_presence(secret_data, SECRET_DATA_TYPE)
    else {
        return false;
    };
    if !secret_data_type.with_value(|value| matches!(value, ValueView::Enumeration(_))) {
        return false;
    }

    let FieldPresence::Unique(key_block) = field_presence(secret_data, KEY_BLOCK) else {
        return false;
    };
    key_block.with_value(|value| match value {
        ValueView::Structure(key_block) => key_block_preserves_wrapping(&key_block),
        _ => false,
    })
}

fn key_block_preserves_wrapping(key_block: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(key_value) = field_presence(key_block, KEY_VALUE) else {
        return false;
    };
    let key_value_is_byte_string =
        key_value.with_value(|value| matches!(value, ValueView::ByteString(_)));
    match field_presence(key_block, KEY_WRAPPING_DATA) {
        FieldPresence::Missing => {
            key_value.with_value(|value| matches!(value, ValueView::Structure(_)))
        }
        FieldPresence::Unique(wrapping_data) => {
            key_value_is_byte_string
                && wrapping_data.with_value(|value| match value {
                    ValueView::Structure(data) => wrapping_structure_has_key_information(&data),
                    _ => false,
                })
        }
        FieldPresence::Duplicate => false,
    }
}

fn wrapping_structure_has_key_information(data: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(wrapping_method) = field_presence(data, WRAPPING_METHOD) else {
        return false;
    };
    if !wrapping_method.with_value(|value| matches!(value, ValueView::Enumeration(_))) {
        return false;
    }

    if !key_wrapping_data_has_valid_encoding_option(data) {
        return false;
    }

    let mut has_key_information = false;
    for tag in [ENCRYPTION_KEY_INFORMATION, MAC_SIGNATURE_KEY_INFORMATION] {
        match field_presence(data, tag) {
            FieldPresence::Missing => {}
            FieldPresence::Unique(field) => {
                if !field.with_value(|value| match value {
                    ValueView::Structure(information) => {
                        key_information_has_unique_identifier(&information)
                    }
                    _ => false,
                }) {
                    return false;
                }
                has_key_information = true;
            }
            FieldPresence::Duplicate => return false,
        }
    }
    has_key_information
}

fn key_wrapping_data_has_required_key_information(structure: &StructureView<'_>) -> bool {
    structure.children().iter().all(|field| {
        let wrapping_data_is_valid = if field.tag().raw() == KEY_WRAPPING_DATA {
            field.with_value(|value| match value {
                ValueView::Structure(data) => wrapping_structure_has_key_information(&data),
                _ => false,
            })
        } else {
            true
        };
        wrapping_data_is_valid
            && field.with_value(|value| match value {
                ValueView::Structure(nested) => {
                    key_wrapping_data_has_required_key_information(&nested)
                }
                _ => true,
            })
    })
}

fn key_wrapping_data_has_valid_encoding_option(data: &StructureView<'_>) -> bool {
    match field_presence(data, ENCODING_OPTION) {
        FieldPresence::Missing => {}
        FieldPresence::Unique(encoding_option) => {
            if !encoding_option.with_value(|value| matches!(value, ValueView::Enumeration(_))) {
                return false;
            }
        }
        FieldPresence::Duplicate => return false,
    }
    true
}

fn key_information_has_unique_identifier(information: &StructureView<'_>) -> bool {
    let FieldPresence::Unique(unique_identifier) = field_presence(information, UNIQUE_IDENTIFIER)
    else {
        return false;
    };
    unique_identifier.with_value(|value| matches!(value, ValueView::TextString(_)))
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

fn client_request_tags_use_kmip_prefixes(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.with_ttlv(|message| structure_tags_use_kmip_prefixes(&message))
}

fn client_mask_values_use_ttlv_integer_encoding(
    request: Option<&RequestMessage>,
    response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    let request_is_valid = request.with_ttlv(|message| mask_values_are_ttlv_integers(&message));
    let response_is_valid = response.is_none_or(|(_, response)| {
        response.with_ttlv(|message| mask_values_are_ttlv_integers(&message))
    });

    request_is_valid && response_is_valid
}

fn mask_values_are_ttlv_integers(structure: &StructureView<'_>) -> bool {
    structure.children().iter().all(|item| {
        item.with_value(|value| {
            if is_ttlv_bit_mask_tag(item.tag().raw()) {
                matches!(value, ValueView::Integer(_))
            } else {
                match value {
                    ValueView::Structure(nested) => mask_values_are_ttlv_integers(&nested),
                    _ => true,
                }
            }
        })
    })
}

fn is_ttlv_bit_mask_tag(tag: u32) -> bool {
    matches!(
        tag,
        CRYPTOGRAPHIC_USAGE_MASK | STORAGE_STATUS_MASK | PROTECTION_STORAGE_MASK
    )
}

fn client_request_tags_are_allocated(
    request: Option<&RequestMessage>,
    _response: Option<(&RequestMessage, &ResponseMessage)>,
) -> bool {
    let Some(request) = request else {
        return false;
    };

    request.with_ttlv(|message| structure_tags_are_allocated(&message))
}

fn structure_tags_are_allocated(structure: &StructureView<'_>) -> bool {
    structure.children().iter().all(|item| {
        RawTag::new(item.tag().raw())
            .and_then(|raw_tag| raw_tag.try_checked())
            .is_ok()
            && item.with_value(|value| match value {
                ValueView::Structure(nested) => structure_tags_are_allocated(&nested),
                _ => true,
            })
    })
}

fn structure_tags_use_kmip_prefixes(structure: &StructureView<'_>) -> bool {
    structure.children().iter().all(|item| {
        tag_has_kmip_21_prefix(item.tag().raw())
            && item.with_value(|value| match value {
                ValueView::Structure(nested) => structure_tags_use_kmip_prefixes(&nested),
                _ => true,
            })
    })
}

fn tag_has_kmip_21_prefix(raw_tag: u32) -> bool {
    let first_byte = (raw_tag >> 16) & 0xFF;
    matches!(first_byte, SPECIFICATION_TAG_PREFIX | EXTENSION_TAG_PREFIX)
}

const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-3.1-001",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-3.1-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_tls_for_basic_authentication_suite_transport",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "transport_security",
            value: "tls",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-3.1.1-002",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-3.1.1-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_recommends_tls_1_3_for_basic_authentication_suite_transport",
            kind: RuleTestKind::RecommendedBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "transport_security",
            value: "tls13",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-3.1.1-005",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-3.1.1-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_tls12_server_hello_before_request_dispatch",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "transport_security",
            value: "tls13",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-3.1.2-002",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-3.1.2-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_tls13_cipher_suite_support",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_tls12_server_hello_before_request_dispatch",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "tls13_cipher_suites",
            value: "kmip_profiles_3_1_2",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-3.1.2-005",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-3.1.2-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_prohibited_tls13_cipher_suites",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_tls12_server_hello_before_request_dispatch",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "tls13_cipher_suites",
            value: "kmip_profiles_3_1_2",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-2.3-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-2.3-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_opaque_object_context_custom_attributes),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_register_opaque_object_context_custom_attributes",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_get_attributes_opaque_object_custom_context",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-2.3-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-2.3-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_opaque_object_context_managed_attributes),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_register_opaque_object_context_managed_attributes",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::managed_opaque_context_accepts_table293_attributes_without_a_key_block_and_emits_no_marker",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/register_execution_tests.rs::managed_opaque_register_preserves_standard_table_291_wire_shape_and_attribute_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-2.3-003",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-2.3-OPAQUE-DATA-TYPE-REQUIRED"),
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_register_opaque_object_has_required_data_fields),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_fields_of_opaque_register_object",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-2.4-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-2.4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(import_preserves_pgp_key_form_and_opaque_bytes),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_pgp_key_forms_and_versions_as_opaque_bytes",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/import_operation_tests.rs::import_preserves_pgp_key_versions_and_opaque_key_bytes",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-2.4-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-2.4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(import_preserves_pgp_key_form_and_opaque_bytes),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_pgp_key_forms_and_versions_as_opaque_bytes",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/import_operation_tests.rs::import_preserves_pgp_key_versions_and_opaque_key_bytes",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-2.4-002-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-2.4-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: Some(import_preserves_pgp_key_form_and_opaque_bytes),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_pgp_key_forms_and_versions_as_opaque_bytes",
                kind: RuleTestKind::RecommendedBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/import_operation_tests.rs::import_preserves_pgp_key_versions_and_opaque_key_bytes",
                kind: RuleTestKind::RecommendedBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-2.7-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-2.7-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(import_preserves_wrapped_secret_data_shape),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_import_of_wrapped_secret_data",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/import_operation_tests.rs::import_preserves_wrapped_secret_data_without_key_wrap_type",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-2.8-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-2.8-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_split_key_part_identifier_is_within_declared_parts),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_constrains_split_key_part_identifier_to_declared_parts",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_constrains_split_key_part_identifier_to_declared_parts",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_split_key_rule_ignores_other_object_types_and_rejects_malformed_payloads",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-2.8-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-2.8-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_create_split_key_requires_prime_field_size_for_polynomial_method),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_prime_field_size_for_polynomial_split_key_requests",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_prime_field_size_for_polynomial_split_key_requests",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_split_key_tests.rs::polynomial_prime_field_request_requires_the_explicit_client_input",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_split_key_tests.rs::polynomial_prime_field_size_is_preserved_as_big_integer",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_split_key_tests.rs::table_193_prime_field_size_remains_optional_for_other_methods_and_future_values",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.3-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.3-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_key_wrapping_data_is_absent_or_has_required_key_information),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_key_wrapping_data_for_register_and_import",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.3-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.3-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_key_wrapping_data_is_absent_or_has_required_key_information),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_key_wrapping_data_for_register_and_import",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.3-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.3-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_caller_owned_wrapping_iv_and_ttlv_encoding_obligations",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "iv_counter_nonce_when_required_by_wrapping_method_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.3-006",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.3-006",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_caller_owned_wrapping_iv_and_ttlv_encoding_obligations",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "wrapped_key_value_ttlv_when_encoding_option_absent_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.3-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.3-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(client_no_encoding_excludes_key_value_attributes),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_key_value_attributes_when_wrapping_uses_no_encoding",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_key_value_attributes_when_wrapping_uses_no_encoding",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.3-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.3-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_key_wrapping_data_is_absent_or_has_required_key_information),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_key_information_when_wrapping_data_is_supplied",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.3-009",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.3-009",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_key_wrapping_data_is_absent_or_has_required_key_information),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_unique_identifier_for_key_wrapping_key_information",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_unique_identifier_for_key_wrapping_key_information",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.7-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.7-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_transparent_rsa_private_key_has_material_alternative),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_each_complete_transparent_rsa_private_key_material_alternative",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_transparent_rsa_private_key_without_any_complete_material_alternative",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_incomplete_transparent_rsa_private_key_component_pairs",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_does_not_inspect_wrapped_or_non_rsa_private_key_material_for_rsa_alternatives",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_represent_multi_instance_attributes),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_multi_instance_attribute_representation_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_distinct_multi_instance_attribute_values_as_recommended_behavior",
            kind: RuleTestKind::RecommendedBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "caller_supplies_distinct_values_for_multi_instance_attributes",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_preserves_single_instance_attribute_cardinality),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_single_instance_attributes_as_required_behavior",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_repeated_single_instance_attributes_in_get_attributes_response",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_repeated_single_instance_attributes_in_create_request",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_valid_single_instance_attribute_in_create_request",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_repeated_conditional_dsa_without_object_context",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_repeated_single_instance_attributes_in_all_object_attribute_requests",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_repeated_single_instance_attributes_in_nested_key_value_groups",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_repeated_single_instance_attributes_in_export_response",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_repeated_single_instance_attributes_in_export_key_value",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_repeated_dsa_for_pgp_certificate_context",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_repeated_dsa_for_pgp_key_object_type",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.4-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_application_specific_information_can_omit_data),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_application_specific_information_request_shapes",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.26-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.26-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_server_generated_key_format_type_is_enumeration),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_requested_key_format_for_server_generated_material_operations",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_tests.rs::request_encodes_raw_key_format_for_server_generated_create",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.1-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.1-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_wrapped_key_value_requires_key_wrapping_data),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_key_wrapping_data_for_wrapped_register_and_import_values",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_key_wrapping_data_for_wrapped_register_and_import_values",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.1-006",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.1-006",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_key_wrapping_data_is_only_present_for_wrapped_key_value),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_omits_key_wrapping_data_for_plaintext_register_and_import_values",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_omits_key_wrapping_data_for_plaintext_register_and_import_values",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.2-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.2-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_register_key_value_attributes_are_structured),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_key_value_attributes_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::request_preserves_key_value_attributes_associated_with_raw_key_material",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.1-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.1-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_get_or_export_key_compression_type_is_enumeration),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_export_key_compression_type_selector",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_get_key_compression_type_selector",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/get_tests.rs::request_preserves_the_optional_key_compression_type_enumeration",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.14-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.14-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_create_key_pair_domain_parameters_have_valid_wire_types),
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
        requirement_id: "KMIPKIT-REQ-SPEC-4.14-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.14-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_create_key_pair_domain_parameters_have_valid_wire_types),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_type_applicable_dsa_q_length_domain_parameter",
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
        requirement_id: "KMIPKIT-REQ-SPEC-5.1-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-5.1-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_repeated_attributes_are_representable),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_repeated_attributes_for_client_requests_and_responses",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/attribute_tests.rs::property_roundtrips_ordered_repeated_direct_attribute_items",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-5.2-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-5.2-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_create_key_pair_common_attributes_are_representable),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_repeated_create_key_pair_attribute_groups",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_key_pair_tests.rs::request_preserves_six_distinct_optional_groups_and_repeated_items_in_wire_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-5.3-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-5.3-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_create_key_pair_private_key_attributes_are_representable),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_repeated_create_key_pair_attribute_groups",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_key_pair_tests.rs::request_preserves_six_distinct_optional_groups_and_repeated_items_in_wire_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-5.4-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-5.4-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_create_key_pair_public_key_attributes_are_representable),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_repeated_create_key_pair_attribute_groups",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_key_pair_tests.rs::request_preserves_six_distinct_optional_groups_and_repeated_items_in_wire_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.7-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.7-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: Some(client_check_requires_multi_item_batch_when_check_is_present),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_recommends_check_only_in_a_multi_item_batch_without_ordering",
            kind: RuleTestKind::RecommendedBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.7-002-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.7-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: Some(client_check_batch_order_option_is_true_when_check_is_present),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_recommends_batch_order_true_when_check_is_present",
                kind: RuleTestKind::RecommendedBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/check_execution_tests.rs::check_leaves_recommended_batch_options_explicitly_selected_by_the_caller",
                kind: RuleTestKind::RecommendedBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.7-002-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.7-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: Some(
            client_check_batch_error_continuation_option_is_stop_or_undo_when_check_is_present,
        ),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_recommends_stop_or_undo_when_check_is_present",
            kind: RuleTestKind::RecommendedBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.7-002-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.7-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_check_additional_attributes_are_limited_to_listed_set),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_check_additional_attributes_to_the_listed_set",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.7-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.7-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_check_optional_usage_limits_count_is_long_integer),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_check_usage_limits_count",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/check_execution_tests.rs::check_sends_table_183_fields_and_exposes_table_184_response_once",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.23-002-001",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.23-USAGE-LIMITS-COUNT-REQUIRED"),
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_get_usage_allocation_requires_usage_limits_count),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_get_usage_allocation_required_usage_limits_count",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.25-006",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.25-006",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_import_preserves_wrapped_object_and_attributes),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_import_of_wrapped_object_and_attributes_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/import_execution_tests.rs::explicit_import_sends_supplied_attributes_and_object_once_and_preserves_server_error",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.4-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_archive_request_is_well_formed),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_archive_request_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/archive_operation_tests.rs::request_omits_the_optional_unique_identifier",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/archive_execution_tests.rs::archive_success_returns_server_result_without_claiming_archival_completion",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.7-004-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.7-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_check_optional_cryptographic_usage_mask_is_integer),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_check_intended_cryptographic_uses",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/check_execution_tests.rs::check_sends_table_183_fields_and_exposes_table_184_response_once",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.7-004-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.7-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_check_optional_cryptographic_usage_mask_is_integer),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_check_cryptographic_usage_mask",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/check_execution_tests.rs::locate_and_check_requests_preserve_distinct_cryptographic_usage_masks",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.7-005",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.7-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_check_optional_lease_time_is_interval),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_check_lease_time",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/check_execution_tests.rs::check_sends_table_183_fields_and_exposes_table_184_response_once",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.9-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.9-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_create_key_pair_common_attributes_are_representable),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_create_key_pair_attribute_assignment_groups",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_key_pair_tests.rs::request_preserves_six_distinct_optional_groups_and_repeated_items_in_wire_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_key_pair_tests.rs::request_distinguishes_absent_groups_from_present_empty_groups_without_synthesizing_choices",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.9-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.9-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_create_key_pair_differing_attributes_are_representable),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_create_key_pair_attribute_assignment_groups",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_key_pair_tests.rs::request_preserves_six_distinct_optional_groups_and_repeated_items_in_wire_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_key_pair_tests.rs::request_distinguishes_absent_groups_from_present_empty_groups_without_synthesizing_choices",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.9-006",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.9-006",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_create_key_pair_table_191_values_match),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_equal_create_key_pair_table_191_effective_values",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_unequal_create_key_pair_table_191_values",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_inherited_table_191_conflict_in_mixed_batch",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_key_pair_tests.rs::table_191_accepts_absent_values_and_common_attribute_fallback_without_copying",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_key_pair_tests.rs::table_191_rejects_one_sided_effective_values_and_conflicts_for_each_attribute",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.9-007",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.9-007",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_create_key_pair_table_191_values_match),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_equal_create_key_pair_table_191_effective_values",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_unequal_create_key_pair_table_191_values",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_inherited_table_191_conflict_in_mixed_batch",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_key_pair_tests.rs::table_191_accepts_absent_values_and_common_attribute_fallback_without_copying",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_key_pair_tests.rs::table_191_rejects_one_sided_effective_values_and_conflicts_for_each_attribute",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.1-004-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.1-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_key_block_algorithm_may_be_omitted_if_value_present),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_key_block_algorithm_and_length_omission_from_key_value",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_key_block_algorithm_and_length_omission_for_import",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.1-005-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.1-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_key_block_length_may_be_omitted_if_value_present),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_key_block_algorithm_and_length_omission_from_key_value",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_key_block_algorithm_and_length_omission_for_import",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.1-004-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.1-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_key_block_algorithm_requires_length),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_absent_or_present_direct_key_block_pairs",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-3.1-005-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-3.1-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_key_block_length_requires_algorithm),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_absent_or_present_direct_key_block_pairs",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.43-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.43-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_register_has_required_object_type_and_attributes),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_object_type_and_attributes_as_required",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::request_emits_table_291_fields_in_order_and_preserves_object_item",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.43-007",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.43-007",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_register_has_required_object_type_and_attributes),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_object_type_and_attributes_as_required",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::request_emits_table_291_fields_in_order_and_preserves_object_item",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.43-009-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.43-009",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_register_table293_pair_may_be_in_key_block),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_conditional_algorithm_length_and_signature_omissions",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_omission_exemptions_and_malformed_containers",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_omission_pair_metadata_branches",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::request_preserves_key_value_attributes_associated_with_raw_key_material",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.43-010-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.43-010",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_register_table293_pair_may_be_in_key_block),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_conditional_algorithm_length_and_signature_omissions",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_omission_exemptions_and_malformed_containers",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_omission_pair_metadata_branches",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::request_preserves_key_value_attributes_associated_with_raw_key_material",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.43-011",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.43-011",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_register_signature_algorithm_may_be_in_certificate),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_conditional_algorithm_length_and_signature_omissions",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_omission_exemptions_and_malformed_containers",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_omission_pair_metadata_branches",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/register_execution_tests.rs::register_client_omits_signature_algorithm_when_x509_certificate_encodes_it",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.43-009-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.43-009",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_register_algorithm_requires_length),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_register_algorithm_requires_length_in_attributes",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_does_not_apply_register_managed_crypto_pairing_to_context_unknown_opaque",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_exempts_secret_data_from_register_algorithm_length_requirement",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.43-010-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.43-010",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_register_length_requires_algorithm),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_register_length_requires_algorithm_in_attributes",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.27-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.27-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_register_key_value_location_requires_omitted_key_value),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_register_key_value_location_only_without_key_value",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.28-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.28-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(client_register_does_not_specify_key_value_present),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_key_value_present_in_register_but_preserves_locate_criterion",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_malformed_attributes_type_for_register_key_value_present",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_duplicate_attributes_for_register_key_value_present",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_checks_register_key_value_present_for_each_batch_item",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.28-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.28-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(client_key_value_present_is_not_modified),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_key_value_present_attribute_mutations",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4-001-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(client_read_only_attribute_modification_is_not_requested),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_read_only_attribute_modification_on_all_client_routes",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.57-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.57-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(client_state_modify_attribute_is_not_requested),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_client_modify_attribute_for_state",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.57-005-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.57-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: None,
        test_evidence: &[],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "compromised_object_protected_information_only_trusted_client_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4-001-005",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(client_read_only_attribute_deletion_is_not_requested),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_read_only_attribute_deletion",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.59-003-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.59-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(client_usage_limits_count_is_not_set_by_add_or_modify),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_usage_limits_count_in_add_and_modify_attribute",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/attribute_mutation_execution_tests.rs::usage_limits_new_attribute_is_rejected_only_for_add_and_modify",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.28-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.28-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_locate_key_value_present_is_boolean_when_used),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_locate_key_value_present_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/locate_tests.rs::locate_encodes_key_value_present_boolean_as_a_search_criterion",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.34-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.34-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_register_nist_key_type_is_enumeration),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_register_nist_key_type_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.59-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.59-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/profile_preflight_tests.rs::usage_limits_total_change_after_protection_use_is_rejected_with_bound_context",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/profile_preflight_tests.rs::usage_limits_total_change_after_allocation_without_count_change_uses_general_prohibition",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "usage_limits_total_immutable_after_protection_begins_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.59-003-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.59-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-client/tests/unit/profile_preflight_tests.rs::usage_limits_total_change_after_allocation_is_attributed_only_when_count_changed",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "usage_limits_total_immutable_after_count_allocation_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.34-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.34-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_single_key_usage_recommendation_as_caller_owned_behavior",
            kind: RuleTestKind::RecommendedBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "single_cryptographic_usage_per_key_object_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.31-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.31-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_omit_link_when_associated_key_material_is_unavailable),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_absent_link_for_unavailable_associated_key_material",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.31-004",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-4.31-LINK-TYPE-REQUIRED"),
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_link_type_is_required_in_object_state),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_complete_client_object_state_link_values",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_incomplete_client_object_state_links",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_incomplete_client_object_state_links_in_get_attributes_responses",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_correlates_reordered_get_attributes_link_responses_by_batch_id",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_allows_partial_link_locate_search_criteria",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.31-005",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-4.31-LINKED-OBJECT-IDENTIFIER-REQUIRED"),
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_linked_object_identifier_is_required_in_object_state),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_complete_client_object_state_link_values",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_incomplete_client_object_state_links",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_incomplete_client_object_state_links_in_get_attributes_responses",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_correlates_reordered_get_attributes_link_responses_by_batch_id",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_allows_partial_link_locate_search_criteria",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.32-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.32-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/unit/attribute_tests.rs::name_attribute_is_preserved_as_caller_supplied_text",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "name_unique_within_target_server_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.6-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.6-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_certify_requests_at_most_one_certificate),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_at_most_one_certify_certificate_request_value_per_batch_item",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.6-002-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.6-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_certify_certificate_request_value_is_omittable),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_omitted_certify_certificate_request_object_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.6-002-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.6-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_certify_omitted_request_requires_unique_identifier),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_omitted_certify_request_value_with_explicit_unique_identifier",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_certify_request_value_without_unique_identifier",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.6-002-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.6-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_certify_omitted_request_requires_certificate_type_attribute),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_omitted_certify_certificate_type_in_attributes",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.6-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.6-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_retrieve_certified_certificate_with_get_in_batch),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_certify_generated_certificate_get_in_same_batch_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/certify_execution_tests.rs::certify_can_produce_the_placeholder_for_a_get_in_the_same_batch",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.6-007",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.6-007",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_certify_request_type_required_with_request_value),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_certificate_request_type_with_request_value",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.43-005",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.43-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_register_preserves_optional_wrapped_key_value),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_register_wrapped_key_value_with_embedded_attributes",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/register_execution_tests.rs::register_preserves_caller_supplied_opaque_key_value_bytes_with_wrapping_data_from_table_291",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.35-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.35-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_object_group_names_are_representable),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_single_object_group_membership_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_tests.rs::create_preserves_two_repeated_object_group_text_values_in_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.35-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.35-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_object_group_names_are_representable),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_multiple_object_group_membership_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_tests.rs::create_preserves_two_repeated_object_group_text_values_in_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.35-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.35-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: Some(client_object_group_names_are_representable),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_recommends_object_group_name_attribute_when_assigning_group",
                kind: RuleTestKind::RecommendedBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_checks_object_group_names_on_every_client_assignment_route",
                kind: RuleTestKind::RecommendedBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_tests.rs::create_preserves_two_repeated_object_group_text_values_in_order",
                kind: RuleTestKind::RecommendedBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.38-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.38-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_register_original_creation_date_is_datetime_when_used),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_register_without_and_with_original_creation_date",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::request_can_omit_original_creation_date_when_registering",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.38-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.38-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_register_original_creation_date_is_datetime_when_used),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_register_without_and_with_original_creation_date",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::request_encodes_client_supplied_original_creation_date_on_register",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.38-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.38-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_add_original_creation_date_is_datetime_when_used),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_add_attribute_original_creation_date",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/add_attribute_tests.rs::request_encodes_original_creation_date_as_new_attribute",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.39-002-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.39-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: Some(client_pkcs12_friendly_name_uses_lowercase_text),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_recommends_lowercase_pkcs12_friendly_name_values",
            kind: RuleTestKind::RecommendedBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.38-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.38-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/profile_preflight_tests.rs::original_creation_date_mutation_with_present_bound_context_is_rejected_before_transport",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/profile_preflight_tests.rs::original_creation_date_non_add_mutation_with_known_absent_state_is_rejected",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "original_creation_date_immutable_after_set_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.46-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.46-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_register_random_number_generator_is_well_formed),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_register_random_number_generator",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::request_encodes_rng_algorithm_directly_on_register",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.46-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.46-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_add_attribute_random_number_generator_is_well_formed),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_add_attribute_random_number_generator",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/add_attribute_tests.rs::request_encodes_rng_algorithm_directly_in_new_attribute",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.36-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.36-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_rng_parameters_are_well_formed),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_shared_rng_parameters_to_client_request_and_response_rules",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_rng_parameters_on_client_create_operation_attribute_groups",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_get_attributes_rng_parameters",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_rng_parameters_with_optional_fields",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.36-002-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.36-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_rng_parameters_are_well_formed),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_shared_rng_parameters_to_client_request_and_response_rules",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_rng_parameters_on_client_create_operation_attribute_groups",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_get_attributes_rng_parameters",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_register_random_number_generator",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_rng_parameters_without_rng_algorithm",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_rng_algorithm_with_wrong_type_or_duplicate",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.36-002-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.36-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_rng_parameters_are_well_formed),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_shared_rng_parameters_to_client_request_and_response_rules",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_rng_parameters_on_client_create_operation_attribute_groups",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_get_attributes_rng_parameters",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_unspecified_rng_algorithm_when_details_are_unknown",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.36-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.36-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_rng_parameters_are_well_formed),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_shared_rng_parameters_to_client_request_and_response_rules",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_rng_parameters_on_client_create_operation_attribute_groups",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_get_attributes_rng_parameters",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_correlates_reordered_get_attributes_rng_responses_by_batch_id",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_rng_parameters_with_optional_fields",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_rng_parameters_with_malformed_optional_field_type",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_duplicate_rng_optional_field",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.1-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.1-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_activation_date_use_as_caller_owned_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "activation_date_reached_before_use_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.1-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.1-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_pre_activation_use_restriction_as_caller_owned_prohibition",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "no_cryptographic_use_before_activation_date_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.1-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.1-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_activation_date_mutation_restriction_as_caller_owned_prohibition",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "activation_date_immutable_after_pre_active_until_destruction_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.17-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.17-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_usage_mask_eligibility_as_caller_owned_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "target_usage_mask_allows_requested_function_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.17-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.17-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_usage_mask_restriction_as_caller_owned_prohibition",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "target_usage_mask_excludes_requested_function_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.18-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.18-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_deactivated_object_exceptions_as_caller_owned_prohibition",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "deactivation_date_exceptional_use_requires_special_permission_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.18-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.18-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_deactivation_date_mutation_restriction_as_caller_owned_prohibition",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "deactivation_date_change_only_in_pre_active_or_active_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.40-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.40-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_process_start_and_protect_stop_date_rules_as_caller_owned",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "process_start_date_use_after_date_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.40-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.40-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_process_start_and_protect_stop_date_rules_as_caller_owned",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "no_cryptographic_use_before_process_start_date_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.40-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.40-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_process_start_and_protect_stop_date_rules_as_caller_owned",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "process_start_date_may_equal_or_follow_activation_date_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.40-001-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.40-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_process_start_and_protect_stop_date_rules_as_caller_owned",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "process_start_date_must_not_precede_activation_date_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.40-001-005",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.40-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_process_start_and_protect_stop_date_rules_as_caller_owned",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "process_start_date_immutable_after_occurrence_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.41-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.41-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_process_start_and_protect_stop_date_rules_as_caller_owned",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "no_cryptographic_protection_after_protect_stop_date_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.41-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.41-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_process_start_and_protect_stop_date_rules_as_caller_owned",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "protect_stop_date_may_equal_or_precede_deactivation_date_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.41-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.41-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_process_start_and_protect_stop_date_rules_as_caller_owned",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "protect_stop_date_must_not_follow_deactivation_date_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.41-001-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.41-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_process_start_and_protect_stop_date_rules_as_caller_owned",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "protect_stop_date_immutable_after_occurrence_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.30-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.30-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_lease_expiry_restriction_as_caller_owned_prohibition",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "lease_not_expired_or_renewed_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.47-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.47-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_revoke_revocation_message_is_text_when_used),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_optional_revoke_revocation_message",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_revoke_without_revocation_message",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/revoke_operation_tests.rs::request_encodes_optional_audit_message_inside_revocation_reason_structure",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.47-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.47-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_revoke_revocation_message_is_text_when_used),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_optional_revoke_revocation_message",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_revoke_without_revocation_message",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/revoke_operation_tests.rs::request_encodes_optional_audit_message_inside_revocation_reason_structure",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.23-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.23-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_usage_allocation_before_protection_as_caller_owned_prohibition",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "usage_allocation_obtained_before_protection_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.23-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.23-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_usage_allocation_date_window_as_caller_owned_required_behavior",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "usage_allocation_requested_within_object_date_window_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.23-002-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.23-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_consumed_allocation_restriction_as_caller_owned_prohibition",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "usage_allocation_remains_available_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.35-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.35-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_post_expiry_lease_restriction_as_caller_owned_prohibition",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "lease_not_expired_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.44-003-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.44-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: Some(
            client_revoke_compromise_occurrence_date_recommended_for_compromise_reasons,
        ),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_recommends_revoke_compromise_occurrence_date_for_compromise_reasons",
            kind: RuleTestKind::RecommendedBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.44-003-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.44-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(client_compromise_occurrence_date_only_for_key_or_ca_compromise),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_revoke_compromise_occurrence_date_for_other_reasons",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_revoke_compromise_occurrence_date_when_reason_is_malformed",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.45-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.45-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/recertify_execution_tests.rs::recertify_sends_one_table_300_request_and_exposes_table_301_identifier",
                kind: RuleTestKind::RecommendedBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_recertify_once_per_existing_certificate_as_caller_owned_recommendation",
                kind: RuleTestKind::RecommendedBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "recertify_once_per_existing_certificate_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.46-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.46-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/rekey_execution_tests.rs::rekey_sends_one_table_305_request_and_exposes_replacement_identifier",
                kind: RuleTestKind::RecommendedBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_rekey_once_per_existing_key_as_caller_owned_recommendation",
                kind: RuleTestKind::RecommendedBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "rekey_once_per_existing_key_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.47-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.47-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/rekey_key_pair_execution_tests.rs::rekey_key_pair_sends_table_310_and_returns_both_identifiers_once",
                kind: RuleTestKind::RecommendedBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_rekey_key_pair_once_per_existing_pair_as_caller_owned_recommendation",
                kind: RuleTestKind::RecommendedBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "rekey_key_pair_once_per_existing_pair_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.48-006",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.48-006",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/reprovision_execution_tests.rs::reprovision_sends_certificate_request_form_and_typed_identifier_once",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_reprovision_when_credentials_are_required_as_caller_directed_behavior",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "reprovision_when_credentials_are_required_caller_directed",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.45-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.45-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_recertify_requests_at_most_one_certificate),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_recertify_single_certificate_per_batch_item_as_required_behavior",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.45-002-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.45-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_recertify_identifier_required_when_request_value_is_omitted),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_recertify_identifier_when_request_value_is_omitted",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/recertify_execution_tests.rs::recertify_sends_explicit_unique_identifier_without_certificate_request_fields",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.45-002-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.45-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_recertify_identifier_required_when_request_value_is_omitted),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_recertify_identifier_when_request_value_is_omitted",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_recertify_without_identifier_when_request_value_is_present",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_recertify_id_placeholder_when_request_value_is_present",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_recertify_without_identifier_when_request_value_is_omitted",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_recertify_id_placeholder_when_request_value_is_omitted",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.45-006-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.45-006",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_recertify_offset_is_optional_interval),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_recertify_offset_as_optional_interval",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/recertify_operation_tests.rs::request_emits_table_300_fields_once_in_order_and_preserves_values",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/recertify_execution_tests.rs::recertify_sends_one_table_300_request_and_exposes_table_301_identifier",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.45-008",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.45-008",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_recertify_request_type_required_with_request_value),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_recertify_request_type_with_request_value",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_recertify_request_without_certificate_request_fields",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/recertify_operation_tests.rs::request_requires_type_when_certificate_request_value_is_present",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/recertify_execution_tests.rs::recertify_sends_one_table_300_request_and_exposes_table_301_identifier",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.46-004-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.46-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_rekey_offset_is_optional_interval),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_rekey_offset_as_optional_interval",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/rekey_operation_tests.rs::request_emits_table_305_fields_once_in_order_and_preserves_values",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/rekey_execution_tests.rs::rekey_sends_one_table_305_request_and_exposes_replacement_identifier",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.47-004-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.47-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_rekey_key_pair_offset_is_optional_interval),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_rekey_key_pair_offset_as_optional_interval",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/rekey_key_pair_operation_tests.rs::request_emits_table_310_fields_once_in_order_and_preserves_groups",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/rekey_key_pair_execution_tests.rs::rekey_key_pair_sends_table_310_and_returns_both_identifiers_once",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.48-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.48-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_reprovision_request_fields_are_well_formed),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_reprovision_certificate_request_forms",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/reprovision_operation_tests.rs::request_supports_certificate_request_certificate_or_no_parameters",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/reprovision_operation_tests.rs::request_preserves_both_optional_certificate_fields",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/reprovision_execution_tests.rs::reprovision_sends_certificate_request_form_and_typed_identifier_once",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/reprovision_execution_tests.rs::reprovision_sends_parameterless_server_credential_form",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/reprovision_execution_tests.rs::reprovision_sends_replacement_certificate_form",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.55-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.55-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_sign_cryptographic_parameters_are_optional_structure),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_sign_cryptographic_parameters_omission_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/sign_operation_tests.rs::request_allows_optional_identifier_and_parameters_to_be_omitted",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/sign_execution_tests.rs::request_table_fields_reach_the_fake_transport_unchanged",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.50-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.50-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(client_accepts_rng_seed_zero_data_length),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_rng_seed_response_when_server_uses_zero_seed_bytes",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/rng_seed_operation_tests.rs::successful_response_preserves_zero_data_length_and_unknown_fields",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/rng_seed_execution_tests.rs::rng_seed_sends_secret_once_and_accepts_zero_bytes_used",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.51-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.51-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(client_set_attribute_does_not_mutate_read_only_attributes),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_set_attribute_on_read_only_standard_attribute",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.56-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.56-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_signature_verify_original_data_is_optional_byte_string),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_signature_verify_original_data_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/signature_verify_operation_tests.rs::request_emits_optional_key_parameters_data_digest_signature_and_framing_in_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/signature_verify_execution_tests.rs::request_table_337_single_part_fields_reach_the_fake_transport_unchanged",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.56-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.56-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_signature_verify_cryptographic_parameters_are_optional_structure),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_signature_verify_cryptographic_parameters_omission_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/signature_verify_operation_tests.rs::request_allows_optional_key_parameters_and_input_data_to_be_omitted",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/signature_verify_execution_tests.rs::request_table_337_single_part_fields_reach_the_fake_transport_unchanged",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.32-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.32-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_mac_cryptographic_parameters_are_optional_structure),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_mac_cryptographic_parameters_omission_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/mac_operation_tests.rs::request_allows_optional_identifier_and_parameters_to_be_omitted",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.33-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.33-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_mac_verify_original_data_is_optional_byte_string),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_mac_verify_original_data_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/mac_verify_operation_tests.rs::request_emits_optional_identifier_parameters_original_data_mac_and_framing_in_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.33-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.33-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_mac_verify_cryptographic_parameters_are_optional_structure),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_mac_verify_cryptographic_parameters_omission_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/mac_verify_operation_tests.rs::request_allows_optional_key_parameters_and_original_data_to_be_omitted",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.57-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.57-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_validate_chain_list_members_are_usable),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_validate_single_chain_required_behavior_over_flat_list_inputs",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.57-002-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.57-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_validate_validity_date_is_usable),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_validate_validity_date_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.57-002-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.57-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_validate_chain_list_members_are_usable),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_validate_mixed_chain_inputs_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.57-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.57-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_validate_repeated_certificates_are_usable),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_validate_repeated_certificates_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.57-005",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.57-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_validate_repeated_unique_identifiers_are_usable),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_validate_repeated_unique_identifiers_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.62-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.62-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_response_preserves_x509_optional_values),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_get_attributes_response_x509_values",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.63-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.63-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_response_preserves_x509_optional_values),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_get_attributes_response_x509_values",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.63-005",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.63-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_response_preserves_x509_optional_values),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_get_attributes_response_x509_values",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.35-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.35-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_obtain_lease_response_values_are_typed),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_obtain_lease_response_capabilities",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_obtain_lease_capabilities_in_ordered_and_id_correlated_mixed_batches",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/obtain_lease_operation_tests.rs::successful_response_preserves_identifier_zero_lease_time_date_and_extensions",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.35-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.35-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_obtain_lease_response_values_are_typed),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_obtain_lease_response_capabilities",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_obtain_lease_capabilities_in_ordered_and_id_correlated_mixed_batches",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/obtain_lease_operation_tests.rs::successful_response_preserves_identifier_zero_lease_time_date_and_extensions",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.35-005",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.35-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_obtain_lease_response_values_are_typed),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_obtain_lease_response_capabilities",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_obtain_lease_capabilities_in_ordered_and_id_correlated_mixed_batches",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/obtain_lease_operation_tests.rs::successful_response_exposes_positive_lease_interval",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_repeat_request_batch_items),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_multiple_client_operations_in_one_batch_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/encrypt_decrypt_id_placeholder_execution_tests.rs::later_encrypt_omits_identifier_with_batch_order_enabled",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-8.1-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-8.1-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_repeat_request_batch_items),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_request_batch_item_repetition_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-8.2-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-8.2-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_repeat_request_attestation_types),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_repeated_request_attestation_types_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.3-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.3-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_sets_attestation_indicator_true),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_attestation_capable_indicator_when_client_can_create_credential",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/attestation_indicator_tests.rs::execute_advertises_attestation_without_authentication_or_credential_payload",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/attestation_indicator_tests.rs::async_follow_up_request_advertises_attestation_capability",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_public_api.rs::standalone_attestation_hashed_password_and_nonce_views_are_lossless",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.4-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_authentication_contains_a_credential_if_present),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_a_credential_when_request_authentication_is_present",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_a_credential_when_request_authentication_is_present",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.4-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_omit_authentication_when_not_required),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_authentication_and_repeated_credentials",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/attestation_indicator_tests.rs::execute_advertises_attestation_without_authentication_or_credential_payload",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.4-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.4-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_repeat_credentials_within_authentication),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_authentication_and_repeated_credentials",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_contract.rs::authentication_preserves_nonempty_credential_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_contract.rs::authentication_roundtrip_preserves_unknown_fields_and_credential_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.11-004-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.11-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_device_credential_has_identifier_if_present),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_a_device_identifier_field_in_device_credentials",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_a_device_identifier_field_in_device_credentials",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.11-004-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.11-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_contract.rs::device_requires_a_named_identifier_and_preserves_all_six_text_fields",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_public_api.rs::typed_credential_wrappers_expose_views_and_lossless_conversions",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "device_identifiers_caller_asserted_unique",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.11-004-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.11-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_device_credential_password_is_text_string_if_present),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_device_password_authentication_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_contract.rs::device_requires_a_named_identifier_and_preserves_all_six_text_fields",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.11-006",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.11-006",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_attestation_credential_has_evidence_if_present),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_attestation_measurement_or_assertion_when_attestation_is_used",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_attestation_measurement_or_assertion_when_attestation_is_used",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_contract.rs::attestation_requires_nonce_type_and_one_or_both_evidence_fields",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_contract.rs::attestation_requires_nonce_type_and_one_or_both_evidence_fields",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.11-010-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.11-010",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_contract.rs::hashed_password_timestamp_sequence_accepts_strictly_increasing_values",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_contract.rs::hashed_password_timestamp_sequence_rejects_equal_or_earlier_values",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_contract.rs::hashed_password_timestamp_sequences_are_independent",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_hashed_password_timestamp_monotonicity_as_caller_owned_behavior",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "hashed_password_timestamp_monotonicity_per_username",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.11-010-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.11-010",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_contract.rs::hashed_password_omission_exposes_sha256_default_without_materializing_field",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_contract.rs::hashed_password_preserves_explicit_and_unknown_algorithm_values",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "hashed_password_sha256_default",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.11-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.11-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_public_api.rs::credential_and_authentication_public_views_preserve_order_and_redact",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/credential_public_api.rs::typed_credential_wrappers_expose_views_and_lossless_conversions",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "credential_authentication_representation",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.5-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.5-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_uses_async_correlation_value_for_poll_or_cancel),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_poll_cancel_asynchronous_correlation_requirements",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/cancel_tests.rs::cancel_request_preserves_exact_correlation_bytes_in_table_order",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/cancel_execution_tests.rs::cancel_exposes_matching_echo_and_assigned_cancellation_result",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.38-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.38-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_uses_async_correlation_value_for_poll_or_cancel),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_poll_cancel_asynchronous_correlation_requirements",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/poll_tests.rs::poll_request_preserves_arbitrary_correlation_bytes_in_table_order",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/poll_execution_tests.rs::pending_poll_returns_without_repeating_and_zeroizes_request_and_response_copies",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.1-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.1-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_uses_async_correlation_value_for_poll_or_cancel),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_poll_cancel_asynchronous_correlation_requirements",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/recover_execution_tests.rs::caller_issues_poll_after_recover_pending_with_the_returned_correlation",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/poll_tests.rs::poll_request_preserves_arbitrary_correlation_bytes_in_table_order",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.19-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.19-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_uses_async_correlation_value_for_poll_or_cancel),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_poll_cancel_asynchronous_correlation_requirements",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/execute_lifecycle_tests.rs::pending_correlation_is_exact_borrowed_and_redacted_from_formatted_output",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/recover_execution_tests.rs::caller_issues_poll_after_recover_pending_with_the_returned_correlation",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.9-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.9-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_include_client_correlation_value),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_client_correlation_value_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_validation.rs::message_headers_preserve_independent_client_correlation_values",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/extension_outbound.rs::execute_encodes_every_optional_batch_header_and_item_identifier",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.12-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.12-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_include_maximum_response_size),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_maximum_response_size_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.12-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.12-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/create_split_key_execution_tests.rs::create_split_key_advertises_local_response_cap_clamped_to_signed_integer",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/create_split_key_execution_tests.rs::create_split_key_accepts_response_exactly_at_local_limit",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/create_split_key_execution_tests.rs::create_split_key_rejects_one_byte_over_local_limit_before_decoder_entry",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "declared_response_capacity",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.12-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.12-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: Some(client_sends_maximum_response_size_only_for_possible_large_replies),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_maximum_response_size_only_for_possible_large_replies",
                kind: RuleTestKind::RecommendedBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/create_split_key_execution_tests.rs::create_split_key_accepts_response_exactly_at_local_limit",
                kind: RuleTestKind::RecommendedBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.20-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.20-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_include_request_time_stamp),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_request_time_stamp_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_contract.rs::request_time_stamp_is_optional_and_preserved_as_raw_date_time",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.20-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.20-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_countdown_relative_date_use_as_caller_owned_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_accessors.rs::response_header_accessors_expose_every_represented_field",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "countdown_timer_relative_dates_from_server_timestamp_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.21-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.21-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_include_optional_unique_batch_item_id),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_request_unique_batch_item_id_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_validation.rs::request_item_ids_remain_associated_in_seeded_batch_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-8.3-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-8.3-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_includes_unique_batch_item_id_for_multi_item_request),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_unique_batch_item_id_for_multi_item_requests",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-8.3-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-8.3-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_repeat_request_message_extensions),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_repeated_request_message_extensions_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_validation.rs::repeated_request_extensions_preserve_order_and_expose_typed_fields",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-8-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-8-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_failure_response_payload_and_message_extensions_for_client_messages",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_contract.rs::response_payload_is_required_for_non_failure_and_absent_for_failure",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_accessors.rs::request_batch_and_extension_accessors_preserve_order_and_values",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_accessors.rs::response_batch_and_extension_accessors_preserve_all_result_shapes",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "optional_failure_response_payload_and_message_extensions",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-8-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-8-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_message_field_order_for_client_requests_and_responses",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_contract.rs::valid_request_and_response_preserve_envelope_order_and_counts",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_contract.rs::request_known_fields_and_envelope_reject_reordering",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_contract.rs::response_known_fields_and_envelope_reject_reordering",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "request_response_fields_follow_kmip_specified_order",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.7-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.7-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_batch_items_for_client_requests_and_responses",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_contract.rs::valid_request_and_response_preserve_envelope_order_and_counts",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_contract.rs::signed_batch_count_must_be_nonnegative_and_match_item_count",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "request_response_include_one_or_more_batch_items",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.13-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.13-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_vendor_extension_structure_and_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_validation.rs::repeated_request_extensions_preserve_order_and_expose_typed_fields",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "caller_selects_message_extension_presence_and_processing",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.13-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.13-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_vendor_extension_structure_and_optional_capability",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_validation.rs::repeated_request_extensions_preserve_order_and_expose_typed_fields",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_validation.rs::malformed_request_message_extension_is_rejected",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "message_extension_fields_and_vendor_data_validated_by_message_model",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.13-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.13-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_vendor_extension_structure_and_optional_capability",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_validation.rs::repeated_request_extensions_preserve_order_and_expose_typed_fields",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/message_validation.rs::malformed_request_message_extension_is_rejected",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "vendor_identification_is_caller_assigned_unique_text",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.13-001-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.13-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_rejects_unknown_critical_response_extensions),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_response_with_critical_message_extension",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_response_with_critical_message_extension",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/option_tests.rs::unknown_critical_message_extension_rejects_the_response",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/execute_lifecycle_tests.rs::noncritical_response_extension_is_preserved_and_critical_extension_is_rejected",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.13-001-005",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.13-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_process_unknown_noncritical_response_extensions),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_noncritical_response_extension_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/option_tests.rs::unknown_non_critical_message_extension_is_preserved_opaque",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/execute_lifecycle_tests.rs::noncritical_response_extension_is_preserved_and_critical_extension_is_rejected",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.5-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.5-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_sets_batch_count_to_request_item_count),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_batch_count_to_request_item_count",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.5-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.5-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_sets_batch_count_to_one_for_single_operation),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_single_operation_batch_count_to_one",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.6-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.6-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_includes_batch_error_continuation_only_for_multi_item_request),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_batch_error_continuation_option_only_for_multi_item_batches",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.6-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.6-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_uses_assigned_batch_error_continuation_values),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_assigned_batch_error_continuation_values",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/option_tests.rs::outbound_batch_error_continuation_accepts_the_three_assigned_values",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.6-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.6-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_defaults_batch_error_continuation_to_stop),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_default_batch_error_continuation_to_stop",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-4.1.2-005",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-4.1.2-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_omit_authentication_when_not_required),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_request_authentication_from_profiles_clause",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-4.1.2-007",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-4.1.2-007",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_include_optional_unique_batch_item_id),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_single_item_unique_batch_item_id_as_optional_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/execute_lifecycle_tests.rs::single_item_request_may_include_or_omit_unique_batch_item_id",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-9.17-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-9.17-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_inspect_optional_result_message),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::result_message_client_access_has_an_optional_typed_rule",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/result_contract.rs::result_message_preserves_presence_and_text",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/result_contract.rs::operation_result_display_and_debug_redact_message",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.16-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(type_specific_parameters_do_not_infer_remote_object_type),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_allows_type_specific_cryptographic_parameters_without_remote_object_inference",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(type_specific_parameters_do_not_infer_remote_object_type),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_optional_cryptographic_parameters_for_encrypt_and_decrypt",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/cryptographic_parameters_tests.rs::omitted_cryptographic_parameters_remain_absent_without_validation",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/encrypt_tests.rs::request_omits_cryptographic_parameters_when_the_caller_omits_them",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/decrypt_tests.rs::request_omits_cryptographic_parameters_and_iv_when_the_caller_omits_them",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.16-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.16-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_variable_iv_cryptographic_parameters_include_iv_length),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_variable_iv_modes_to_required_iv_length",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.16-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.16-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_gcm_cryptographic_parameters_include_tag_length),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_gcm_tag_length_as_required_behavior",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.17-006",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.17-006",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_encrypt_aad_is_on_initial_request),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_multipart_aead_fields_to_initial_requests",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/encrypt_decrypt_multipart_execution_tests.rs::encrypt_caller_reuses_initial_server_correlation_for_middle_and_final_parts",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.17-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.17-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_encrypt_cryptographic_parameters_are_optional_structure),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_encrypt_cryptographic_parameters_omission_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.17-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.17-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_encrypt_iv_counter_nonce_is_optional_byte_string),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_encrypt_iv_counter_nonce_omission_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.11-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.11-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_decrypt_cryptographic_parameters_are_optional_structure),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_decrypt_optional_fields_from_section_6_1_11",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/decrypt_tests.rs::request_omits_cryptographic_parameters_and_iv_when_the_caller_omits_them",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.11-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.11-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_decrypt_iv_counter_nonce_is_optional_byte_string),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_decrypt_optional_fields_from_section_6_1_11",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/decrypt_tests.rs::request_omits_cryptographic_parameters_and_iv_when_the_caller_omits_them",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.11-005",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.11-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_decrypt_aad_is_on_initial_request),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_multipart_aead_fields_to_initial_requests",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/encrypt_decrypt_multipart_execution_tests.rs::decrypt_caller_reuses_initial_server_correlation_and_keeps_aad_and_tag_initial",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.11-006",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.11-006",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_decrypt_tag_is_on_initial_request),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_multipart_aead_fields_to_initial_requests",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/encrypt_decrypt_multipart_execution_tests.rs::decrypt_caller_reuses_initial_server_correlation_and_keeps_aad_and_tag_initial",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.12-003",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.12-RIGHTS-REQUIRED"),
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_delegated_login_rights_is_required_structure),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_delegated_login_rights_as_required_structure",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/delegated_login_operation_tests.rs::request_emits_table_199_fields_in_order_and_preserves_both_lease_time_forms",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.12-005",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.12-LEASE-TIME-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_delegated_login_lease_time_is_optional_interval_or_date_time),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_delegated_login_lease_time_as_optional_interval_or_date_time",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/delegated_login_operation_tests.rs::request_emits_table_199_fields_in_order_and_preserves_both_lease_time_forms",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.12-006",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.12-REQUEST-COUNT-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_delegated_login_request_count_is_optional_integer),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_delegated_login_request_count_as_optional_integer",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/delegated_login_operation_tests.rs::request_emits_table_199_fields_in_order_and_preserves_both_lease_time_forms",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.12-007",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.12-USAGE-LIMITS-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_delegated_login_usage_limits_is_optional_structure),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_delegated_login_usage_limits_as_optional_structure",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/delegated_login_operation_tests.rs::request_emits_table_199_fields_in_order_and_preserves_both_lease_time_forms",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.30-001",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.30-LEASE-TIME-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_login_lease_time_is_optional_interval_or_date_time),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_login_optional_lease_time_as_optional_interval_or_date_time",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/login_operation_tests.rs::optional_request_fields_keep_table_253_order_and_both_lease_forms",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.30-002",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.30-REQUEST-COUNT-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_login_request_count_is_optional_integer),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_login_optional_request_count_as_integer",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/login_operation_tests.rs::optional_request_fields_keep_table_253_order_and_both_lease_forms",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.30-003",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.30-USAGE-LIMITS-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_login_usage_limits_is_optional_structure),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_login_optional_usage_limits_as_table_392_structure",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/login_operation_tests.rs::optional_request_fields_keep_table_253_order_and_both_lease_forms",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.31-001",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.31-TICKET-REQUIRED"),
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_logout_ticket_is_required_structure),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_logout_required_ticket_as_required_behavior",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/logout_operation_tests.rs::table_256_request_requires_and_preserves_the_explicit_secret_ticket",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(discover_versions_advertises_only_client_supported_versions),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::production_discover_versions_rule_accepts_only_kmip_21_offers",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.10-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.10-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_include_optional_create_split_key_source_identifier),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_create_split_key_optional_source_identifier",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(discover_versions_advertises_only_client_supported_versions),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_discover_versions_preference_order_for_single_supported_version",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.13-005",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.13-CURRENT-ATTRIBUTE-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_represent_optional_delete_attribute_selector),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_delete_attribute_current_attribute_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.13-006",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.13-ATTRIBUTE-REFERENCE-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_represent_optional_delete_attribute_selector),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_delete_attribute_reference_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.34-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.34-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_represent_optional_modify_attribute_selector),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_modify_attribute_current_attribute_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.34-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.34-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/unit/modify_attribute_tests.rs::request_preserves_exact_current_and_new_items_in_table_265_order",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "modify_attribute_existing_instance_named",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.27-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.27-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_join_split_key_has_validated_identifier_entries),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_join_split_key_threshold_behavior_using_typed_request_evidence",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_join_split_key_threshold_behavior_using_typed_request_evidence",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/join_split_key_operation_tests.rs::entry_count_must_meet_the_maximum_threshold_and_thresholds_may_differ",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/join_split_key_operation_tests.rs::entry_count_must_meet_the_maximum_threshold_and_thresholds_may_differ",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/join_split_key_execution_tests.rs::absent_or_insufficient_threshold_context_fails_before_transport",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.27-002-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.27-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_join_split_key_secret_data_type_is_valid_when_applicable),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_join_split_key_secret_data_type",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.27-005",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.27-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_join_split_key_has_validated_identifier_entries),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_repeated_join_split_key_identifiers_as_an_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.28-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.28-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_include_optional_locate_maximum_items),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_locate_maximum_items_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.28-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.28-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_include_optional_locate_offset_items),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_locate_offset_items_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.28-007",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.28-007",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_can_send_partial_structured_locate_attributes),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_locate_partial_structured_attribute_fields",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.28-013-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.28-013",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_supplies_locate_attributes_structure),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_locate_attributes_as_required_behavior",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.28-013-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.28-013",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_supplies_locate_attributes_structure),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_locate_empty_attributes_match_all_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.20-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.20-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(get_attributes_references_are_not_repeated),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_repeated_identical_get_attributes_references",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.20-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.20-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_include_optional_get_attributes_attribute_references),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_get_attributes_optional_attribute_references",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.18-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.18-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_export_controls_have_get_semantics),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_export_controls_with_get_semantics",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.18-005",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.18-KEY-FORMAT-TYPE-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_export_key_format_type_is_enumeration),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_export_optional_format_and_wrap_selectors",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/export_operation_tests.rs::request_preserves_all_explicit_optional_choices_and_omits_absent_fields",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.18-006",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.18-KEY-WRAP-TYPE-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_export_key_wrap_type_is_enumeration),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_export_optional_format_and_wrap_selectors",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/export_operation_tests.rs::request_preserves_all_explicit_optional_choices_and_omits_absent_fields",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.18-007",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.18-KEY-COMPRESSION-TYPE-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_get_or_export_key_compression_type_is_enumeration),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_export_key_compression_type_selector",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.18-008",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.18-KEY-WRAPPING-SPECIFICATION-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_get_or_export_optional_wrapping_specification_is_structure),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_export_optional_wrapping_specification",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.19-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.19-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_get_or_export_key_format_type_is_enumeration),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_get_and_export_key_format_client_assumption",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.19-003",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.19-KEY-FORMAT-TYPE-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_get_key_format_type_is_enumeration),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_get_return_format_controls",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.19-004",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.19-KEY-WRAP-TYPE-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_get_key_wrap_type_is_enumeration),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_get_return_format_controls",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.19-005",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.19-KEY-COMPRESSION-TYPE-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_get_or_export_key_compression_type_is_enumeration),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_get_return_format_controls",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.19-006",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.19-KEY-WRAPPING-SPECIFICATION-OPTIONAL"),
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_get_optional_wrapping_specification_is_structure),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_get_return_format_controls",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.31-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.31-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/unit/query_operation_tests.rs::query_response_exposes_all_table_283_members_and_unknown_nested_items",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "profile_specific_query_information_preserved",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.32-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.32-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_query_profile_information_is_well_formed),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_complete_version_in_query_profile_responses",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_complete_version_in_query_profile_responses",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.8-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.8-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_multipart_continuation_has_server_correlation),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_server_generated_correlation_to_multipart_continuations",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/encrypt_decrypt_multipart_execution_tests.rs::encrypt_caller_reuses_initial_server_correlation_for_middle_and_final_parts",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/encrypt_decrypt_multipart_execution_tests.rs::decrypt_caller_reuses_initial_server_correlation_and_keeps_aad_and_tag_initial",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/encrypt_decrypt_multipart_execution_tests.rs::every_typed_multipart_operation_reuses_server_correlation_in_follow_up",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.18-006",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.18-006",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_repeat_get_or_export_wrapping_attribute_names),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_repeated_get_and_export_wrapping_attribute_names_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.18-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.18-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_supports_get_or_export_wrapped_key_request),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_get_and_export_wrapped_key_requests_to_key_wrapping_specification",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.18-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.18-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_get_or_export_wrapping_specification_has_key_information),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_get_and_export_wrapping_key_information_pairing",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.18-005",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.18-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_get_or_export_wrapping_specification_has_key_information),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_get_and_export_wrapping_key_information_pairing",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.40-014",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.40-014",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_query_contains_one_or_more_functions),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_at_least_one_client_query_function",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.40-016",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.40-016",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_repeat_query_functions),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_query_function_repetition_as_optional_capability",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.14-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/unit/derive_key_operation_tests.rs::request_encodes_required_table_205_fields_in_order_and_repeats_identifiers",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "derive_key_source_usage_mask_caller_asserted",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.14-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/unit/derive_key_operation_tests.rs::request_encodes_required_table_205_fields_in_order_and_repeats_identifiers",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "derive_key_multiple_inputs_caller_selected",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.14-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(derive_key_specifies_cryptographic_length),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_derive_key_cryptographic_length",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.14-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(derive_key_specifies_cryptographic_algorithm_for_key_output),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_derive_key_cryptographic_algorithm_for_key_results",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-007-001",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.14-ATTRIBUTES-REQUIRED"),
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(derive_key_includes_required_attributes),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_derive_key_attributes",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-007-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.14-007",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(derive_key_symmetric_output_specifies_cryptographic_attributes),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_derive_key_symmetric_output_attributes",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-009",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.14-OBJECT-TYPE-REQUIRED"),
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(derive_key_includes_required_object_type),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_derive_key_object_type",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-010",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.14-UNIQUE-IDENTIFIER-REQUIRED"),
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(derive_key_specifies_explicit_source_identifiers),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_derive_key_unique_identifier",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-011",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.14-DERIVATION-METHOD-REQUIRED"),
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(derive_key_includes_required_derivation_method),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_derive_key_derivation_method",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-012",
        source_clause_id: "",
        source_gap_id: Some("KMIPKIT-SOURCE-GAP-SPEC-6.1.14-DERIVATION-PARAMETERS-REQUIRED"),
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(derive_key_includes_required_derivation_parameters),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_derive_key_parameters",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-001-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.14-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(derive_key_secret_data_uses_total_cryptographic_length),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_optional_derive_key_secret_data_total_length",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-006-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.14-006",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(derive_key_repeated_unique_identifiers_are_usable),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_repeated_derive_key_unique_identifiers",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.14-006-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.14-006",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(derive_key_specifies_explicit_source_identifiers),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_derive_key_unique_identifiers_exclude_id_placeholder",
            kind: RuleTestKind::NegativeBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.12-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.12-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(derive_key_hash_method_selects_explicit_prf),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_derive_key_prf_parameters_with_source_condition",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-7.12-005-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-7.12-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(derive_key_optional_input_forms_are_representable),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_derive_key_input_forms",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_table_145_unique_identifier_encodings_for_derive_key",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/derive_key_operation_tests.rs::both_derivation_input_forms_are_sent_and_server_error_is_not_rejected_locally",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-8-003-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-8-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(client_can_signal_asynchronous_response_support),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_asynchronous_indicator_capability",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/option_tests.rs::outbound_asynchronous_indicator_preserves_each_assigned_value",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-10.1.2-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-10.1.2-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::structure_preserves_child_order_and_repeated_tags",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_a_device_identifier_field_in_device_credentials",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "encoding",
            value: "ttlv",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-10.1.2-002-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-10.1.2-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::encodes_big_integer_golden_vector_with_sign_extension_in_item_length",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::big_integer_padding_is_minimal_and_sign_extended",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "encoding",
            value: "ttlv",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-10.1.2-002-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-10.1.2-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::encodes_big_integer_golden_vector_with_sign_extension_in_item_length",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "encoding",
            value: "ttlv",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-10.1.5-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-10.1.5-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::encodes_byte_string_golden_vector_with_minimum_following_padding",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::encodes_text_string_golden_vector_with_minimum_following_padding",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::text_and_byte_string_padding_is_minimal_at_eight_byte_boundaries",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "encoding",
            value: "ttlv",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-10.1.5-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-10.1.5-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::fixed_width_four_byte_values_have_four_following_padding_bytes",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "encoding",
            value: "ttlv",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-10.4-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-10.4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_sends_exact_bytes_over_tls13_mtls_and_closes_after_one_frame",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-transport/tests/https.rs::https_default_target_posts_exact_bytes_with_required_headers_and_zeroizes_on_success",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "transport_security",
            value: "tls13",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-10.4-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-10.4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-transport/tests/tls_integrity.rs::raw_tls_rejects_a_tampered_post_handshake_application_record",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-transport/tests/tls_integrity.rs::https_invalidates_a_connection_after_a_tampered_post_handshake_application_record",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "transport_security",
            value: "tls13",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-10.4-001-003",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-10.4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_sends_exact_bytes_over_tls13_mtls_and_closes_after_one_frame",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_unknown_server_ca",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_expired_server_certificate",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_server_name_mismatch",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "transport_security",
            value: "tls13",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-11.56-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-11.56-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_request_tags_use_kmip_prefixes),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_standard_and_extension_tag_prefixes_for_outbound_requests",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-11-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-11-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: Some(client_request_tags_are_allocated),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-ttlv/tests/tag_allocation.rs::individually_and_residually_reserved_tags_are_rejected",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-ttlv/tests/codec_negative.rs::public_decoder_rejects_received_reserved_tags",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-12-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-12-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_mask_values_use_ttlv_integer_encoding),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_ttlv_mask_integer_values",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_non_integer_ttlv_mask_values",
                kind: RuleTestKind::NegativeBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-4.1.2-001",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-4.1.2-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::baseline_profile_conformance_gates_are_explicit_and_do_not_restrict_requests",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_conformance_evidence",
            value: "profiles_4_1_2_test_variations",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-6-001",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-6-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::baseline_profile_conformance_gates_are_explicit_and_do_not_restrict_requests",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_conformance_evidence",
            value: "all_applicable_profile_normative_statements",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-6.1-001",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-6.1-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::baseline_profile_conformance_gates_are_explicit_and_do_not_restrict_requests",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_conformance_evidence",
            value: "kmip_specification_v2_1",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-6.1-002",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-6.1-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::baseline_profile_conformance_gates_are_explicit_and_do_not_restrict_requests",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_conformance_evidence",
            value: "profiles_3_1_basic_authentication_suite",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-6.1-003",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-6.1-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::baseline_profile_conformance_gates_are_explicit_and_do_not_restrict_requests",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_conformance_evidence",
            value: "baseline_client_conditions_5_1_1",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-6.1-004",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-6.1-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::baseline_profile_conformance_gates_are_explicit_and_do_not_restrict_requests",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_conformance_evidence",
            value: "one_or_more_baseline_mandatory_tests_5_1_3_or_5_6_3",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-client/tests/unit/encrypt_decrypt_fixture_execution_tests.rs::fixture_derived_encrypt_decrypt_pairs_execute_once_with_exact_wire_and_paired_response",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "implemented_operations",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: Some(client_understands_response_operations),
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::client_response_operation_matches_the_operation_it_was_sent_for",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::client_response_operation_matches_the_operation_it_was_sent_for",
                kind: RuleTestKind::NegativeBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/encrypt_decrypt_fixture_execution_tests.rs::fixture_derived_encrypt_decrypt_pairs_execute_once_with_exact_wire_and_paired_response",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1-003-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1-003",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/register_execution_tests.rs::register_produces_id_placeholder_for_a_later_identifier_omitting_operation",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/encrypt_decrypt_id_placeholder_execution_tests.rs::later_encrypt_omits_identifier_with_batch_order_enabled",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "id_placeholder",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1-004",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_tests.rs::request_keeps_the_required_empty_attributes_structure_without_synthesizing_fields",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/unit/create_tests.rs::create_preserves_two_repeated_object_group_text_values_in_order",
                kind: RuleTestKind::OptionalCapabilityUsable,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "zero_or_more_create_attributes",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1-005",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1-005",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[
            RuleTestEvidence {
                test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::recover_before_use_remains_explicit_caller_controlled_behavior",
                kind: RuleTestKind::PositiveBehavior,
            },
            RuleTestEvidence {
                test_id: "crates/kmipkit-client/tests/unit/recover_execution_tests.rs::caller_executes_encrypt_only_after_completed_recover_using_the_returned_identifier",
                kind: RuleTestKind::PositiveBehavior,
            },
        ],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "recover_before_use_caller_controlled",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.28-004-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.28-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-client/tests/unit/object_read_execution_tests.rs::caller_directs_archived_locate_recover_then_get_as_three_exchanges",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "archived_locate_recover_get_workflow_caller_directed",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.42-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.42-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-client/tests/unit/recover_execution_tests.rs::caller_issues_poll_after_recover_pending_with_the_returned_correlation",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "recover_poll_caller_directed",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.42-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.42-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-client/tests/unit/object_read_execution_tests.rs::caller_directs_archived_locate_recover_then_get_as_three_exchanges",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "recover_get_caller_directed",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-14.1-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-14.1-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::selection_resolves_kmip_client_conformance_only_for_a_complete_profile_closure",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_selection",
            value: "at_least_one",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-14.1-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-14.1-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::selection_resolves_kmip_client_conformance_only_for_a_complete_profile_closure",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_closure",
            value: "all_normative_statements",
        }],
    },
];

/// Redacted validation failure containing only stable IDs and a fixed code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileValidationError {
    code: ProfileValidationErrorCode,
    profile_id: Option<Box<str>>,
    requirement_id: Option<Box<str>>,
    source_clause_id: Option<&'static str>,
    source_gap_id: Option<&'static str>,
    source_gap_locator: Option<&'static str>,
    batch_item_index: Option<usize>,
}

fn catalogued_source_gap_locator(source_gap_id: &str) -> Option<&'static str> {
    CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.source_gap_id == Some(source_gap_id))
        .and_then(|requirement| requirement.source_locator)
}

impl ProfileValidationError {
    fn new(code: ProfileValidationErrorCode) -> Self {
        Self {
            code,
            profile_id: None,
            requirement_id: None,
            source_clause_id: None,
            source_gap_id: None,
            source_gap_locator: None,
            batch_item_index: None,
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
            profile_id: Some(profile_id.into()),
            requirement_id: None,
            source_clause_id: None,
            source_gap_id: None,
            source_gap_locator: None,
            batch_item_index: None,
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
            profile_id: Some(profile_id.into()),
            requirement_id: Some(requirement_id.into()),
            source_clause_id,
            source_gap_id: None,
            source_gap_locator: None,
            batch_item_index: None,
        }
    }

    fn for_rule_requirement(
        code: ProfileValidationErrorCode,
        profile_id: &str,
        requirement_id: &str,
        source_clause_id: &'static str,
        source_gap_id: Option<&'static str>,
    ) -> Self {
        let (source_clause_id, source_gap_id) = match (source_clause_id, source_gap_id) {
            (source_clause_id, None) if !source_clause_id.is_empty() => {
                (Some(source_clause_id), None)
            }
            ("", Some(source_gap_id)) => (None, Some(source_gap_id)),
            _ => (None, None),
        };
        let mut error = Self::for_requirement(code, profile_id, requirement_id, source_clause_id);
        error.source_gap_id = source_gap_id;
        error.source_gap_locator = source_gap_id.and_then(catalogued_source_gap_locator);
        error
    }

    fn with_batch_item_index(mut self, index: usize) -> Self {
        self.batch_item_index = Some(index);
        self
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

    /// Returns the exact source gap for a normative table-body requirement.
    #[must_use]
    pub const fn source_gap_id(&self) -> Option<&'static str> {
        self.source_gap_id
    }

    /// Returns the exact catalogued source locator for a normative table-body row.
    #[must_use]
    pub const fn source_gap_locator(&self) -> Option<&'static str> {
        self.source_gap_locator
    }

    /// Returns the zero-based request batch item for a contextual failure.
    #[must_use]
    pub const fn batch_item_index(&self) -> Option<usize> {
        self.batch_item_index
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
        } else if let Some(source_gap_id) = self.source_gap_id {
            write!(formatter, "; source gap {source_gap_id}")?;
            if let Some(source_gap_locator) = self.source_gap_locator {
                write!(formatter, " at {source_gap_locator}")?;
            }
        }
        if let Some(batch_item_index) = self.batch_item_index {
            write!(formatter, "; batch item {batch_item_index}")?;
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
    let mut constraints = std::collections::BTreeMap::<&'static str, &'static str>::new();
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
                merge_rule_constraint(
                    &mut constraints,
                    profile_id,
                    requirement,
                    mapping,
                    constraint,
                )?;
            }
            requirement_ids.insert(requirement.requirement_id);
        }
    }

    Ok(ProfileValidationResult {
        profile_ids: resolved_ids.into_iter().collect(),
        requirement_ids: requirement_ids.into_iter().map(str::to_owned).collect(),
    })
}

fn merge_rule_constraint(
    constraints: &mut std::collections::BTreeMap<&'static str, &'static str>,
    profile_id: &str,
    requirement: &GeneratedClientRequirement,
    mapping: &TypedRuleMapping,
    constraint: &RuleConstraint,
) -> Result<(), ProfileValidationError> {
    if let Some(existing_value) = constraints.get(constraint.key) {
        if !rule_constraint_values_are_compatible(constraint.key, existing_value, constraint.value)
        {
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
    Ok(())
}

fn rule_constraint_values_are_compatible(key: &str, left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }

    match key {
        // These constraints are independent capability/evidence labels, not
        // assignments to one mutually exclusive runtime setting.
        "client_profile_capability" | "client_profile_conformance_evidence" => true,
        // TLS 1.3 satisfies the broader TLS requirement; both profile clauses
        // can apply to the same selected client.
        "transport_security" => matches!((left, right), ("tls", "tls13") | ("tls13", "tls")),
        _ => false,
    }
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
        let (source_clause_id, source_gap_id) = if let Some(source_gap_id) = requirement
            .source_gap_id
            .filter(|_| requirement.source_locator.is_some())
        {
            ("", Some(source_gap_id))
        } else {
            let source_clause_id = if requirement
                .source_clause_ids
                .contains(&mapping.source_clause_id)
            {
                mapping.source_clause_id
            } else {
                requirement.source_clause_ids.first().copied().unwrap_or("")
            };
            (source_clause_id, None)
        };
        ProfileValidationError::for_rule_requirement(
            code,
            profile_id,
            requirement.requirement_id,
            source_clause_id,
            source_gap_id,
        )
    };
    let source_matches = match (mapping.source_clause_id, mapping.source_gap_id) {
        (source_clause_id, None) if !source_clause_id.is_empty() => {
            requirement.source_clause_ids.contains(&source_clause_id)
        }
        ("", Some(source_gap_id)) => {
            requirement.source_gap_id == Some(source_gap_id) && requirement.source_locator.is_some()
        }
        _ => false,
    };
    if !source_matches || mapping.verification_category != requirement.verification_category {
        return Err(error(ProfileValidationErrorCode::RuleMappingMismatch));
    }
    if mapping
        .constraints
        .iter()
        .any(|constraint| !is_supported_rule_constraint(constraint))
    {
        return Err(error(ProfileValidationErrorCode::RuleMappingMismatch));
    }
    if is_contextual_rule_mapping(mapping.requirement_id) {
        // The context-aware evaluator is maintained separately so ordinary
        // message-only evaluators cannot accidentally invent remote state.
    } else if mapping.evaluator.is_none() && !is_configuration_only_mapping(mapping) {
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

fn is_configuration_only_mapping(mapping: &TypedRuleMapping) -> bool {
    mapping.evaluator.is_none()
        && !mapping.constraints.is_empty()
        && mapping
            .constraints
            .iter()
            .all(is_configuration_only_rule_constraint)
}

fn is_caller_owned_temporal_constraint(constraint: &RuleConstraint) -> bool {
    matches!(
        (constraint.key, constraint.value),
        (
            "client_profile_capability",
            "activation_date_reached_before_use_caller_asserted"
                | "no_cryptographic_use_before_activation_date_caller_asserted"
                | "activation_date_immutable_after_pre_active_until_destruction_caller_asserted"
                | "deactivation_date_exceptional_use_requires_special_permission_caller_asserted"
                | "deactivation_date_change_only_in_pre_active_or_active_caller_asserted"
                | "process_start_date_use_after_date_caller_asserted"
                | "no_cryptographic_use_before_process_start_date_caller_asserted"
                | "process_start_date_may_equal_or_follow_activation_date_caller_asserted"
                | "process_start_date_must_not_precede_activation_date_caller_asserted"
                | "process_start_date_immutable_after_occurrence_caller_asserted"
                | "no_cryptographic_protection_after_protect_stop_date_caller_asserted"
                | "protect_stop_date_may_equal_or_precede_deactivation_date_caller_asserted"
                | "protect_stop_date_must_not_follow_deactivation_date_caller_asserted"
                | "protect_stop_date_immutable_after_occurrence_caller_asserted"
                | "countdown_timer_relative_dates_from_server_timestamp_caller_asserted"
        )
    )
}

fn is_supported_rule_constraint(constraint: &RuleConstraint) -> bool {
    let is_known_constraint = matches!(
        (constraint.key, constraint.value),
        ("transport", "raw_tls" | "https")
            | ("transport_security", "tls" | "tls13")
            | ("tls13_cipher_suites", "kmip_profiles_3_1_2")
            | ("encoding", "ttlv")
            | ("client_profile_selection", "at_least_one")
            | ("client_profile_closure", "all_normative_statements")
            | (
                "client_profile_conformance_evidence",
                "profiles_4_1_2_test_variations"
                    | "all_applicable_profile_normative_statements"
                    | "kmip_specification_v2_1"
                    | "profiles_3_1_basic_authentication_suite"
                    | "baseline_client_conditions_5_1_1"
                    | "one_or_more_baseline_mandatory_tests_5_1_3_or_5_6_3"
            )
            | (
                "client_profile_capability",
                "implemented_operations"
                    | "id_placeholder"
                    | "zero_or_more_create_attributes"
                    | "recover_before_use_caller_controlled"
                    | "archived_locate_recover_get_workflow_caller_directed"
                    | "recover_poll_caller_directed"
                    | "recover_get_caller_directed"
                    | "hashed_password_sha256_default"
                    | "hashed_password_timestamp_monotonicity_per_username"
                    | "credential_authentication_representation"
                    | "declared_response_capacity"
                    | "device_identifiers_caller_asserted_unique"
                    | "name_unique_within_target_server_caller_asserted"
                    | "caller_supplies_distinct_values_for_multi_instance_attributes"
                    | "derive_key_source_usage_mask_caller_asserted"
                    | "target_usage_mask_allows_requested_function_caller_asserted"
                    | "target_usage_mask_excludes_requested_function_caller_asserted"
                    | "lease_not_expired_or_renewed_caller_asserted"
                    | "usage_allocation_obtained_before_protection_caller_asserted"
                    | "usage_allocation_requested_within_object_date_window_caller_asserted"
                    | "usage_allocation_remains_available_caller_asserted"
                    | "lease_not_expired_caller_asserted"
                    | "recertify_once_per_existing_certificate_caller_asserted"
                    | "rekey_once_per_existing_key_caller_asserted"
                    | "rekey_key_pair_once_per_existing_pair_caller_asserted"
                    | "reprovision_when_credentials_are_required_caller_directed"
                    | "iv_counter_nonce_when_required_by_wrapping_method_caller_asserted"
                    | "wrapped_key_value_ttlv_when_encoding_option_absent_caller_asserted"
                    | "caller_selects_message_extension_presence_and_processing"
                    | "message_extension_fields_and_vendor_data_validated_by_message_model"
                    | "optional_failure_response_payload_and_message_extensions"
                    | "request_response_fields_follow_kmip_specified_order"
                    | "request_response_include_one_or_more_batch_items"
                    | "profile_specific_query_information_preserved"
                    | "vendor_identification_is_caller_assigned_unique_text"
                    | "single_cryptographic_usage_per_key_object_caller_asserted"
                    | "original_creation_date_immutable_after_set_caller_asserted"
                    | "compromised_object_protected_information_only_trusted_client_caller_asserted"
                    | "usage_limits_total_immutable_after_protection_begins_caller_asserted"
                    | "usage_limits_total_immutable_after_count_allocation_caller_asserted"
                    | "derive_key_multiple_inputs_caller_selected"
                    | "modify_attribute_existing_instance_named"
            )
    );
    is_known_constraint || is_caller_owned_temporal_constraint(constraint)
}

fn is_configuration_only_rule_constraint(constraint: &RuleConstraint) -> bool {
    let is_configuration_only = matches!(
        (constraint.key, constraint.value),
        ("transport_security", "tls" | "tls13")
            | ("tls13_cipher_suites", "kmip_profiles_3_1_2")
            | ("encoding", "ttlv")
            | ("client_profile_selection", "at_least_one")
            | ("client_profile_closure", "all_normative_statements")
            | (
                "client_profile_conformance_evidence",
                "profiles_4_1_2_test_variations"
                    | "all_applicable_profile_normative_statements"
                    | "kmip_specification_v2_1"
                    | "profiles_3_1_basic_authentication_suite"
                    | "baseline_client_conditions_5_1_1"
                    | "one_or_more_baseline_mandatory_tests_5_1_3_or_5_6_3"
            )
            | (
                "client_profile_capability",
                "implemented_operations"
                    | "id_placeholder"
                    | "zero_or_more_create_attributes"
                    | "recover_before_use_caller_controlled"
                    | "archived_locate_recover_get_workflow_caller_directed"
                    | "recover_poll_caller_directed"
                    | "recover_get_caller_directed"
                    | "hashed_password_sha256_default"
                    | "hashed_password_timestamp_monotonicity_per_username"
                    | "credential_authentication_representation"
                    | "declared_response_capacity"
                    | "device_identifiers_caller_asserted_unique"
                    | "name_unique_within_target_server_caller_asserted"
                    | "caller_supplies_distinct_values_for_multi_instance_attributes"
                    | "derive_key_source_usage_mask_caller_asserted"
                    | "target_usage_mask_allows_requested_function_caller_asserted"
                    | "target_usage_mask_excludes_requested_function_caller_asserted"
                    | "lease_not_expired_or_renewed_caller_asserted"
                    | "usage_allocation_obtained_before_protection_caller_asserted"
                    | "usage_allocation_requested_within_object_date_window_caller_asserted"
                    | "usage_allocation_remains_available_caller_asserted"
                    | "lease_not_expired_caller_asserted"
                    | "recertify_once_per_existing_certificate_caller_asserted"
                    | "rekey_once_per_existing_key_caller_asserted"
                    | "rekey_key_pair_once_per_existing_pair_caller_asserted"
                    | "reprovision_when_credentials_are_required_caller_directed"
                    | "iv_counter_nonce_when_required_by_wrapping_method_caller_asserted"
                    | "wrapped_key_value_ttlv_when_encoding_option_absent_caller_asserted"
                    | "caller_selects_message_extension_presence_and_processing"
                    | "message_extension_fields_and_vendor_data_validated_by_message_model"
                    | "optional_failure_response_payload_and_message_extensions"
                    | "request_response_fields_follow_kmip_specified_order"
                    | "request_response_include_one_or_more_batch_items"
                    | "profile_specific_query_information_preserved"
                    | "vendor_identification_is_caller_assigned_unique_text"
                    | "single_cryptographic_usage_per_key_object_caller_asserted"
                    | "original_creation_date_immutable_after_set_caller_asserted"
                    | "compromised_object_protected_information_only_trusted_client_caller_asserted"
                    | "usage_limits_total_immutable_after_protection_begins_caller_asserted"
                    | "usage_limits_total_immutable_after_count_allocation_caller_asserted"
                    | "derive_key_multiple_inputs_caller_selected"
                    | "modify_attribute_existing_instance_named"
            )
    );
    is_configuration_only || is_caller_owned_temporal_constraint(constraint)
}

#[cfg(test)]
#[path = "../tests/support/profile_unit.rs"]
mod tests;
