from __future__ import annotations

from kmipkit import profile_catalog
from kmipkit.profile_readiness import ProfileEvidenceRecord, ProfileReadiness

BASELINE_ID = "KMIPKIT-PROFILE-BASELINE-CLIENT"


def test_baseline_readiness_uses_rust_evaluator_and_never_claims_support() -> None:
    profiles = profile_catalog.client_profiles()
    baseline = next(row for row in profiles if row.profile_id == BASELINE_ID)
    evidence = tuple(ProfileEvidenceRecord.passed(item) for item in baseline.requirement_ids)

    result = ProfileReadiness.evaluate(BASELINE_ID, evidence)

    assert result is not None
    assert result.profile_id == BASELINE_ID
    assert result.target_selected is True
    assert result.runtime_validation_available is False
    assert result.effective_state == "evidence_incomplete"
    assert result.requirement_records_complete is True
    assert result.published_claim is False
    assert any(blocker.kind == "verification_manifest_unavailable" for blocker in result.blockers)
    assert ProfileReadiness.evaluate("KMIPKIT-PROFILE-UNKNOWN", ()) is None


def test_readiness_blockers_keep_only_stable_optional_identifiers() -> None:
    failed = ProfileReadiness.evaluate(
        BASELINE_ID,
        (ProfileEvidenceRecord.failed("KMIPKIT-REQ-PROF-4.1.2-001"),
         ProfileEvidenceRecord.passed("KMIPKIT-REQ-PROF-4.1.2-001"),
         ProfileEvidenceRecord.failed("UNKNOWN-CALLER-MARKER")),
    )

    assert failed is not None
    assert all(isinstance(item.kind, str) and item.message for item in failed.blockers)
    assert any(item.kind == "duplicate_requirement_record" for item in failed.blockers)
    assert any(item.kind == "unknown_requirement_record" for item in failed.blockers)
    assert all("UNKNOWN-CALLER-MARKER" not in item.message for item in failed.blockers)
    assert all(isinstance(item.requirement_id, (str, type(None))) for item in failed.blockers)
    assert all(isinstance(item.profile_id, (str, type(None))) for item in failed.blockers)
    assert all(isinstance(item.test_id, (str, type(None))) for item in failed.blockers)
    assert all(isinstance(item.source_clause_id, (str, type(None))) for item in failed.blockers)


def test_catalog_snapshots_are_complete_ordered_and_immutable() -> None:
    profiles = profile_catalog.client_profiles()
    requirements = profile_catalog.client_requirements()
    test_cases = profile_catalog.profile_test_cases()

    assert isinstance(profiles, tuple) and len(profiles) == 18
    assert isinstance(requirements, tuple) and len(requirements) == 483
    assert isinstance(test_cases, tuple) and len(test_cases) == 94
    assert tuple(row.profile_id for row in profiles) == tuple(sorted(row.profile_id for row in profiles))
    assert tuple(row.requirement_id for row in requirements) == tuple(sorted(row.requirement_id for row in requirements))
    assert tuple(row.test_id for row in test_cases) == tuple(sorted(row.test_id for row in test_cases))
    assert {row.verification_category.value for row in requirements} == {
        "required_behavior", "prohibited_behavior", "recommended_behavior_or_deviation",
        "discouraged_absence_or_deviation", "optional_capability",
    }

    for row in profiles:
        assert row.profile_id and row.name and row.applicability and row.claim_state
        assert isinstance(row.selected_target, bool)
        for values in (row.direct_source_clause_ids, row.source_clause_ids, row.requirement_ids,
                       row.dependency_profile_ids, row.transport_requirements,
                       row.encoding_requirements, row.test_cases, row.unresolved_scope_obligations):
            assert isinstance(values, tuple)
        for blocker in row.unresolved_scope_obligations:
            assert blocker.requirement_id and blocker.source_clause_id and blocker.scope_state
        for case in row.test_cases:
            assert case.test_id and case.official_case_id and case.mandatory_status and case.fixture_availability
    for row in requirements:
        assert row.requirement_id and row.scope_state
        assert isinstance(row.has_accepted_deviation, bool)
        for value in (row.source_keyword, row.source_force, row.source_gap_id, row.source_locator):
            assert value is None or isinstance(value, str)
        for values in (row.source_clause_ids, row.profile_ids, row.official_test_case_ids):
            assert isinstance(values, tuple)
    for row in test_cases:
        assert row.test_id and row.official_case_id and row.mandatory_status and row.fixture_availability
        assert isinstance(row.mandatory_status, str) and isinstance(row.fixture_availability, str)

    baseline = next(row for row in profiles if row.profile_id == BASELINE_ID)
    assert baseline.claim_state == "evidence_incomplete"
    assert baseline.selected_target is True
