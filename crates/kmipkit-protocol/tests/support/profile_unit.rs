use std::fmt;
use std::fmt::Write as _;

use super::{
    CLIENT_REQUIREMENTS, ProfileEvidenceRecord, ProfileReadiness, ProfileReadinessBlocker,
    ProfileTransport, ProfileValidationError, ProfileValidationErrorCode, ProfileValidationResult,
    ProfileValidationSelection, RuleConstraint, RuleTestEvidence, RuleTestKind,
    TYPED_RULE_MAPPINGS, TypedRuleMapping,
    discover_versions_advertises_only_client_supported_versions, protocol_version_is_kmip_2_1,
    reconcile_effective_state, validate_mapping, validate_selection_with_registry,
};
use crate::profiles_generated::{
    GeneratedClientProfile, GeneratedClientRequirement, GeneratedVerificationCategory,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

const REQUEST_HEADER_TAG: u32 = 0x0042_0077;
const RESPONSE_HEADER_TAG: u32 = 0x0042_007A;
const PROTOCOL_VERSION_TAG: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR_TAG: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR_TAG: u32 = 0x0042_006B;
const BATCH_COUNT_TAG: u32 = 0x0042_000D;
const BATCH_ITEM_TAG: u32 = 0x0042_000F;
const OPERATION_TAG: u32 = 0x0042_005C;
const REQUEST_PAYLOAD_TAG: u32 = 0x0042_0079;
const RESPONSE_PAYLOAD_TAG: u32 = 0x0042_007C;
const RESULT_STATUS_TAG: u32 = 0x0042_007F;
const TIME_STAMP_TAG: u32 = 0x0042_0092;

struct RejectingWriter;

impl fmt::Write for RejectingWriter {
    fn write_str(&mut self, _value: &str) -> fmt::Result {
        Err(fmt::Error)
    }
}

static CONSTRAINTS_HTTPS: [RuleConstraint; 1] = [RuleConstraint {
    key: "transport",
    value: "https",
}];
static CONSTRAINTS_RAW_TLS: [RuleConstraint; 1] = [RuleConstraint {
    key: "transport",
    value: "raw_tls",
}];
static UNKNOWN_CONSTRAINT: [RuleConstraint; 1] = [RuleConstraint {
    key: "untyped-policy",
    value: "opaque",
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

fn reject_outbound_rule(
    _request: Option<&crate::RequestMessage>,
    response: Option<(&crate::RequestMessage, &crate::ResponseMessage)>,
) -> bool {
    response.is_some()
}

fn reject_response_rule(
    _request: Option<&crate::RequestMessage>,
    response: Option<(&crate::RequestMessage, &crate::ResponseMessage)>,
) -> bool {
    response.is_none()
}

fn tagged(raw_tag: u32, value: Value) -> Item {
    let tag: Tag = RawTag::new(raw_tag)
        .expect("test tag fits the TTLV width")
        .try_checked()
        .expect("test tag is allocated");
    Item::new(tag, value).expect("test item is valid")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut result = Structure::new();
    for item in items {
        result
            .try_push(item)
            .expect("test structure depth is valid");
    }
    result
}

fn test_protocol_version() -> Item {
    tagged(
        PROTOCOL_VERSION_TAG,
        Value::structure(structure([
            tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
            tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
        ])),
    )
}

fn request_message() -> crate::RequestMessage {
    crate::RequestMessage::try_from_ttlv(structure([
        tagged(
            REQUEST_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(0x1E)),
                tagged(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
            ])),
        ),
    ]))
    .expect("test Request Message is valid")
}

fn request_message_with_operation_payload(
    operation: u32,
    payload: Structure,
) -> crate::RequestMessage {
    crate::RequestMessage::try_from_ttlv(structure([
        tagged(
            REQUEST_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(operation)),
                tagged(REQUEST_PAYLOAD_TAG, Value::structure(payload)),
            ])),
        ),
    ]))
    .expect("test Request Message with operation payload is valid")
}

fn discover_versions_payload(versions: &[(i32, i32)]) -> Structure {
    let mut payload = Structure::new();
    for &(major, minor) in versions {
        payload
            .try_push(tagged(
                PROTOCOL_VERSION_TAG,
                Value::structure(structure([
                    tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(major)),
                    tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(minor)),
                ])),
            ))
            .expect("test Discover Versions payload remains valid");
    }
    payload
}

fn response_message() -> crate::ResponseMessage {
    crate::ResponseMessage::try_from_ttlv(structure([
        tagged(
            RESPONSE_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(TIME_STAMP_TAG, Value::date_time(1)),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(0x1E)),
                tagged(RESULT_STATUS_TAG, Value::enumeration(0)),
                tagged(RESPONSE_PAYLOAD_TAG, Value::structure(Structure::new())),
            ])),
        ),
    ]))
    .expect("test Response Message is valid")
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
        unresolved_scope_obligations: &[],
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
        source_keyword: Some("SHALL"),
        source_force: None,
        source_gap_id: None,
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
fn production_discover_versions_rule_accepts_only_kmip_21_offers() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.16-001-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Discover Versions capability has a production typed rule mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("Discover Versions capability resolves to generated normative metadata");
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the production rule matches its generated clause and optional-test category");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    assert!(!discover_versions_advertises_only_client_supported_versions(None, None));

    let supported_request = request_message_with_operation_payload(
        0x0000_001E,
        crate::DiscoverVersionsRequest::new()
            .to_ttlv_payload()
            .expect("the typed request advertises KMIP 2.1"),
    );
    assert!(result.validate_request(&supported_request).is_ok());

    let omitted_optional_list =
        request_message_with_operation_payload(0x0000_001E, Structure::new());
    assert!(result.validate_request(&omitted_optional_list).is_ok());

    let other_operation = request_message_with_operation_payload(0x0000_0001, Structure::new());
    assert!(result.validate_request(&other_operation).is_ok());

    let malformed_version = request_message_with_operation_payload(
        0x0000_001E,
        structure([tagged(
            PROTOCOL_VERSION_TAG,
            Value::text_string("2.1".to_owned()),
        )]),
    );
    assert!(result.validate_request(&malformed_version).is_err());

    let unsupported_request =
        request_message_with_operation_payload(0x0000_001E, discover_versions_payload(&[(3, 0)]));
    let error = result
        .validate_request(&unsupported_request)
        .expect_err("a 1.0 client must not advertise a protocol version it does not support");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.profile_id(), Some(BASELINE_PROFILE_ID));
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-6.1.16-001")
    );

    let mixed_versions = request_message_with_operation_payload(
        0x0000_001E,
        discover_versions_payload(&[(2, 1), (3, 0)]),
    );
    assert!(result.validate_request(&mixed_versions).is_err());
}

#[test]
fn public_profile_result_entrypoints_validate_transport_and_responses() {
    // KMIP Specification v2.1 §14.1 and KMIP Profiles v2.1 §6.1; KMIPKIT-0010 FR-014.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.16-001-001";
    const PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let result = ProfileValidationResult {
        profile_ids: vec![PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };

    result
        .validate_transport(ProfileTransport::Https)
        .expect("the Discover Versions rule has no incompatible transport constraint");

    let supported_request = request_message_with_operation_payload(
        0x0000_001E,
        crate::DiscoverVersionsRequest::new()
            .to_ttlv_payload()
            .expect("the typed request advertises KMIP 2.1"),
    );
    result
        .validate_response(&supported_request, &response_message())
        .expect("a KMIP 2.1 Discover Versions response satisfies the selected rule");

    let unsupported_request =
        request_message_with_operation_payload(0x0000_001E, discover_versions_payload(&[(3, 0)]));
    let error = result
        .validate_response(&unsupported_request, &response_message())
        .expect_err("a response cannot validate a request advertising an unsupported version");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::ResponseRuleRejected
    );
    assert_eq!(error.profile_id(), Some(PROFILE_ID));
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-6.1.16-001")
    );

    let unmapped = ProfileValidationResult {
        profile_ids: vec!["TEST-PROFILE".to_owned()],
        requirement_ids: vec!["TEST-REQ".to_owned()],
    };
    let error = unmapped
        .validate_transport(ProfileTransport::Https)
        .expect_err("a selected result with a missing typed mapping fails closed");
    assert_eq!(error.code(), ProfileValidationErrorCode::MissingRuleMapping);
    assert_eq!(error.profile_id(), Some("TEST-PROFILE"));
    assert_eq!(error.requirement_id(), Some("TEST-REQ"));
    assert_eq!(error.source_clause_id(), None);
}

#[test]
fn runtime_rule_evaluation_rejects_missing_and_incomplete_mappings() {
    // KMIP Specification v2.1 §14.1 and KMIP Profiles v2.1 §6.1; KMIPKIT-0010 FR-014.
    let result = ProfileValidationResult {
        profile_ids: vec!["TEST-PROFILE".to_owned()],
        requirement_ids: vec!["TEST-REQ".to_owned()],
    };

    let missing = result
        .evaluate_rules(
            None,
            None,
            &[],
            ProfileValidationErrorCode::RequestRuleRejected,
        )
        .expect_err("runtime evaluation fails closed when a mapping is absent");
    assert_eq!(
        missing.code(),
        ProfileValidationErrorCode::MissingRuleMapping
    );
    assert_eq!(missing.source_clause_id(), None);

    let incomplete = [TypedRuleMapping {
        evaluator: None,
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &[],
        )
    }];
    let error = result
        .evaluate_rules(
            None,
            None,
            &incomplete,
            ProfileValidationErrorCode::RequestRuleRejected,
        )
        .expect_err("runtime evaluation rejects a mapping without executable code");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::IncompleteRuleMapping
    );
    assert_eq!(error.source_clause_id(), Some("TEST-CLAUSE"));
}

#[test]
fn selection_rejects_a_requirement_without_a_typed_rule_mapping() {
    // KMIP Specification v2.1 §14.1 and KMIP Profiles v2.1 §6.1; KMIPKIT-0010 FR-014.
    let profiles = [profile("TEST-PROFILE", &["TEST-REQ"], &[])];
    let requirements = [requirement(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        false,
    )];
    let error = validate_selection_with_registry(
        &ProfileValidationSelection::new(["TEST-PROFILE"]),
        &profiles,
        &requirements,
        &[],
    )
    .expect_err("selection fails closed when its applicable rule has no typed mapping");

    assert_eq!(error.code(), ProfileValidationErrorCode::MissingRuleMapping);
    assert_eq!(error.profile_id(), Some("TEST-PROFILE"));
    assert_eq!(error.requirement_id(), Some("TEST-REQ"));
    assert_eq!(error.source_clause_id(), Some("TEST-CLAUSE"));
}

#[test]
fn discover_versions_protocol_version_requires_one_integer_major_minor_pair() {
    let valid = structure([
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    assert!(protocol_version_is_kmip_2_1(&valid.view()));

    let missing_minor = structure([tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2))]);
    assert!(!protocol_version_is_kmip_2_1(&missing_minor.view()));

    let duplicate_major = structure([
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    assert!(!protocol_version_is_kmip_2_1(&duplicate_major.view()));

    let duplicate_minor = structure([
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    assert!(!protocol_version_is_kmip_2_1(&duplicate_minor.view()));

    let wrong_major_type = structure([
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::enumeration(2)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    assert!(!protocol_version_is_kmip_2_1(&wrong_major_type.view()));

    let wrong_minor_type = structure([
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::enumeration(1)),
    ]);
    assert!(!protocol_version_is_kmip_2_1(&wrong_minor_type.view()));

    let unknown_version_field = structure([
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
        tagged(PROTOCOL_VERSION_TAG, Value::integer(99)),
    ]);
    assert!(protocol_version_is_kmip_2_1(&unknown_version_field.view()));
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
        &CONSTRAINTS_HTTPS,
    )];
    let selection = ProfileValidationSelection::new(["TEST-PROFILE"]);
    assert_eq!(&["TEST-PROFILE".to_owned()], selection.profile_ids());
    assert_eq!(
        format!("{selection:?}"),
        "ProfileValidationSelection { profile_count: 1 }"
    );

    let result = validate_selection_with_registry(&selection, &profiles, &requirements, &mappings)
        .expect("complete synthetic code and verification evidence is selectable");

    assert_eq!(result.profile_ids(), &["TEST-PROFILE"]);
    assert_eq!(result.requirement_ids(), &["TEST-REQ"]);
}

#[test]
fn mapped_request_and_response_rejections_keep_stable_source_metadata() {
    let result = ProfileValidationResult {
        profile_ids: vec!["TEST-PROFILE".to_owned()],
        requirement_ids: vec!["TEST-REQ".to_owned()],
    };
    let required = requirement(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        false,
    );
    let outbound_mapping = [TypedRuleMapping {
        evaluator: Some(reject_outbound_rule),
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &[],
        )
    }];
    assert!(validate_mapping("TEST-PROFILE", &required, &outbound_mapping[0]).is_ok());
    let request = request_message();
    let error = result
        .evaluate_rules(
            Some(&request),
            None,
            &outbound_mapping,
            ProfileValidationErrorCode::RequestRuleRejected,
        )
        .expect_err("an outbound rule can reject the request before transport");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.profile_id(), Some("TEST-PROFILE"));
    assert_eq!(error.requirement_id(), Some("TEST-REQ"));
    assert_eq!(error.source_clause_id(), Some("TEST-CLAUSE"));

    let response_mapping = [TypedRuleMapping {
        evaluator: Some(reject_response_rule),
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &[],
        )
    }];
    assert!(validate_mapping("TEST-PROFILE", &required, &response_mapping[0]).is_ok());
    let error = result
        .evaluate_rules(
            Some(&request),
            Some((&request, &response_message())),
            &response_mapping,
            ProfileValidationErrorCode::ResponseRuleRejected,
        )
        .expect_err("a response rule can reject a received response");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::ResponseRuleRejected
    );
    assert_eq!(error.profile_id(), Some("TEST-PROFILE"));
    assert_eq!(error.requirement_id(), Some("TEST-REQ"));
    assert_eq!(error.source_clause_id(), Some("TEST-CLAUSE"));
}

#[test]
fn minimum_capability_rules_do_not_reject_an_extra_base_protocol_operation() {
    let result = ProfileValidationResult {
        profile_ids: vec!["TEST-PROFILE".to_owned()],
        requirement_ids: vec!["TEST-REQ".to_owned()],
    };
    let mapping = [mapping(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        RuleTestKind::PositiveBehavior,
        &[],
    )];
    let request = request_message();
    assert_eq!(
        request
            .batch_items()
            .next()
            .and_then(crate::RequestBatchItemView::operation),
        Some(0x1E),
        "the synthetic extra operation is Discover Versions"
    );

    result
        .evaluate_rules(
            Some(&request),
            None,
            &mapping,
            ProfileValidationErrorCode::RequestRuleRejected,
        )
        .expect("an operation outside a profile minimum is not an allowlist violation");
}

#[test]
fn selected_transport_constraints_match_or_reject_with_source_ids() {
    let result = ProfileValidationResult {
        profile_ids: vec!["TEST-PROFILE".to_owned()],
        requirement_ids: vec!["TEST-REQ".to_owned()],
    };
    let mappings = [mapping(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        RuleTestKind::PositiveBehavior,
        &CONSTRAINTS_HTTPS,
    )];

    result
        .validate_transport_with_mappings(ProfileTransport::Https, &mappings)
        .expect("HTTPS satisfies its selected typed constraint");
    let error = result
        .validate_transport_with_mappings(ProfileTransport::RawTtlvTls, &mappings)
        .expect_err("raw TTLV/TLS violates the selected HTTPS constraint");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::TransportConstraintRejected
    );
    assert_eq!(error.profile_id(), Some("TEST-PROFILE"));
    assert_eq!(error.requirement_id(), Some("TEST-REQ"));
    assert_eq!(error.source_clause_id(), Some("TEST-CLAUSE"));
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

    let untyped_constraint = TypedRuleMapping {
        constraints: &UNKNOWN_CONSTRAINT,
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &[],
        )
    };
    let error = validate_mapping("TEST-PROFILE", &requirement, &untyped_constraint)
        .expect_err("unknown constraint keys cannot appear executable");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RuleMappingMismatch
    );
}

#[test]
fn validation_error_codes_and_readiness_blockers_have_stable_text() {
    let validation_codes = [
        (
            ProfileValidationErrorCode::EmptySelection,
            "empty_selection",
        ),
        (
            ProfileValidationErrorCode::DuplicateProfile,
            "duplicate_profile",
        ),
        (
            ProfileValidationErrorCode::UnknownProfile,
            "unknown_profile",
        ),
        (
            ProfileValidationErrorCode::ProfileUnavailable,
            "profile_unavailable",
        ),
        (
            ProfileValidationErrorCode::ConditionalProfileUnsatisfied,
            "conditional_profile_unsatisfied",
        ),
        (
            ProfileValidationErrorCode::DependencyCycle,
            "dependency_cycle",
        ),
        (
            ProfileValidationErrorCode::MissingRequirementMetadata,
            "missing_requirement_metadata",
        ),
        (
            ProfileValidationErrorCode::UnresolvedRequirementScope,
            "unresolved_requirement_scope",
        ),
        (
            ProfileValidationErrorCode::MissingRuleMapping,
            "missing_rule_mapping",
        ),
        (
            ProfileValidationErrorCode::IncompleteRuleMapping,
            "incomplete_rule_mapping",
        ),
        (
            ProfileValidationErrorCode::RuleMappingMismatch,
            "rule_mapping_mismatch",
        ),
        (
            ProfileValidationErrorCode::MissingCategoryTest,
            "missing_category_test",
        ),
        (
            ProfileValidationErrorCode::ConflictingConstraints,
            "conflicting_constraints",
        ),
        (
            ProfileValidationErrorCode::RequestRuleRejected,
            "request_rule_rejected",
        ),
        (
            ProfileValidationErrorCode::ResponseRuleRejected,
            "response_rule_rejected",
        ),
        (
            ProfileValidationErrorCode::TransportConstraintRejected,
            "transport_constraint_rejected",
        ),
    ];
    for (code, expected) in validation_codes {
        assert_eq!(expected, code.as_str());
    }
}

#[test]
fn readiness_blockers_have_stable_text_and_requirement_ids() {
    let blockers = [
        (
            ProfileReadinessBlocker::MissingRequirementRecord {
                requirement_id: "REQ-MISSING",
            },
            "required requirement evidence is missing",
            Some("REQ-MISSING"),
        ),
        (
            ProfileReadinessBlocker::FailedRequirementRecord {
                requirement_id: "REQ-FAILED",
            },
            "required requirement evidence failed",
            Some("REQ-FAILED"),
        ),
        (
            ProfileReadinessBlocker::DuplicateRequirementRecord {
                requirement_id: "REQ-DUPLICATE",
            },
            "requirement evidence record is duplicated",
            Some("REQ-DUPLICATE"),
        ),
        (
            ProfileReadinessBlocker::UnknownRequirementRecord,
            "requirement evidence record is unknown",
            None,
        ),
        (
            ProfileReadinessBlocker::RuntimeValidationUnavailable,
            "runtime profile validation is unavailable",
            None,
        ),
        (
            ProfileReadinessBlocker::MandatoryFixtureUnavailable {
                test_id: "FIXTURE-1",
            },
            "mandatory profile fixture is unavailable",
            None,
        ),
        (
            ProfileReadinessBlocker::ConditionalProfileUnsatisfied,
            "conditional profile is unsatisfied",
            None,
        ),
        (
            ProfileReadinessBlocker::DependencyEvidenceIncomplete {
                profile_id: "PROFILE-DEPENDENCY",
            },
            "dependency profile evidence is incomplete",
            None,
        ),
        (
            ProfileReadinessBlocker::VerificationManifestUnavailable,
            "current verification manifest is unavailable",
            None,
        ),
        (
            ProfileReadinessBlocker::StaleCatalogEvidenceState,
            "catalog evidence-complete state is stale",
            None,
        ),
        (
            ProfileReadinessBlocker::CatalogEvidenceReviewRequired,
            "catalog evidence state requires human review",
            None,
        ),
        (
            ProfileReadinessBlocker::ProfileUnavailable,
            "profile is outside the KMIPKit 1.0 client scope",
            None,
        ),
    ];
    for (blocker, expected, requirement_id) in blockers {
        assert_eq!(expected, blocker.as_str());
        assert_eq!(requirement_id, blocker.requirement_id());
        assert_eq!(None, blocker.source_clause_id());
    }
    let unresolved = ProfileReadinessBlocker::UnresolvedRequirementScope {
        requirement_id: "REQ-SCOPE-UNCLEAR",
        source_clause_id: "CLAUSE-SCOPE-UNCLEAR",
    };
    assert_eq!(
        "profile requirement applicability is unresolved",
        unresolved.as_str()
    );
    assert_eq!(Some("REQ-SCOPE-UNCLEAR"), unresolved.requirement_id());
    assert_eq!(Some("CLAUSE-SCOPE-UNCLEAR"), unresolved.source_clause_id());
}

#[test]
fn validation_error_display_contains_only_stable_identifiers() {
    let code_only = ProfileValidationError::new(ProfileValidationErrorCode::EmptySelection);
    assert_eq!(
        "profile validation failed: empty_selection",
        code_only.to_string()
    );
    assert_eq!(None, code_only.profile_id());
    assert_eq!(None, code_only.requirement_id());
    assert_eq!(None, code_only.source_clause_id());

    let profile = ProfileValidationError::for_profile(
        ProfileValidationErrorCode::UnknownProfile,
        "TEST-PROFILE",
    );
    assert_eq!(Some("TEST-PROFILE"), profile.profile_id());
    assert_eq!(
        "profile validation failed: unknown_profile; profile TEST-PROFILE",
        profile.to_string()
    );

    let requirement = ProfileValidationError::for_requirement(
        ProfileValidationErrorCode::MissingRuleMapping,
        "TEST-PROFILE",
        "TEST-REQ",
        Some("TEST-CLAUSE"),
    );
    assert_eq!(Some("TEST-REQ"), requirement.requirement_id());
    assert_eq!(Some("TEST-CLAUSE"), requirement.source_clause_id());
    assert_eq!(
        "profile validation failed: missing_rule_mapping; profile TEST-PROFILE; requirement TEST-REQ; source clause TEST-CLAUSE",
        requirement.to_string()
    );

    assert!(write!(&mut RejectingWriter, "{code_only}").is_err());
}

#[test]
fn readiness_records_classify_duplicates_and_unknown_ids() {
    let baseline = super::CLIENT_PROFILES
        .iter()
        .find(|profile| profile.profile_id == "KMIPKIT-PROFILE-BASELINE-CLIENT")
        .expect("generated catalog contains Baseline Client");
    let first = baseline.requirement_ids[0];
    let records: Vec<_> = baseline
        .requirement_ids
        .iter()
        .map(|requirement_id| ProfileEvidenceRecord::passed(*requirement_id))
        .chain([ProfileEvidenceRecord::passed(first)])
        .chain([ProfileEvidenceRecord::passed("UNKNOWN-REQ")])
        .collect();

    let readiness = ProfileReadiness::evaluate(baseline.profile_id, &records)
        .expect("generated Baseline Client resolves");

    assert!(!readiness.requirement_records_complete());
    assert!(
        readiness
            .blockers()
            .contains(&ProfileReadinessBlocker::DuplicateRequirementRecord {
                requirement_id: first
            })
    );
    assert!(
        readiness
            .blockers()
            .contains(&ProfileReadinessBlocker::UnknownRequirementRecord)
    );
    assert!(readiness.target_selected());
    assert!(!readiness.runtime_validation_available());
    assert!(
        readiness
            .blockers()
            .contains(&ProfileReadinessBlocker::RuntimeValidationUnavailable)
    );
    assert_eq!(baseline.profile_id, readiness.profile_id());
    assert_eq!("evidence_incomplete", readiness.effective_state());
    assert!(!readiness.published_claim());
    assert!(ProfileReadiness::evaluate("UNKNOWN-PROFILE", &[]).is_none());

    let failed =
        ProfileReadiness::evaluate(baseline.profile_id, &[ProfileEvidenceRecord::failed(first)])
            .expect("generated Baseline Client resolves");
    assert!(
        failed
            .blockers()
            .contains(&ProfileReadinessBlocker::FailedRequirementRecord {
                requirement_id: first
            })
    );
}

#[test]
fn readiness_blocks_conditional_out_of_scope_and_dependent_profiles() {
    let conditional = ProfileReadiness::evaluate("KMIPKIT-PROFILE-HTTPS-CLIENT", &[])
        .expect("generated HTTPS Client resolves");
    assert!(
        conditional
            .blockers()
            .contains(&ProfileReadinessBlocker::ConditionalProfileUnsatisfied)
    );

    let unavailable = ProfileReadiness::evaluate("KMIPKIT-PROFILE-JSON-CLIENT", &[])
        .expect("generated JSON Client resolves");
    assert!(
        unavailable
            .blockers()
            .contains(&ProfileReadinessBlocker::ProfileUnavailable)
    );

    let dependent = ProfileReadiness::evaluate("KMIPKIT-PROFILE-AES-XTS-CLIENT", &[])
        .expect("generated AES-XTS Client resolves");
    assert!(dependent.blockers().contains(
        &ProfileReadinessBlocker::DependencyEvidenceIncomplete {
            profile_id: "KMIPKIT-PROFILE-BASELINE-CLIENT"
        }
    ));
}

#[test]
fn readiness_reconciles_catalog_states_with_blockers() {
    let mut stale = vec![ProfileReadinessBlocker::RuntimeValidationUnavailable];
    assert_eq!(
        "evidence_incomplete",
        reconcile_effective_state("evidence_complete", &mut stale)
    );
    assert!(stale.contains(&ProfileReadinessBlocker::StaleCatalogEvidenceState));

    let mut review_required = Vec::new();
    assert_eq!(
        "not_claimed",
        reconcile_effective_state("not_claimed", &mut review_required)
    );
    assert_eq!(
        vec![ProfileReadinessBlocker::CatalogEvidenceReviewRequired],
        review_required
    );

    let repeated = ProfileReadinessBlocker::VerificationManifestUnavailable;
    let mut unchanged_with_duplicates = vec![repeated, repeated];
    assert_eq!(
        "evidence_incomplete",
        reconcile_effective_state("evidence_incomplete", &mut unchanged_with_duplicates)
    );
    assert_eq!(vec![repeated], unchanged_with_duplicates);
}

#[test]
fn selection_rejects_missing_catalog_metadata_and_skips_resolved_dependencies() {
    let empty = validate_selection_with_registry(
        &ProfileValidationSelection::new(std::iter::empty::<&str>()),
        &[],
        &[],
        &[],
    )
    .expect_err("an empty explicit profile selection is invalid");
    assert_eq!(ProfileValidationErrorCode::EmptySelection, empty.code());

    let unknown_duplicate = validate_selection_with_registry(
        &ProfileValidationSelection::new(["UNKNOWN", "UNKNOWN"]),
        &[],
        &[],
        &[],
    )
    .expect_err("duplicate unknown identifiers are still duplicates");
    assert_eq!(
        ProfileValidationErrorCode::DuplicateProfile,
        unknown_duplicate.code()
    );
    assert_eq!(None, unknown_duplicate.profile_id());

    let known_duplicate = validate_selection_with_registry(
        &ProfileValidationSelection::new(["TEST-PROFILE", "TEST-PROFILE"]),
        &[profile("TEST-PROFILE", &[], &[])],
        &[],
        &[],
    )
    .expect_err("a known profile cannot be selected twice");
    assert_eq!(
        ProfileValidationErrorCode::DuplicateProfile,
        known_duplicate.code()
    );
    assert_eq!(Some("TEST-PROFILE"), known_duplicate.profile_id());

    let unknown = validate_selection_with_registry(
        &ProfileValidationSelection::new(["UNKNOWN"]),
        &[],
        &[],
        &[],
    )
    .expect_err("unknown profiles do not resolve");
    assert_eq!(ProfileValidationErrorCode::UnknownProfile, unknown.code());

    let empty_requirements = [profile("EMPTY", &[], &[])];
    let missing_list = validate_selection_with_registry(
        &ProfileValidationSelection::new(["EMPTY"]),
        &empty_requirements,
        &[],
        &[],
    )
    .expect_err("a selected profile must list applicable requirements");
    assert_eq!(
        ProfileValidationErrorCode::MissingRequirementMetadata,
        missing_list.code()
    );

    let missing_record_profiles = [profile("MISSING-RECORD", &["TEST-REQ"], &[])];
    let missing_record = validate_selection_with_registry(
        &ProfileValidationSelection::new(["MISSING-RECORD"]),
        &missing_record_profiles,
        &[],
        &[],
    )
    .expect_err("every profile requirement must resolve to metadata");
    assert_eq!(
        ProfileValidationErrorCode::MissingRequirementMetadata,
        missing_record.code()
    );

    let shared_dependency_profiles = [
        profile("SHARED", &[], &[]),
        profile("ROOT-A", &[], &["SHARED"]),
        profile("ROOT-B", &[], &["SHARED"]),
    ];
    let dependency_error = validate_selection_with_registry(
        &ProfileValidationSelection::new(["ROOT-A", "ROOT-B"]),
        &shared_dependency_profiles,
        &[],
        &[],
    )
    .expect_err("empty requirement metadata remains fail-closed after closure resolution");
    assert_eq!(
        ProfileValidationErrorCode::MissingRequirementMetadata,
        dependency_error.code()
    );
}

#[test]
fn passing_evaluator_is_safe_for_absent_and_present_exchange_contexts() {
    assert!(passing_rule(None, None));
}
