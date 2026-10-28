"""Readiness contracts for operations, capabilities, and public declarations."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from tools.api_manifest import generate
from tools.api_manifest.generate import ManifestError
from tools.api_manifest.tests.test_manifest_v2 import (
    EXPECTED_RELEASE_SHARED_CAPABILITIES,
    _valid_v2_manifest,
)

ROOT = Path(__file__).resolve().parents[3]
SCHEMA_PATH = ROOT / "specification" / "api" / "public-api.schema.json"
LANGUAGES = ("rust", "c", "java", "python")


def _contract_manifest() -> dict[str, object]:
    return _valid_v2_manifest()


def _discover_versions_operation(manifest: dict[str, object]) -> dict[str, object]:
    return next(
        operation
        for operation in manifest["operations"]
        if operation["name"] == "Discover Versions"
    )


def _approved_feature_source() -> dict[str, str]:
    return {
        "kind": "feature-specification",
        "path": "specs/007-client-execution/approval-record.md",
        "status": "Approved",
        "evidence": "Maintainer acceptance for release 1.0.0",
    }


def _four_language_mappings() -> dict[str, list[str]]:
    return {
        "rust": ["kmipkit_client::Client::execute"],
        "c": ["kmipkit_client_execute"],
        "java": ["org.kmipkit.Client.execute"],
        "python": ["kmipkit.Client.execute"],
    }


def _ready_operation(manifest: dict[str, object]) -> dict[str, object]:
    operation = _discover_versions_operation(manifest)
    operation.pop("blockedReason")
    operation.update(
        {
            "apiState": "declared",
            "ownerDependency": {"state": "approved", "specId": "KMIPKIT-0007"},
            "featureSpecId": "KMIPKIT-0007",
            "approvalEvidence": _approved_feature_source(),
            "declarations": {
                language: [f"discover_versions_execute_{language}"]
                for language in LANGUAGES
            },
        }
    )
    return operation


def _ready_declarations(
    *, declaration_id: str = "shared_client_execute", capability_id: str = "shared_client"
) -> list[dict[str, object]]:
    package_ids = {
        "rust": "rust-kmipkit-protocol",
        "c": "c-kmipkit",
        "java": "java-kmipkit",
        "python": "python-kmipkit",
    }
    export_names = {language: f"{language}.{declaration_id}" for language in LANGUAGES}
    return [
        {
            "declarationId": f"{declaration_id}_{language}",
            "packageId": package_ids[language],
            "kind": "function",
            "exportName": export_names[language],
            "capabilityId": capability_id,
            "requirementIds": ["KMIPKIT-0043-FR-001"],
            "declarationState": "declared",
            "parityScope": "four-language",
            "approvalSource": _approved_feature_source(),
            "languageMappings": {
                mapped_language: {
                    "packageId": package_ids[mapped_language],
                    "name": export_names[mapped_language],
                }
                for mapped_language in LANGUAGES
            },
        }
        for language in LANGUAGES
    ]


def _ready_capability() -> dict[str, object]:
    return {
        "capabilityId": "shared_client",
        "requirementIds": ["KMIPKIT-0043-FR-001"],
        "capabilityState": "declared",
        "parityScope": "four-language",
        "ownerSpecId": "KMIPKIT-0007",
        "approvalEvidence": _approved_feature_source(),
        "mappings": {
            language: [f"{language}.shared_client_execute"]
            for language in LANGUAGES
        },
    }


def _write_approved_feature(root: Path) -> None:
    owner = root / "specs" / "007-client-execution"
    owner.mkdir(parents=True)
    (owner / "spec.md").write_text(
        "# Feature\n\n**Status**: Approved for implementation.\n",
        encoding="utf-8",
    )
    (owner / "approval-record.md").write_text(
        "Maintainer acceptance for release 1.0.0\n",
        encoding="utf-8",
    )


def _complete_readiness_manifest(root: Path) -> dict[str, object]:
    _write_approved_feature(root)
    evidence = root / "tests" / "fixtures" / "api" / "parity.json"
    evidence.parent.mkdir(parents=True)
    evidence.write_text("{}\n", encoding="utf-8")
    manifest = _contract_manifest()
    language_packages = {
        "rust": "rust-kmipkit-protocol",
        "c": "c-kmipkit",
        "java": "java-kmipkit",
        "python": "python-kmipkit",
    }
    declaration_ids: dict[str, dict[str, str]] = {}
    for index, operation in enumerate(manifest["operations"]):
        declaration_id = f"operation_{index:02d}"
        declaration_ids[operation["operationId"]] = {
            language: f"{declaration_id}_{language}" for language in LANGUAGES
        }
        operation.pop("blockedReason")
        operation.update(
            {
                "apiState": "verified",
                "ownerDependency": {"state": "approved", "specId": "KMIPKIT-0007"},
                "featureSpecId": "KMIPKIT-0007",
                "approvalEvidence": _approved_feature_source(),
                "declarations": {
                    language: [declaration_ids[operation["operationId"]][language]]
                    for language in LANGUAGES
                },
                "parityEvidenceIds": ["tests/fixtures/api/parity.json"],
            }
        )
    manifest["publicDeclarations"] = [
        {
            "declarationId": declaration_ids[operation["operationId"]][language],
            "packageId": language_packages[language],
            "kind": "function",
            "exportName": f"{language}.operation_{index:02d}",
            "capabilityId": operation["operationId"],
            "requirementIds": ["KMIPKIT-0043-FR-001"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "approvalSource": _approved_feature_source(),
            "languageMappings": {
                mapped_language: {
                    "packageId": language_packages[mapped_language],
                    "name": f"{mapped_language}.operation_{index:02d}",
                }
                for mapped_language in LANGUAGES
            },
            "parityEvidenceIds": ["tests/fixtures/api/parity.json"],
        }
        for index, operation in enumerate(manifest["operations"])
        for language in LANGUAGES
    ]
    manifest["publicDeclarations"].extend(
        {
            "declarationId": f"shared_capability_{index:02d}_{language}",
            "packageId": language_packages[language],
            "kind": "function",
            "exportName": f"{language}.shared_{index:02d}",
            "capabilityId": capability_id,
            "requirementIds": ["KMIPKIT-0043-FR-001"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "approvalSource": _approved_feature_source(),
            "languageMappings": {
                mapped_language: {
                    "packageId": language_packages[mapped_language],
                    "name": f"{mapped_language}.shared_{index:02d}",
                }
                for mapped_language in LANGUAGES
            },
            "parityEvidenceIds": ["tests/fixtures/api/parity.json"],
        }
        for index, capability_id in enumerate(
            sorted(EXPECTED_RELEASE_SHARED_CAPABILITIES)
        )
        for language in LANGUAGES
    )
    manifest["capabilities"] = [
        {
            "capabilityId": capability_id,
            "requirementIds": ["KMIPKIT-0043-FR-001"],
            "capabilityState": "verified",
            "parityScope": "four-language",
            "ownerSpecId": "KMIPKIT-0007",
            "approvalEvidence": _approved_feature_source(),
            "mappings": {
                language: [f"{language}.shared_{index:02d}"]
                for language in LANGUAGES
            },
            "parityEvidenceIds": ["tests/fixtures/api/parity.json"],
        }
        for index, capability_id in enumerate(
            sorted(EXPECTED_RELEASE_SHARED_CAPABILITIES)
        )
    ]
    return manifest


def _validate_v2_readiness(
    test: unittest.TestCase,
    manifest: dict[str, object],
    root: Path,
    *,
    require_complete: bool = False,
) -> None:
    _write_dependency_matrix_fixture_if_missing(manifest, root)
    validator = getattr(generate, "_validate_v2_readiness", None)
    test.assertTrue(callable(validator), "v2 readiness validation is not implemented")
    validator(manifest, root, require_complete=require_complete)


def _write_dependency_matrix_fixture_if_missing(
    manifest: dict[str, object], root: Path
) -> None:
    matrix = root / "specs" / "043-public-api-bindings" / "contracts" / "operation-dependencies.md"
    if matrix.exists():
        return
    rows = [
        "# Test operation dependencies",
        "",
        "| OASIS section | Operation | Linked owner specification | Current status / API gate |",
        "|---|---|---|---|",
    ]
    for operation in manifest["operations"]:
        dependency = operation.get(
            "ownerDependency", {"state": "unassigned", "specId": None}
        )
        spec_id = dependency["specId"]
        state = dependency["state"]
        if spec_id is None:
            owner = "—"
        else:
            owner = spec_id
            number = int(spec_id[-4:])
            existing = sorted((root / "specs").glob(f"{number:03d}-*"))
            feature = existing[0] if existing else root / "specs" / f"{number:03d}-test-owner"
            feature.mkdir(parents=True, exist_ok=True)
            status = "Approved" if state == "approved" else "Draft"
            (feature / "spec.md").write_text(
                f"# Test owner\n\n**Status**: {status}.\n", encoding="utf-8"
            )
        rows.append(
            f"| §{operation['section']} | {operation['name']} | {owner} | Test fixture. |"
        )
    matrix.parent.mkdir(parents=True, exist_ok=True)
    matrix.write_text("\n".join(rows) + "\n", encoding="utf-8")


class OperationReadinessSchemaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        cls.validator = Draft202012Validator(cls.schema)

    def errors(self, manifest: dict[str, object]) -> list[object]:
        return list(self.validator.iter_errors(manifest))

    def test_feature_owner_id_resolves_through_spec_feature_branch(self) -> None:
        feature = generate._feature_directory(ROOT, "KMIPKIT-0036")
        self.assertEqual(feature.name, "022-object-lifecycle")

    def test_merged_spec_pr_approval_evidence_resolves_stale_owner_status(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            owner = root / "specs" / "018-managed-object-lifecycle" / "spec.md"
            owner.parent.mkdir(parents=True)
            owner.write_text(
                "# Lifecycle\n\n**Status**: Draft for independent human review\n",
                encoding="utf-8",
            )
            evidence = (
                root
                / "specs"
                / "043-public-api-bindings"
                / "implementation-evidence.md"
            )
            evidence.parent.mkdir(parents=True)
            evidence.write_text(
                "- **Merged owner approval evidence**: `KMIPKIT-0018` is approved "
                "for implementation by specification PR #65, MERGED into "
                "`release/1.0.0` on `2026-10-09`; the PR body gates implementation "
                "on approval and merge. The checked-in owner status remains "
                "`Status: Draft for independent human review`.\n",
                encoding="utf-8",
            )

            self.assertEqual(
                "approved", generate._operation_owner_state(root, "KMIPKIT-0018")
            )

    def test_autonomous_implementation_status_without_merged_approval_is_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            owner = root / "specs" / "020-query-ping" / "spec.md"
            owner.parent.mkdir(parents=True)
            owner.write_text(
                "# Query and Ping\n\n"
                "**Status**: Approved for autonomous implementation under a standing instruction.\n",
                encoding="utf-8",
            )

            self.assertEqual(
                "candidate", generate._operation_owner_state(root, "KMIPKIT-0020")
            )

    def test_merged_spec_pr_approval_resolves_autonomous_implementation_status(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            owner = root / "specs" / "020-query-ping" / "spec.md"
            owner.parent.mkdir(parents=True)
            owner.write_text(
                "# Query and Ping\n\n"
                "**Status**: Approved for autonomous implementation under a standing instruction.\n",
                encoding="utf-8",
            )
            evidence = (
                root
                / "specs"
                / "043-public-api-bindings"
                / "implementation-evidence.md"
            )
            evidence.parent.mkdir(parents=True)
            evidence.write_text(
                "- **Merged owner approval evidence**: `KMIPKIT-0020` is approved "
                "for implementation by specification PR #67, MERGED into "
                "`release/1.0.0` on `2026-10-09`; the PR body gates implementation "
                "on approval and merge. The checked-in owner status remains "
                "`Approved for autonomous implementation under a standing instruction`.\n",
                encoding="utf-8",
            )

            self.assertTrue(
                generate._has_merged_specification_approval_record(
                    root, "KMIPKIT-0020"
                )
            )
            self.assertEqual(
                "approved", generate._operation_owner_state(root, "KMIPKIT-0020")
            )
            status_evidence = (
                "**Status**: Approved for autonomous implementation under "
                "a standing instruction."
            )
            generate._validate_approval_source(
                {
                    "kind": "feature-specification",
                    "path": "specs/020-query-ping/spec.md",
                    "status": "Approved",
                    "evidence": status_evidence,
                },
                root,
                feature_id="KMIPKIT-0020",
            )

    def test_authorization_only_status_is_not_manifest_approval_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            owner = root / "specs" / "020-query-ping" / "spec.md"
            owner.parent.mkdir(parents=True)
            status = (
                "**Status**: Approved for autonomous implementation under a standing instruction."
            )
            owner.write_text(f"# Query and Ping\n\n{status}\n", encoding="utf-8")

            with self.assertRaisesRegex(ManifestError, "merged specification approval"):
                generate._validate_approval_source(
                    {
                        "kind": "feature-specification",
                        "path": "specs/020-query-ping/spec.md",
                        "status": "Approved",
                        "evidence": status,
                    },
                    root,
                    feature_id="KMIPKIT-0020",
                )

    def test_dependency_check_rejects_authorization_only_owner_as_approved(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            owner = root / "specs" / "020-query-ping" / "spec.md"
            owner.parent.mkdir(parents=True)
            owner.write_text(
                "# Query and Ping\n\n"
                "**Status**: Approved for autonomous implementation under a standing instruction.\n",
                encoding="utf-8",
            )
            matrix = root / "specs" / "043-public-api-bindings" / "contracts"
            matrix.mkdir(parents=True)
            lines = [
                "# Operation dependencies",
                "",
                "| OASIS section | Operation | Owner | Status |",
                "|---|---|---|---|",
            ]
            operations: list[dict[str, object]] = []
            for number in range(1, 57):
                section = "6.1.40" if number == 40 else f"6.1.{number}"
                name = "Query" if number == 40 else f"Operation {number}"
                owner_id = "KMIPKIT-0020" if number == 40 else None
                lines.append(
                    f"| §{section} | {name} | {owner_id or '—'} | Test fixture. |"
                )
                operations.append(
                    {
                        "section": section,
                        "name": name,
                        "ownerDependency": {
                            "state": "approved" if owner_id else "unassigned",
                            "specId": owner_id,
                        },
                    }
                )
            (matrix / "operation-dependencies.md").write_text(
                "\n".join(lines) + "\n", encoding="utf-8"
            )

            with self.assertRaisesRegex(
                ManifestError, "owner state differs from its specification"
            ):
                generate._validate_operation_dependencies(operations, root)

    def test_merged_spec_pr_allows_manifest_source_with_stale_draft_status(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            owner = root / "specs" / "018-managed-object-lifecycle" / "spec.md"
            owner.parent.mkdir(parents=True)
            owner.write_text(
                "# Lifecycle\n\n"
                "**Status**: Draft for independent human review\n",
                encoding="utf-8",
            )
            evidence = (
                root
                / "specs"
                / "043-public-api-bindings"
                / "implementation-evidence.md"
            )
            evidence.parent.mkdir(parents=True)
            evidence.write_text(
                "- **Merged owner approval evidence**: `KMIPKIT-0018` is approved "
                "for implementation by specification PR #65, MERGED into "
                "`release/1.0.0` on `2026-10-09`; the PR body gates implementation "
                "on approval and merge. The checked-in owner status remains "
                "`Status: Draft for independent human review`.\n",
                encoding="utf-8",
            )

            generate._validate_approval_source(
                {
                    "kind": "feature-specification",
                    "path": "specs/018-managed-object-lifecycle/spec.md",
                    "status": "Approved",
                    "evidence": "**Status**: Draft for independent human review",
                },
                root,
                feature_id="KMIPKIT-0018",
            )

    def test_pkcs11_dependency_matrix_preserves_implementation_only_approval(self) -> None:
        matrix_path = (
            ROOT
            / "specs"
            / "043-public-api-bindings"
            / "contracts"
            / "operation-dependencies.md"
        )
        matrix = matrix_path.read_text(encoding="utf-8")
        pkcs11_row = next(
            line
            for line in matrix.splitlines()
            if line.startswith("| §6.1.37 | PKCS#11 |")
        )
        owner_spec = (
            ROOT / "specs" / "026-pkcs11" / "spec.md"
        ).read_text(encoding="utf-8")
        manifest = json.loads(
            (ROOT / "specification" / "api" / "public-api.json").read_text(
                encoding="utf-8"
            )
        )
        pkcs11_operation = next(
            operation
            for operation in manifest["operations"]
            if operation["section"] == "6.1.37"
        )

        self.assertIn("Approved for implementation", owner_spec)
        self.assertIn("does not start implementation", owner_spec)
        self.assertEqual(pkcs11_operation["apiState"], "verified")
        self.assertEqual(
            pkcs11_operation["ownerDependency"],
            {"state": "approved", "specId": "KMIPKIT-0040"},
        )
        self.assertEqual(
            pkcs11_operation["approvalEvidence"]["status"], "Approved"
        )
        self.assertNotIn("blockedReason", pkcs11_operation)
        self.assertIn("Approved for implementation", pkcs11_row)
        self.assertIn("not formal acceptance", pkcs11_row)
        self.assertNotIn("Approved owner", pkcs11_row)

        fixture = json.loads(
            (ROOT / "tests" / "fixtures" / "api" / "pkcs11-parity.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(fixture["authorization"]["manifestApiState"], "verified")
        self.assertEqual(
            fixture["authorization"]["implementationAuthorization"]["date"],
            "2026-10-20",
        )
        self.assertFalse(fixture["authorization"]["formalAcceptance"])

    def test_dependency_matrix_matches_current_owner_and_api_readiness(self) -> None:
        matrix_path = (
            ROOT
            / "specs"
            / "043-public-api-bindings"
            / "contracts"
            / "operation-dependencies.md"
        )
        matrix = matrix_path.read_text(encoding="utf-8")
        operation_rows = {
            fields[1]: fields[3]
            for line in matrix.splitlines()
            if line.startswith("| §6.1.")
            for fields in [[field.strip() for field in line.strip("|").split("|")]]
        }
        owner_counts = {
            state: sum(
                operation["ownerDependency"]["state"] == state
                for operation in self._release_manifest()["operations"]
            )
            for state in ("approved", "candidate", "unassigned")
        }
        manifest = self._release_manifest()

        self.assertEqual(56, len(operation_rows))
        self.assertEqual(52, owner_counts["approved"])
        self.assertEqual(4, owner_counts["candidate"])
        self.assertEqual(0, owner_counts["unassigned"])
        self.assertIn(
            f"Linked to an approved/accepted owner: **{owner_counts['approved']} operations**.",
            matrix,
        )
        self.assertIn(
            f"Linked to an unapproved candidate owner: **{owner_counts['candidate']} operations**.",
            matrix,
        )

        for operation in manifest["operations"]:
            if operation["apiState"] == "verified":
                with self.subTest(operation=operation["name"]):
                    self.assertNotIn("binding remains blocked", operation_rows[operation["name"]])

    def _release_manifest(self) -> dict[str, object]:
        return json.loads(
            (ROOT / "specification" / "api" / "public-api.json").read_text(
                encoding="utf-8"
            )
        )

    def test_blocked_operation_records_only_candidate_and_reason(self) -> None:
        manifest = _contract_manifest()
        operation = manifest["operations"][0]
        operation["candidateSpecId"] = "KMIPKIT-0009"
        self.assertEqual(self.errors(manifest), [])

        operation["featureSpecId"] = "KMIPKIT-0009"
        operation["approvalEvidence"] = _approved_feature_source()
        operation["declarations"] = _four_language_mappings()
        self.assertTrue(self.errors(manifest))

    def test_approved_release_operations_resolve_to_their_current_owner_specs(self) -> None:
        manifest = json.loads((ROOT / "specification" / "api" / "public-api.json").read_text(encoding="utf-8"))
        expected_owner_by_section = {
            "6.1.6": "KMIPKIT-0036",
            "6.1.7": "KMIPKIT-0036",
            "6.1.8": "KMIPKIT-0014",
            "6.1.9": "KMIPKIT-0014",
            "6.1.10": "KMIPKIT-0014",
            "6.1.12": "KMIPKIT-0038",
            "6.1.14": "KMIPKIT-0037",
            "6.1.16": "KMIPKIT-0007",
            "6.1.18": "KMIPKIT-0037",
            "6.1.19": "KMIPKIT-0017",
            "6.1.22": "KMIPKIT-0039",
            "6.1.23": "KMIPKIT-0039",
            "6.1.24": "KMIPKIT-0021",
            "6.1.25": "KMIPKIT-0037",
            "6.1.26": "KMIPKIT-0042",
            "6.1.27": "KMIPKIT-0037",
            "6.1.28": "KMIPKIT-0017",
            "6.1.29": "KMIPKIT-0042",
            "6.1.30": "KMIPKIT-0038",
            "6.1.31": "KMIPKIT-0038",
            "6.1.32": "KMIPKIT-0021",
            "6.1.33": "KMIPKIT-0021",
            "6.1.35": "KMIPKIT-0036",
            "6.1.37": "KMIPKIT-0040",
            "6.1.43": "KMIPKIT-0036",
            "6.1.44": "KMIPKIT-0036",
            "6.1.45": "KMIPKIT-0036",
            "6.1.46": "KMIPKIT-0036",
            "6.1.47": "KMIPKIT-0036",
            "6.1.48": "KMIPKIT-0036",
            "6.1.49": "KMIPKIT-0041",
            "6.1.50": "KMIPKIT-0041",
            "6.1.52": "KMIPKIT-0039",
            "6.1.53": "KMIPKIT-0039",
            "6.1.55": "KMIPKIT-0021",
            "6.1.56": "KMIPKIT-0021",
            "6.1.57": "KMIPKIT-0042",
        }

        operation_by_section = {
            operation["section"]: operation for operation in manifest["operations"]
        }
        self.assertEqual(len(expected_owner_by_section), 37)
        for section, spec_id in expected_owner_by_section.items():
            with self.subTest(section=section):
                dependency = operation_by_section[section]["ownerDependency"]
                self.assertEqual(dependency, {"state": "approved", "specId": spec_id})

    def test_checked_in_operation_dependencies_resolve_against_dependency_matrix(self) -> None:
        manifest = json.loads((ROOT / "specification" / "api" / "public-api.json").read_text(encoding="utf-8"))
        _validate_v2_readiness(self, manifest, ROOT)
        operation = manifest["operations"][0]
        operation["ownerDependency"]["specId"] = "KMIPKIT-9999"
        with self.assertRaises(ManifestError):
            _validate_v2_readiness(self, manifest, ROOT)

    def test_declared_operation_requires_approved_owner_and_all_language_mappings(self) -> None:
        manifest = _contract_manifest()
        operation = _discover_versions_operation(manifest)
        operation.pop("blockedReason")
        operation["apiState"] = "declared"
        operation["featureSpecId"] = "KMIPKIT-0007"
        operation["approvalEvidence"] = _approved_feature_source()
        operation["declarations"] = _four_language_mappings()
        self.assertEqual(self.errors(manifest), [])

        del operation["declarations"]["java"]
        self.assertTrue(self.errors(manifest))

    def test_verified_operation_requires_resolvable_parity_evidence(self) -> None:
        manifest = _contract_manifest()
        operation = _discover_versions_operation(manifest)
        operation.pop("blockedReason")
        operation["apiState"] = "verified"
        operation["featureSpecId"] = "KMIPKIT-0007"
        operation["approvalEvidence"] = _approved_feature_source()
        operation["declarations"] = _four_language_mappings()
        self.assertTrue(self.errors(manifest))

        operation["parityEvidenceIds"] = ["tests/fixtures/api/client-parity.json"]
        self.assertEqual(self.errors(manifest), [])

    def test_blocked_observed_export_requires_exact_candidate_and_reason_without_mappings(self) -> None:
        manifest = _contract_manifest()
        manifest["publicDeclarations"] = [
            {
                "declarationId": "observed_draft_export",
                "packageId": "rust-kmipkit-client",
                "kind": "function",
                "exportName": "kmipkit_client::candidate",
                "capabilityId": "candidate_operation",
                "requirementIds": ["KMIPKIT-0043-FR-001"],
                "declarationState": "blocked",
                "parityScope": "four-language",
                "candidateSpecId": "KMIPKIT-0017",
                "blockedReason": "The owning operation specification is not approved.",
            }
        ]
        self.assertEqual(self.errors(manifest), [])

        del manifest["publicDeclarations"][0]["candidateSpecId"]
        self.assertTrue(self.errors(manifest))

    def test_capability_state_uses_required_parity_scope_mappings(self) -> None:
        manifest = _contract_manifest()
        manifest["capabilities"] = [
            {
                "capabilityId": "shared_client",
                "requirementIds": ["KMIPKIT-0043-FR-001"],
                "capabilityState": "declared",
                "parityScope": "four-language",
                "ownerSpecId": "KMIPKIT-0007",
                "approvalEvidence": _approved_feature_source(),
                "mappings": _four_language_mappings(),
            }
        ]
        self.assertEqual(self.errors(manifest), [])

        del manifest["capabilities"][0]["mappings"]["python"]
        self.assertTrue(self.errors(manifest))

    def test_ready_owner_must_resolve_to_approved_feature_and_recorded_acceptance(self) -> None:
        manifest = _contract_manifest()
        _ready_operation(manifest)
        manifest["publicDeclarations"] = _ready_declarations(
            declaration_id="discover_versions_execute",
            capability_id="KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS",
        )
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _write_approved_feature(root)
            _validate_v2_readiness(self, manifest, root)

            (root / "specs" / "007-client-execution" / "spec.md").write_text(
                "# Feature\n\n**Status**: Draft.\n", encoding="utf-8"
            )
            with self.assertRaises(ManifestError):
                _validate_v2_readiness(self, manifest, root)

    def test_adr_owned_capability_must_resolve_to_accepted_adr(self) -> None:
        manifest = _contract_manifest()
        adr_path = "docs/adr/0014-public-transport-exchange-contract.md"
        manifest["capabilities"] = [
            {
                "capabilityId": "transport_exchange",
                "requirementIds": ["KMIPKIT-0043-FR-001"],
                "capabilityState": "declared",
                "parityScope": "rust-only-adr",
                "ownerAdrId": "ADR-0014",
                "acceptedEvidence": {
                    "kind": "accepted-adr",
                    "path": adr_path,
                    "status": "Accepted",
                    "evidence": "Maintainer acceptance for release 1.0.0",
                },
                "mappings": {"rust": []},
            }
        ]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            adr = root / adr_path
            adr.parent.mkdir(parents=True)
            adr.write_text(
                "# ADR-0014\n\n**Status**: Accepted.\n\n"
                "Maintainer acceptance for release 1.0.0\n",
                encoding="utf-8",
            )
            _validate_v2_readiness(self, manifest, root)

            adr.write_text(
                "# ADR-0014\n\nStatus: Accepted.\n\n"
                "Maintainer acceptance for release 1.0.0\n",
                encoding="utf-8",
            )
            _validate_v2_readiness(self, manifest, root)

            adr.write_text(
                "# ADR-0014\n\n**Status**: Proposed.\n\n"
                "Maintainer acceptance for release 1.0.0\n",
                encoding="utf-8",
            )
            with self.assertRaises(ManifestError):
                _validate_v2_readiness(self, manifest, root)

    def test_language_declaration_mapping_must_name_a_package_in_that_language(self) -> None:
        manifest = _contract_manifest()
        declarations = _ready_declarations()
        manifest["capabilities"] = [_ready_capability()]
        declarations[0]["languageMappings"]["python"]["packageId"] = "java-kmipkit"
        manifest["publicDeclarations"] = declarations

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _write_approved_feature(root)
            with self.assertRaises(ManifestError):
                _validate_v2_readiness(self, manifest, root)

    def test_release_validation_rejects_each_kind_of_blocked_readiness_record(self) -> None:
        mutations = (
            lambda manifest: manifest["operations"][0].update({"apiState": "blocked"}),
            lambda manifest: manifest["capabilities"][0].update(
                {"capabilityState": "blocked"}
            ),
            lambda manifest: manifest["publicDeclarations"][0].update(
                {"declarationState": "blocked"}
            ),
        )
        for mutate in mutations:
            with self.subTest(record=mutate.__code__.co_firstlineno):
                with tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary)
                    manifest = _complete_readiness_manifest(root)
                    mutate(manifest)
                    with self.assertRaises(ManifestError):
                        _validate_v2_readiness(
                            self, manifest, root, require_complete=True
                        )

    def test_complete_readiness_records_pass_release_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = _complete_readiness_manifest(root)
            _validate_v2_readiness(self, manifest, root, require_complete=True)

if __name__ == "__main__":
    unittest.main()
