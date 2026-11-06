"""Local request and generic TTLV validation binding parity."""

from __future__ import annotations

from kmipkit import ttlv, validation


def test_generic_structure_without_schema_does_not_guess_an_operation():
    with ttlv.ttlv_structure_create() as structure:
        with validation.validate_structure(structure) as report:
            assert report.is_valid
            assert report.issues == ()


def test_explicit_create_schema_reports_redacted_stable_metadata():
    with ttlv.ttlv_structure_create() as structure:
        with validation.validate_structure(
            structure, schema="RequestPayload/Create"
        ) as report:
            assert not report.is_valid
            assert len(report.issues) == 2
            issue = report.issues[0]
            assert issue.code == "KMIPKIT-0051-R019"
            assert issue.path == "Request Payload.Object Type"
            assert issue.severity == "ERROR"
            assert issue.source_identifier == "KMIP v2.1"
            assert issue.source_reference == "§6.1.8, Table 186"
            assert "SECRET" not in str(issue)
