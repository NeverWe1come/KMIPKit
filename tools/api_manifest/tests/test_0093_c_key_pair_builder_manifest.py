"""Keep the Create Key Pair builder mapped across all supported bindings."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification/api/public-api.json"


class CreateKeyPairBuilderManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_c_abi_is_declared_without_claiming_four_language_readiness(self) -> None:
        capability = next(
            row for row in self.manifest["capabilities"]
            if row["capabilityId"] == "fluent_create_key_pair_c_abi"
        )
        self.assertEqual(capability["capabilityState"], "declared")
        self.assertEqual(capability["parityScope"], "c-only")
        declarations = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("capabilityId") == capability["capabilityId"]
        ]
        exports = {row["exportName"] for row in declarations}
        self.assertEqual(exports, set(capability["mappings"]["c"]))
        self.assertTrue(all(row["declarationState"] == "declared" for row in declarations))
        self.assertTrue(all(row["parityScope"] == "c-only" for row in declarations))

        parity = next(
            row for row in self.manifest["capabilities"]
            if row["capabilityId"] == "fluent_create_key_pair_parity"
        )
        self.assertEqual(parity["capabilityState"], "declared")
        self.assertEqual(parity["parityScope"], "four-language")
        self.assertEqual(
            set(parity["mappings"]), {"rust", "c", "java", "python"}
        )
        helper = next(
            row for row in self.manifest["publicDeclarations"]
            if row["declarationId"]
            == "rust-observed-kmipkit-0093-create-key-pair-detached"
        )
        self.assertEqual(helper["declarationState"], "declared")
        self.assertEqual(
            set(helper["languageMappings"]), {"rust", "c", "java", "python"}
        )

        exports = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("capabilityId") == parity["capabilityId"]
        }
        self.assertIn("org.kmipkit.fluent.CreateKeyPairBuilder", exports)
        self.assertIn("org.kmipkit.fluent.FluentRequest.createKeyPair", exports)
        self.assertIn("kmipkit.fluent.CreateKeyPairBuilder", exports)
        self.assertIn("kmipkit.fluent.Request.create_key_pair", exports)
        self.assertTrue(
            all(row["declarationState"] == "declared" for row in exports.values())
        )

    def test_create_and_validate_c_function_signatures_are_exact(self) -> None:
        functions = {
            row["id"]: row for row in self.manifest["functions"]
            if row["id"].startswith("create_key_pair_builder_")
        }
        create = functions["create_key_pair_builder_create"]["c"]
        self.assertEqual(
            create["parameters"][0]["type"],
            "kmipkit_create_key_pair_builder_t **",
        )
        validate = functions["create_key_pair_builder_validate"]["c"]
        self.assertEqual(validate["symbol"], "kmipkit_create_key_pair_builder_validate")
        self.assertEqual(validate["returnType"], "int32_t")
        self.assertEqual(validate["parameters"][0]["type"], "kmipkit_create_key_pair_builder_t *")


if __name__ == "__main__":
    unittest.main()
