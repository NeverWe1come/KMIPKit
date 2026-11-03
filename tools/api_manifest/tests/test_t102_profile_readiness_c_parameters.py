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

        release = self.declarations["T102:c:kmipkit_profile_readiness_release"]
        self.assertNotIn("cSignature", release)
        handle = next(row for row in self.manifest["types"]
                      if row["id"] == "ProfileReadinessHandle")
        self.assertEqual(handle["c"]["ownership"], "library-owned")
        self.assertEqual(handle["c"]["releaseFunction"], "kmipkit_profile_readiness_release")

    def test_versioned_evidence_records_can_embed_the_approved_byte_span_carrier(self) -> None:
        catalog = generate._load_json(ROOT, generate.CATALOG_PATH, generate.MAX_CATALOG_BYTES)
        generate._validate_v2_manifest(
            self.manifest, self.schema, catalog, ROOT, require_complete=False
        )

    def test_generated_header_emits_approved_span_before_evidence_record_layout(self) -> None:
        header = generate.render_c_header(self.manifest)
        span_layout = "struct kmipkit_byte_span_v1_t {"
        record_layout = "struct kmipkit_profile_evidence_record_v1_t {"
        self.assertIn(span_layout, header)
        self.assertIn(record_layout, header)
        self.assertLess(header.index(span_layout), header.index(record_layout))


if __name__ == "__main__":
    unittest.main()
