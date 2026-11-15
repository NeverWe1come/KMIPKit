use kmipkit_protocol::{ProfileEvidenceReadiness, ProfileEvidenceRecord, ProfileReadinessBlocker};

const BASELINE: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
const UNKNOWN_MESSAGE: &str = "requirement evidence record is unknown";

#[test]
fn empty_and_whitespace_unknown_requirement_ids_use_the_fixed_blocker() {
    for requirement_id in ["", " \t\r\n"] {
        let record = ProfileEvidenceRecord::passed(requirement_id);
        let readiness = ProfileEvidenceReadiness::evaluate(BASELINE, &[record])
            .expect("Baseline profile resolves");
        let blocker = readiness
            .blockers()
            .iter()
            .find(|blocker| matches!(blocker, ProfileReadinessBlocker::UnknownRequirementRecord))
            .expect("unknown requirement produces a blocker");

        assert_eq!(blocker.as_str(), UNKNOWN_MESSAGE);
    }
}
