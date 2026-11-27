"""A provider fixture is evidence for a table row and its exact endpoint."""

import json
from pathlib import Path
import tempfile
import unittest

from verification.interop.shared.tools.cosmian_minimum import ROOT, SUPPORT_TABLE
from verification.interop.shared.tools.table_vendor_extensions import (
    GROUP_MEMBER_VALUES, HTTP_CHECKS, ROWS, attach_evidence,
)


class VendorExtensionEvidenceTests(unittest.TestCase):
    def test_provider_evidence_requires_the_observed_version_and_preserves_binding_gaps(self):
        table = json.loads((ROOT / SUPPORT_TABLE).read_text(encoding="utf-8"))
        report = {
            "schema_version": "1.0", "scope": "live", "transport": "https", "encoding": "JSON",
            "protocol_version": "2.1", "source_sha256": table["source_sha256"],
            "server": {"id": "cosmian-fixture", "product": "Cosmian KMS", "version": "5.28.0",
                       "configuration_fingerprint": "a" * 64},
            "observations": {
                tag: {"representation": representation, "http_checks": checks.copy()}
                for tag, (_, representation) in ROWS.items()
                for checks in (HTTP_CHECKS[tag],)
            },
        }
        report["observations"]["ObjectGroupMember"]["omitted_values"] = list(GROUP_MEMBER_VALUES)
        rows = [
            {"name": name, "declared_support": "vendor_extension",
             "obligations": [{"language": "rust", "api_level": "generic-ttlv"}]}
            for name, _ in ROWS.values()
        ]
        with tempfile.TemporaryDirectory() as work:
            path = Path(work) / "provider.json"
            path.write_text(json.dumps(report), encoding="utf-8")
            attach_evidence(ROOT, {"protocol_version": "2.0", "rows": rows}, [path])
            self.assertFalse(any("provider_fixture_results" in row for row in rows))
            inventory = {"protocol_version": "2.1", "rows": rows}
            attach_evidence(ROOT, inventory, [path])
            for row in rows:
                observation, = row["provider_fixture_results"]
                self.assertEqual(("2.1", "https", "JSON"),
                                 (observation["protocol_version"], observation["transport"], observation["encoding"]))
                self.assertNotIn("case_results", row["obligations"][0])

            report["observations"]["AttributeIndex"]["http_checks"]["read_back"] = 200
            path.write_text(json.dumps(report), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "unexpected AttributeIndex extension evidence"):
                attach_evidence(ROOT, inventory, [path])
