"""T095 contract for KMIPKIT-0013 production transport declarations."""

from __future__ import annotations

import hashlib
import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "KMIPKIT-0013"

# Fingerprint of (packageId, exportName) from the exact 14c7f173 base. This pins
# the complete 258-path inventory, including 222 compiler-emitted trait paths,
# without repeating those long paths in the test source.
BASE_CANDIDATE_COUNT = 258
BASE_CANDIDATE_SHA256 = "5bbdc52dc931c30043be197b4c34ac9d0568ee8baee077fb63a3a26f36d02fb0"
BASE_FUNCTIONAL_EXPORTS = {
    "kmipkit_transport::TimeoutPolicy::connect",
    "kmipkit_transport::TimeoutPolicy::write",
    "kmipkit_transport::TimeoutPolicy::read",
    "kmipkit_transport::TimeoutPolicy::total",
    "kmipkit_transport::TimeoutPolicy::with_overrides",
    "kmipkit_transport::TimeoutPolicy::with_connect",
    "kmipkit_transport::TimeoutPolicy::with_write",
    "kmipkit_transport::TimeoutPolicy::with_read",
    "kmipkit_transport::TimeoutPolicy::with_total",
    "kmipkit_transport::TransportConfig::endpoint",
    "kmipkit_transport::TransportConfig::target_uri",
    "kmipkit_transport::TransportConfig::max_request_bytes",
    "kmipkit_transport::TransportConfig::timeouts",
    "kmipkit_transport::TransportConfigBuilder::client_identity",
    "kmipkit_transport::TransportConfigBuilder::trust_source",
    "kmipkit_transport::TransportConfigError",
    "kmipkit_transport::TransportConfigError::InvalidEndpoint",
    "kmipkit_transport::TransportConfigError::InvalidTarget",
    "kmipkit_transport::TransportConfigError::InvalidRequestLimit",
    "kmipkit_transport::TransportConfigError::MissingIdentity",
    "kmipkit_transport::TransportConfigError::MissingTrust",
    "kmipkit_transport::TransportConfigError::InvalidCredential",
    "kmipkit_transport::TransportConfigError::InvalidTrust",
    "kmipkit_transport::TransportConfigError::InvalidRevocationList",
    "kmipkit_transport::TransportConfigError::InvalidServerName",
    "kmipkit_transport::TransportConfigError::InvalidTlsConfiguration",
    "kmipkit_transport::TransportConfigError::PlatformTrustUnavailable",
    "kmipkit_transport::TrustSource::CertificateAuthorities",
    "kmipkit_transport::TrustSource::CertificateAuthorities::0",
    "kmipkit_transport::HttpsTransport",
    "kmipkit_transport::HttpsTransport::new",
    "kmipkit_transport::HttpsTransport::exchange_with_options",
    "kmipkit_transport::RawTlsTransport",
    "kmipkit_transport::RawTlsTransport::new",
    "kmipkit_transport::RawTlsTransport::exchange_with_options",
    "kmipkit_transport::TimeoutLimit::Bounded::0",
}
RUST_ONLY_EXPORTS = {
    "kmipkit_transport::HttpsTransport",
    "kmipkit_transport::HttpsTransport::new",
    "kmipkit_transport::HttpsTransport::exchange_with_options",
    "kmipkit_transport::RawTlsTransport",
    "kmipkit_transport::RawTlsTransport::new",
    "kmipkit_transport::RawTlsTransport::exchange_with_options",
}
EXCLUDED_READBACK_EXPORTS = {
    "kmipkit_transport::TimeoutPolicy::connect",
    "kmipkit_transport::TimeoutPolicy::write",
    "kmipkit_transport::TimeoutPolicy::read",
    "kmipkit_transport::TimeoutPolicy::total",
    "kmipkit_transport::TransportConfig::endpoint",
    "kmipkit_transport::TransportConfig::target_uri",
    "kmipkit_transport::TransportConfig::max_request_bytes",
    "kmipkit_transport::TransportConfig::timeouts",
}
FOUR_LANGUAGE_EXPORTS = BASE_FUNCTIONAL_EXPORTS - RUST_ONLY_EXPORTS - EXCLUDED_READBACK_EXPORTS
LANGUAGES = {"rust", "c", "java", "python"}
OWNER_SPEC_PATH = "specs/013-production-transport/spec.md"
ADR_0014_PATH = "docs/adr/0014-public-transport-exchange-contract.md"
T095_REQUIREMENT_IDS = {
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
SHARED_OWNER_REQUIREMENT_IDS = T095_REQUIREMENT_IDS
READBACK_EXCLUSION_REQUIREMENT_IDS = {
    "KMIPKIT-0013-FR-001",
    "KMIPKIT-0013-FR-007",
    "KMIPKIT-0013-FR-008",
    "KMIPKIT-0013-FR-009",
    "KMIPKIT-0013-FR-011",
    "KMIPKIT-0043-FR-021",
}
TRAIT_EXCLUSION_REQUIREMENT_IDS = {"KMIPKIT-0043-FR-021"}


def _shared_requirements(*owner_requirements: str) -> set[str]:
    return SHARED_OWNER_REQUIREMENT_IDS | {
        f"KMIPKIT-0013-FR-{requirement}" for requirement in owner_requirements
    }


_RUST_ONLY_REQUIREMENTS = _shared_requirements("016") | {"KMIPKIT-0043-FR-020"}
EXPECTED_FUNCTIONAL_REQUIREMENTS = {
    **{
        name: _shared_requirements("011")
        for name in (
            "kmipkit_transport::TimeoutLimit::Bounded::0",
            "kmipkit_transport::TimeoutPolicy::with_connect",
            "kmipkit_transport::TimeoutPolicy::with_overrides",
            "kmipkit_transport::TimeoutPolicy::with_read",
            "kmipkit_transport::TimeoutPolicy::with_total",
            "kmipkit_transport::TimeoutPolicy::with_write",
        )
    },
    "kmipkit_transport::TransportConfigBuilder::client_identity": _shared_requirements("004", "005"),
    "kmipkit_transport::TransportConfigBuilder::trust_source": _shared_requirements("003"),
    "kmipkit_transport::TransportConfigError": _shared_requirements("001"),
    "kmipkit_transport::TransportConfigError::InvalidCredential": _shared_requirements("004", "005"),
    "kmipkit_transport::TransportConfigError::InvalidEndpoint": _shared_requirements("007"),
    "kmipkit_transport::TransportConfigError::InvalidRequestLimit": _shared_requirements("017"),
    "kmipkit_transport::TransportConfigError::InvalidRevocationList": _shared_requirements("003"),
    "kmipkit_transport::TransportConfigError::InvalidServerName": _shared_requirements("002"),
    "kmipkit_transport::TransportConfigError::InvalidTarget": _shared_requirements("008", "009"),
    "kmipkit_transport::TransportConfigError::InvalidTlsConfiguration": _shared_requirements("002"),
    "kmipkit_transport::TransportConfigError::InvalidTrust": _shared_requirements("003"),
    "kmipkit_transport::TransportConfigError::MissingIdentity": _shared_requirements("004"),
    "kmipkit_transport::TransportConfigError::MissingTrust": _shared_requirements("003"),
    "kmipkit_transport::TransportConfigError::PlatformTrustUnavailable": _shared_requirements("003"),
    "kmipkit_transport::TrustSource::CertificateAuthorities": _shared_requirements("003"),
    "kmipkit_transport::TrustSource::CertificateAuthorities::0": _shared_requirements("003"),
    "kmipkit_transport::HttpsTransport": _RUST_ONLY_REQUIREMENTS,
    "kmipkit_transport::HttpsTransport::new": _RUST_ONLY_REQUIREMENTS,
    "kmipkit_transport::HttpsTransport::exchange_with_options": _RUST_ONLY_REQUIREMENTS | {"KMIPKIT-0013-FR-011"},
    "kmipkit_transport::RawTlsTransport": _RUST_ONLY_REQUIREMENTS,
    "kmipkit_transport::RawTlsTransport::new": _RUST_ONLY_REQUIREMENTS,
    "kmipkit_transport::RawTlsTransport::exchange_with_options": _RUST_ONLY_REQUIREMENTS | {"KMIPKIT-0013-FR-011"},
}


def _evidence_requirement_ids(evidence: str) -> set[str]:
    requirement_ids = set(re.findall(r"KMIPKIT-\d{4}-FR-\d{3}", evidence))
    for owner, abbreviated_ids in re.findall(
        r"(KMIPKIT-\d{4}) FR-(\d{3}(?:/\d{3})*)", evidence
    ):
        requirement_ids.update(
            f"{owner}-FR-{requirement}" for requirement in abbreviated_ids.split("/")
        )
    return requirement_ids


def _mapping_targets(export_name: str) -> dict[str, dict[str, str]]:
    rust = {"packageId": "rust-kmipkit-transport", "name": export_name}
    if export_name.endswith("::with_overrides"):
        return {
            "rust": rust,
            "c": {"packageId": "c-kmipkit", "name": "kmipkit_request_options_v1_t"},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.extensions.RequestOptions"},
            "python": {"packageId": "python-kmipkit", "name": "kmipkit.client.RequestOptions"},
        }
    if export_name.endswith("::Bounded::0"):
        return {
            "rust": rust,
            "c": {"packageId": "c-kmipkit", "name": "kmipkit_timeout_limit_v1_t"},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.extensions.TransportConfiguration.Timeout"},
            "python": {"packageId": "python-kmipkit", "name": "kmipkit.client.TimeoutLimit"},
        }
    if "TimeoutPolicy::with_" in export_name:
        phase = export_name.rsplit("_", 1)[1]
        return {
            "rust": rust,
            "c": {"packageId": "c-kmipkit", "name": f"kmipkit_transport_config_v1_t.{phase}_timeout"},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.extensions.TransportConfiguration.Builder.timeouts"},
            "python": {"packageId": "python-kmipkit", "name": f"kmipkit.client.TransportConfig.{phase}_timeout"},
        }
    if export_name.endswith("::client_identity"):
        return {
            "rust": rust,
            "c": {"packageId": "c-kmipkit", "name": "kmipkit_transport_config_v1_t.client_certificates"},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.extensions.TransportConfiguration.Builder.clientIdentity"},
            "python": {"packageId": "python-kmipkit", "name": "kmipkit.client.TransportConfig.client_certificates"},
        }
    if export_name.endswith("::trust_source"):
        return {
            "rust": rust,
            "c": {"packageId": "c-kmipkit", "name": "kmipkit_transport_config_v1_t.trust_kind"},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.extensions.TransportConfiguration.Builder"},
            "python": {"packageId": "python-kmipkit", "name": "kmipkit.client.TrustSource"},
        }
    if export_name.startswith("kmipkit_transport::TransportConfigError"):
        return {
            "rust": rust,
            "c": {"packageId": "c-kmipkit", "name": "KMIPKIT_ERROR_INVALID_INPUT"},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.InvalidInputException"},
            "python": {"packageId": "python-kmipkit", "name": "kmipkit.errors.InvalidInputError"},
        }
    if export_name.startswith("kmipkit_transport::TrustSource::CertificateAuthorities"):
        python_name = (
            "kmipkit.client.TrustSource.certificates"
            if export_name.endswith("::0")
            else "kmipkit.client.TrustSource.certificate_authorities"
        )
        return {
            "rust": rust,
            "c": {"packageId": "c-kmipkit", "name": "kmipkit_transport_config_v1_t.trust_certificates"},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.extensions.TransportConfiguration.Builder.trustCertificates"},
            "python": {"packageId": "python-kmipkit", "name": python_name},
        }
    raise AssertionError(f"no reviewed T095 mapping target for {export_name}")


def _inventory_fingerprint(rows: list[dict[str, object]]) -> str:
    paths = sorted(
        f"{row['packageId']}\0{row['exportName']}"
        for row in rows
        if isinstance(row.get("packageId"), str)
        and isinstance(row.get("exportName"), str)
    )
    return hashlib.sha256("\n".join(paths).encode("utf-8")).hexdigest()


class T095TransportParityManifestTests(unittest.TestCase):
    def test_no_kmipkit_0013_candidate_declarations_remain(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = manifest["publicDeclarations"]
        candidates = [
            row for row in declarations
            if row.get("candidateSpecId") == OWNER_SPEC
            and not (
                row.get("packageId") == "rust-kmipkit"
                and row.get("exportName", "").startswith("kmipkit::transport_config::")
            )
        ]
        exclusions = [
            row
            for row in manifest["approvedExclusions"]
            if row.get("candidateSpecId") == OWNER_SPEC
            and not (
                row.get("packageId") == "rust-kmipkit"
                and row.get("exportName", "").startswith(
                    ("kmipkit::transport_config::", "<kmipkit::transport_config::")
                )
            )
        ]
        functional_rows = {
            row["exportName"]: row
            for row in declarations
            if row.get("packageId") == "rust-kmipkit-transport"
            and row.get("exportName") in BASE_FUNCTIONAL_EXPORTS
        }
        candidate_paths = candidates + list(functional_rows.values()) + exclusions

        self.assertEqual(BASE_CANDIDATE_COUNT, len({row["exportName"] for row in candidate_paths}))
        self.assertEqual(BASE_CANDIDATE_SHA256, _inventory_fingerprint(candidate_paths))
        self.assertEqual(BASE_FUNCTIONAL_EXPORTS, functional_rows.keys() | {row["exportName"] for row in exclusions if row["exportName"] in BASE_FUNCTIONAL_EXPORTS})
        self.assertEqual(
            [],
            candidates,
            "all exact KMIPKIT-0013 public declarations must be mapped or specifically excluded",
        )

        trait_exclusions = [row for row in exclusions if row["exportName"].startswith("<")]
        readback_exclusions = {
            row["exportName"]: row
            for row in exclusions
            if row["exportName"] in EXCLUDED_READBACK_EXPORTS
        }
        self.assertEqual(222, len(trait_exclusions))
        self.assertEqual(EXCLUDED_READBACK_EXPORTS, readback_exclusions.keys())
        self.assertEqual(
            BASE_FUNCTIONAL_EXPORTS - EXCLUDED_READBACK_EXPORTS,
            EXPECTED_FUNCTIONAL_REQUIREMENTS.keys(),
        )

        for exclusion in trait_exclusions + list(readback_exclusions.values()):
            with self.subTest(exclusion=exclusion["exportName"]):
                self.assertTrue(exclusion.get("reason", "").strip())
                self.assertTrue(exclusion.get("evidence", "").strip())
                self.assertEqual(OWNER_SPEC_PATH, exclusion["approvalSource"]["path"])
                self.assertEqual("Accepted", exclusion["approvalSource"]["status"])
                expected_requirements = (
                    TRAIT_EXCLUSION_REQUIREMENT_IDS
                    if exclusion in trait_exclusions
                    else READBACK_EXCLUSION_REQUIREMENT_IDS
                )
                self.assertEqual(
                    expected_requirements,
                    _evidence_requirement_ids(exclusion["evidence"]),
                )

        self.assertEqual(BASE_FUNCTIONAL_EXPORTS - EXCLUDED_READBACK_EXPORTS, functional_rows.keys())
        for export_name in sorted(FOUR_LANGUAGE_EXPORTS):
            row = functional_rows[export_name]
            with self.subTest(export=export_name):
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual("production_transport", row["capabilityId"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(OWNER_SPEC_PATH, row["approvalSource"]["path"])
                self.assertEqual(LANGUAGES, row["languageMappings"].keys())
                self.assertEqual(
                    EXPECTED_FUNCTIONAL_REQUIREMENTS[export_name],
                    set(row["requirementIds"]),
                )
                self.assertEqual(_mapping_targets(export_name), row["languageMappings"])

        for export_name in sorted(RUST_ONLY_EXPORTS):
            row = functional_rows[export_name]
            with self.subTest(export=export_name):
                self.assertEqual("rust-only-adr", row["parityScope"])
                self.assertEqual("rust_transport_exchange", row["capabilityId"])
                self.assertEqual(ADR_0014_PATH, row["approvalSource"]["path"])
                self.assertEqual({"rust"}, row["languageMappings"].keys())
                self.assertEqual(
                    EXPECTED_FUNCTIONAL_REQUIREMENTS[export_name],
                    set(row["requirementIds"]),
                )

        mapped_targets = {
            (mapping["packageId"], mapping["name"])
            for row in functional_rows.values()
            for mapping in row.get("languageMappings", {}).values()
        }
        observed_exports = {
            (row.get("packageId"), row.get("exportName")) for row in declarations
        }
        for package_id, name in mapped_targets:
            with self.subTest(mapping=f"{package_id}:{name}"):
                self.assertIn((package_id, name), observed_exports)


if __name__ == "__main__":
    unittest.main()
