"""T095 contract for KMIPKIT-0013 production transport declarations."""

from __future__ import annotations

import hashlib
import json
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
        candidates = [row for row in declarations if row.get("candidateSpecId") == OWNER_SPEC]
        functional = {row["exportName"] for row in candidates if not row["exportName"].startswith("<")}

        self.assertEqual(BASE_CANDIDATE_COUNT, len(candidates))
        self.assertEqual(BASE_CANDIDATE_SHA256, _inventory_fingerprint(candidates))
        self.assertEqual(BASE_FUNCTIONAL_EXPORTS, functional)
        self.assertEqual(
            [],
            candidates,
            "all exact KMIPKIT-0013 public declarations must be mapped or specifically excluded",
        )


if __name__ == "__main__":
    unittest.main()
