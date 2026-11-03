"""Exact T102 C ABI parameter and ownership contract."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest import generate

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
SCHEMA_PATH = ROOT / "specification" / "api" / "public-api.schema.json"


class T102ProfileReadinessCParameterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }

    def test_readiness_functions_use_schema_valid_fixed_width_ownership_shapes(self) -> None:
        function_ids = (
            "T102:c:kmipkit_profile_readiness_evaluate_v1",
            "T102:c:kmipkit_profile_readiness_view_get_v1",
            "T102:c:kmipkit_profile_readiness_blocker_at_v1",
            "T102:c:kmipkit_profile_catalog_table_count_v1",
            "T102:c:kmipkit_profile_catalog_row_at_v1",
            "T102:c:kmipkit_profile_catalog_relation_count_v1",
            "T102:c:kmipkit_profile_catalog_relation_at_v1",
            "T102:c:kmipkit_profile_readiness_release",
        )
        for declaration_id in function_ids:
            with self.subTest(declaration_id=declaration_id):
                signature = self.declarations[declaration_id]["cSignature"]
                errors: list[str] = []
                self.assertTrue(generate._schema_matches(
                    signature,
                    self.schema["$defs"]["cSignature"],
                    self.schema,
                    declaration_id,
                    errors,
                ), errors)

        evaluate = self.declarations[function_ids[0]]["cSignature"]
        parameters = {item["name"]: item for item in evaluate["parameters"]}
        self.assertEqual(parameters["profile_id"]["kind"], "input-value-pointer")
        self.assertEqual(parameters["profile_id"]["type"], "const kmipkit_byte_span_v1_t *")
        self.assertEqual(parameters["profile_id"]["ownership"], "borrowed")
        self.assertEqual(parameters["records"]["kind"], "input-array")
        self.assertEqual(
            parameters["records"]["elementType"], "kmipkit_profile_evidence_record_v1_t"
        )
        self.assertEqual(parameters["records"]["ownership"], "borrowed")
        self.assertEqual(parameters["out_readiness"]["kind"], "output")
        self.assertEqual(parameters["out_readiness"]["ownership"], "library-owned")
        self.assertTrue(parameters["out_readiness"]["nullable"])

        release = self.declarations[function_ids[-1]]["cSignature"]
        self.assertEqual(release["returnType"], "void")
        self.assertEqual(release["parameters"], [{
            "name": "readiness",
            "kind": "handle",
            "type": "kmipkit_profile_readiness_t *",
            "ownership": "consumed",
        }])


if __name__ == "__main__":
    unittest.main()
