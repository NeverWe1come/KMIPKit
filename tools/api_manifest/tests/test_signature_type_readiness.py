"""Ready API signatures cannot refer to blocked public value types."""

from __future__ import annotations

import copy
import json
from pathlib import Path
import unittest

from tools.api_manifest import generate


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"


class SignatureTypeReadinessTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))

    def test_ready_signatures_reject_blocked_result_types(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        rust_type = next(
            declaration
            for declaration in manifest["publicDeclarations"]
            if declaration["declarationId"] == "ExtensionRegistryLimits:rust"
        )
        rust_type["declarationState"] = "blocked"
        rust_type.pop("approvalSource", None)
        rust_type.pop("languageMappings", None)

        with self.assertRaisesRegex(generate.ManifestError, "signature.*type|blocked.*type"):
            generate._validate_v2_readiness(manifest, ROOT)

