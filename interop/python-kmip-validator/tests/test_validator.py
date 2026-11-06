import json
import tempfile
import unittest
from pathlib import Path

from kmipkit_validator.cli import load_capabilities, load_catalog, run_execution
from kmipkit_validator.driver import EquivalenceVerifier, NormalizedResponse, RecipeRunner
from kmipkit_validator.isolation import AttemptContext
from kmipkit_validator.matcher import CapabilityMatcher
from kmipkit_validator.models import ServerCapabilities, TestCase


class TestKmipValidator(unittest.TestCase):
    def test_attempt_context_uniqueness(self):
        ctx1 = AttemptContext("run-1", "Python", "TTLV", "L2")
        ctx2 = AttemptContext("run-1", "Python", "TTLV", "L2")

        name1 = ctx1.generate_unique_name("key")
        name2 = ctx2.generate_unique_name("key")

        self.assertNotEqual(name1, name2)
        self.assertTrue(name1.startswith("key-Python-TTLV-L2-"))

    def test_capability_matcher(self):
        caps = ServerCapabilities(
            server_id="test-server",
            supported_operations={"Create", "Get", "Destroy"},
            supported_objects={"SymmetricKey"},
            supported_attributes={"CryptographicAlgorithm"},
        )
        matcher = CapabilityMatcher(caps)

        tc_ok = TestCase(case_id="TC-1", category="core", operation="Create", object_type="SymmetricKey")
        self.assertTrue(matcher.is_supported(tc_ok))

        tc_bad_op = TestCase(case_id="TC-2", category="core", operation="Sign", object_type="SymmetricKey")
        self.assertFalse(matcher.is_supported(tc_bad_op))

        tc_bad_obj = TestCase(case_id="TC-3", category="core", operation="Create", object_type="Certificate")
        self.assertFalse(matcher.is_supported(tc_bad_obj))

    def test_parity_cosmian(self):
        catalog_path = Path(__file__).resolve().parent.parent.parent / "shared/catalog/kmipkit-1.0-test-catalog.json"
        profile_path = Path(__file__).resolve().parent.parent.parent / "shared/profiles/cosmian-kms.json"

        caps = load_capabilities(profile_path)
        catalog = load_catalog(catalog_path)
        matcher = CapabilityMatcher(caps)

        supported = sum(1 for tc in catalog if matcher.is_supported(tc))
        unsupported = len(catalog) - supported

        self.assertEqual(len(catalog), 200)
        self.assertEqual(supported, 117)
        self.assertEqual(unsupported, 83)

    def test_cli_run_cosmian(self):
        catalog_path = Path(__file__).resolve().parent.parent.parent / "shared/catalog/kmipkit-1.0-test-catalog.json"
        profile_path = Path(__file__).resolve().parent.parent.parent / "shared/profiles/cosmian-kms.json"

        caps = load_capabilities(profile_path)
        catalog = load_catalog(catalog_path)
        matcher = CapabilityMatcher(caps)

        with tempfile.TemporaryDirectory() as tmpdir:
            out_dir = Path(tmpdir)
            ret = run_execution(caps, catalog, matcher, out_dir)
            self.assertEqual(ret, 0)

            self.assertTrue((out_dir / "report.md").is_file())
            self.assertTrue((out_dir / "report.html").is_file())
            self.assertTrue((out_dir / "results.json").is_file())

            results_data = json.loads((out_dir / "results.json").read_text(encoding="utf-8"))  # NOSONAR
            self.assertEqual(results_data["summary"]["passed"], 117)
            self.assertEqual(results_data["summary"]["unsupported_by_server"], 83)

    def test_equivalence_verifier(self):
        r1 = NormalizedResponse(status="Success", object_type="SymmetricKey", sanitized_payload_hex="deadbeef")
        r2 = NormalizedResponse(status="Success", object_type="SymmetricKey", sanitized_payload_hex="deadbeef")
        r_diff = NormalizedResponse(status="OperationFailed", result_reason="Fail", object_type="SymmetricKey")

        res_ok = EquivalenceVerifier.verify_equivalence({"HighLevel": r1, "Typed": r2})
        self.assertTrue(res_ok.is_equivalent)
        self.assertEqual(len(res_ok.discrepancies), 0)

        res_bad = EquivalenceVerifier.verify_equivalence({"HighLevel": r1, "LowLevel": r_diff})
        self.assertFalse(res_bad.is_equivalent)
        self.assertEqual(len(res_bad.discrepancies), 3)

    def test_recipe_runner_guarantees_teardown(self):
        teardown_called = []

        def teardown():
            teardown_called.append(True)
            return True

        def failing_target():
            raise ValueError("Target failed")

        with self.assertRaises(ValueError):
            RecipeRunner.execute_with_guaranteed_teardown(lambda: True, failing_target, teardown)

        self.assertEqual(len(teardown_called), 1)


if __name__ == "__main__":
    unittest.main()
