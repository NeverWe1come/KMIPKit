//! Profile evidence remains incomplete when any required record is absent or failed.
//!
//! This local Rust view never substitutes for the current protected-release-push
//! evidence manifest validated by the repository readiness tooling.

use kmipkit_protocol::{
    DiscoverVersionsResponse, ProfileEvidenceReadiness, ProfileEvidenceRecord,
    QueryAsyncRequestsResponse, ResponseMessage, profiles_generated::CLIENT_PROFILES,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

const BASELINE: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const ASYNCHRONOUS_CORRELATION_VALUES: u32 = 0x0042_0176;
const QUERY_ASYNCHRONOUS_REQUESTS: u32 = 0x0000_0039;
const DISCOVER_VERSIONS: u32 = 0x0000_001E;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits the 24-bit KMIP field")
        .try_checked()
        .expect("fixture tag is allocated by the KMIP 2.1 catalog")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("fixture item uses a checked tag")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut result = Structure::new();
    for child in items {
        result
            .try_push(child)
            .expect("fixture structure stays within the model limit");
    }
    result
}

fn response(operation: u32, payload: Option<Structure>) -> ResponseMessage {
    let header = structure([
        item(
            PROTOCOL_VERSION,
            Value::structure(structure([
                item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
                item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
            ])),
        ),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut batch_item = vec![
        item(OPERATION, Value::enumeration(operation)),
        item(RESULT_STATUS, Value::enumeration(0)),
    ];
    if let Some(payload) = payload {
        batch_item.push(item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    ResponseMessage::try_from_ttlv(structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(structure(batch_item))),
    ]))
    .expect("fixture is a valid KMIP response message")
}

fn baseline_requirements() -> &'static [&'static str] {
    CLIENT_PROFILES
        .iter()
        .find(|profile| profile.profile_id == BASELINE)
        .expect("generated metadata includes Baseline Client")
        .requirement_ids
}

fn passing_records() -> Vec<ProfileEvidenceRecord> {
    baseline_requirements()
        .iter()
        .map(|requirement_id| ProfileEvidenceRecord::passed(*requirement_id))
        .collect()
}

#[test]
fn a_missing_or_failed_required_record_prevents_evidence_completion() {
    let requirements = baseline_requirements();
    let passing = passing_records();
    let missing = ProfileEvidenceReadiness::evaluate(BASELINE, &passing[..passing.len() - 1])
        .expect("Baseline profile resolves");
    assert!(!missing.requirement_records_complete());
    assert_ne!(missing.effective_state(), "evidence_complete");

    let mut failed_records = passing;
    let failed_requirement = requirements[requirements.len() - 1];
    failed_records.pop();
    failed_records.push(ProfileEvidenceRecord::failed(failed_requirement));
    let failed = ProfileEvidenceReadiness::evaluate(BASELINE, &failed_records)
        .expect("Baseline profile resolves");
    assert!(!failed.requirement_records_complete());
    assert_ne!(failed.effective_state(), "evidence_complete");
}

#[test]
fn dependent_profile_readiness_includes_transitive_dependency_requirements() {
    let profiles = kmipkit_protocol::profiles_generated::CLIENT_PROFILES;
    let dependent = profiles
        .iter()
        .find(|profile| profile.profile_id == "KMIPKIT-PROFILE-AES-XTS-CLIENT")
        .expect("AES-XTS Client resolves");
    let dependency = profiles
        .iter()
        .find(|profile| profile.profile_id == BASELINE)
        .expect("Baseline Client resolves");
    let records: Vec<_> = dependent
        .requirement_ids
        .iter()
        .map(|requirement_id| ProfileEvidenceRecord::passed(*requirement_id))
        .collect();

    let readiness = ProfileEvidenceReadiness::evaluate(dependent.profile_id, &records)
        .expect("AES-XTS Client resolves");
    let missing_dependency_requirements: Vec<_> = dependency
        .requirement_ids
        .iter()
        .filter(|requirement_id| !dependent.requirement_ids.contains(requirement_id))
        .collect();

    assert_ne!(missing_dependency_requirements.len(), 0);
    assert!(!readiness.requirement_records_complete());
    for requirement_id in missing_dependency_requirements {
        assert!(readiness.blockers().iter().any(|blocker| {
            blocker.requirement_id() == Some(*requirement_id)
                && blocker.as_str() == "required requirement evidence is missing"
        }));
    }
}

#[test]
fn local_records_never_advance_catalog_state_or_publish_a_claim() {
    let readiness = ProfileEvidenceReadiness::evaluate(BASELINE, &passing_records())
        .expect("Baseline profile resolves");

    assert!(readiness.requirement_records_complete());
    assert_eq!("evidence_incomplete", readiness.effective_state());
    assert!(
        readiness
            .blockers()
            .iter()
            .any(|blocker| { blocker.as_str() == "current verification manifest is unavailable" })
    );
    assert!(!readiness.published_claim());
}

#[test]
fn query_and_discover_versions_responses_do_not_change_profile_readiness() {
    let records = passing_records();
    let before =
        ProfileEvidenceReadiness::evaluate(BASELINE, &records).expect("Baseline profile resolves");

    let query_payload = structure([item(
        ASYNCHRONOUS_CORRELATION_VALUES,
        Value::structure(Structure::new()),
    )]);
    let query_message = response(QUERY_ASYNCHRONOUS_REQUESTS, Some(query_payload));
    let query = QueryAsyncRequestsResponse::try_from_response_item(
        query_message
            .batch_items()
            .next()
            .expect("fixture has one response batch item"),
    )
    .expect("fixture is a Query Asynchronous Requests response");
    assert_eq!(0, query.result().status().raw());

    let discover_payload = structure([item(
        PROTOCOL_VERSION,
        Value::structure(structure([
            item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
            item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
        ])),
    )]);
    let discover_message = response(DISCOVER_VERSIONS, Some(discover_payload));
    let discover = DiscoverVersionsResponse::try_from_response_item(
        discover_message
            .batch_items()
            .next()
            .expect("fixture has one response batch item"),
    )
    .expect("fixture is a Discover Versions response");
    assert_eq!(
        Some(&[kmipkit_protocol::ProtocolVersion::from_raw(2, 1)][..]),
        discover.supported_versions()
    );

    let after =
        ProfileEvidenceReadiness::evaluate(BASELINE, &records).expect("Baseline profile resolves");
    assert_eq!(before, after);
    assert!(!after.published_claim());
}
