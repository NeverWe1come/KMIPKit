"""Readiness contracts for operations, capabilities, and public declarations."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from tools.api_manifest import generate
from tools.api_manifest.generate import ManifestError
from tools.api_manifest.tests.test_manifest_v2 import _valid_v2_manifest


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
                language: ["discover_versions_execute"] for language in LANGUAGES
            },
        }
    )
    return operation


def _ready_declaration(
    *, declaration_id: str = "shared_client_execute", capability_id: str = "shared_client"
) -> dict[str, object]:
    package_ids = {
        "rust": "rust-kmipkit-protocol",
        "c": "c-kmipkit",
        "java": "java-kmipkit",
        "python": "python-kmipkit",
    }
    return {
        "declarationId": declaration_id,
        "packageId": "rust-kmipkit-protocol",
        "kind": "function",
        "exportName": "kmipkit_protocol::ClientRequest",
        "capabilityId": capability_id,
        "requirementIds": ["KMIPKIT-0043-FR-001"],
        "declarationState": "declared",
        "parityScope": "four-language",
        "approvalSource": _approved_feature_source(),
        "languageMappings": {
            language: {"packageId": package_id, "name": f"{language}.client_execute"}
            for language, package_id in package_ids.items()
        },
    }


def _ready_capability() -> dict[str, object]:
    return {
        "capabilityId": "shared_client",
        "requirementIds": ["KMIPKIT-0043-FR-001"],
        "capabilityState": "declared",
        "parityScope": "four-language",
        "ownerSpecId": "KMIPKIT-0007",
        "approvalEvidence": _approved_feature_source(),
        "mappings": {language: [f"{language}.shared_client"] for language in LANGUAGES},
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
    declaration_ids: dict[str, str] = {}
    for index, operation in enumerate(manifest["operations"]):
        declaration_id = f"operation_{index:02d}"
        declaration_ids[operation["operationId"]] = declaration_id
        operation.pop("blockedReason")
        operation.update(
            {
                "apiState": "verified",
                "ownerDependency": {"state": "approved", "specId": "KMIPKIT-0007"},
                "featureSpecId": "KMIPKIT-0007",
                "approvalEvidence": _approved_feature_source(),
                "declarations": {
                    language: [declaration_id] for language in LANGUAGES
                },
                "parityEvidenceIds": ["tests/fixtures/api/parity.json"],
            }
        )
    manifest["publicDeclarations"] = [
        {
            "declarationId": declaration_ids[operation["operationId"]],
            "packageId": "rust-kmipkit-protocol",
            "kind": "function",
            "exportName": f"kmipkit_protocol::{operation['name'].replace(' ', '_').lower()}",
            "capabilityId": operation["operationId"],
            "requirementIds": ["KMIPKIT-0043-FR-001"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "approvalSource": _approved_feature_source(),
            "languageMappings": {
                language: {
                    "packageId": package_id,
                    "name": f"{language}.operation_{index:02d}",
                }
                for language, package_id in language_packages.items()
            },
            "parityEvidenceIds": ["tests/fixtures/api/parity.json"],
        }
        for index, operation in enumerate(manifest["operations"])
    ]
    manifest["capabilities"] = [
        {
            "capabilityId": "shared_client",
            "requirementIds": ["KMIPKIT-0043-FR-001"],
            "capabilityState": "verified",
            "parityScope": "four-language",
            "ownerSpecId": "KMIPKIT-0007",
            "approvalEvidence": _approved_feature_source(),
            "mappings": {language: [f"{language}.shared_client"] for language in LANGUAGES},
            "parityEvidenceIds": ["tests/fixtures/api/parity.json"],
        }
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

    def test_blocked_operation_records_only_candidate_and_reason(self) -> None:
        manifest = _contract_manifest()
        operation = manifest["operations"][0]
        operation["candidateSpecId"] = "KMIPKIT-0009"
        self.assertEqual(self.errors(manifest), [])

        operation["featureSpecId"] = "KMIPKIT-0009"
        operation["approvalEvidence"] = _approved_feature_source()
        operation["declarations"] = _four_language_mappings()
        self.assertTrue(self.errors(manifest))

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
        manifest["publicDeclarations"] = [
            _ready_declaration(
                declaration_id="discover_versions_execute",
                capability_id="KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS",
            )
        ]
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
                "mappings": {"rust": ["kmipkit_transport::Transport::exchange"]},
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
                "# ADR-0014\n\n**Status**: Proposed.\n\n"
                "Maintainer acceptance for release 1.0.0\n",
                encoding="utf-8",
            )
            with self.assertRaises(ManifestError):
                _validate_v2_readiness(self, manifest, root)

    def test_language_declaration_mapping_must_name_a_package_in_that_language(self) -> None:
        manifest = _contract_manifest()
        declaration = _ready_declaration()
        manifest["capabilities"] = [_ready_capability()]
        declaration["languageMappings"]["python"]["packageId"] = "java-kmipkit"
        manifest["publicDeclarations"] = [declaration]

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
