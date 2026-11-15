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
        defaults = next(
            function
            for function in manifest["functions"]
            if function["id"] == "extension_registry_limits_defaults"
        )
        defaults["java"]["returnType"] = "TtlvItemView"
        blocked_type = next(
            declaration
            for declaration in manifest["publicDeclarations"]
            if declaration["packageId"] == "java-kmipkit"
            and declaration["exportName"] == "org.kmipkit.ttlv.TtlvItemView"
        )
        blocked_type["declarationState"] = "blocked"

        with self.assertRaisesRegex(generate.ManifestError, "signature.*type.*TtlvItemView"):
            generate._validate_signature_type_readiness(
                manifest, manifest["publicDeclarations"]
            )
