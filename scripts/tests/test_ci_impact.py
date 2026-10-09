"""RED contracts for exact-tree CI impact classification."""

from __future__ import annotations

import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "ci_impact.py"
IMPACT = None
LOAD_ERROR = None
if SCRIPT.is_file():
    try:
        spec = importlib.util.spec_from_file_location("ci_impact", SCRIPT)
        if spec is None or spec.loader is None:
            raise ImportError("impact classifier has no importable module loader")
        IMPACT = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(IMPACT)
    except Exception as error:  # Keep RED failures actionable when implementation is absent.
        LOAD_ERROR = error


class CiImpactTests(unittest.TestCase):
    def require_impact(self):
        self.assertIsNotNone(
            IMPACT,
            f"ci_impact.py must provide the tested impact contract; load error: {LOAD_ERROR}",
        )
        return IMPACT

    def classify(self, *changes):
        impact = self.require_impact()
        return impact.classify_changes(
            list(changes),
            base_sha="a" * 40,
            merge_sha="b" * 40,
        )

    def test_documentation_only_paths_select_only_docs_contracts(self) -> None:
        plan = self.classify({"status": "M", "paths": ["docs/development/testing.md"]})

        self.assertEqual(["documentation"], plan["classes"])
        self.assertEqual(["docs-contracts"], plan["selected_jobs"])
        self.assertEqual([], plan["coverage_scopes"])
        self.assertFalse(plan["full"])

    def test_empty_diff_selects_documentation_contracts_and_is_valid(self) -> None:
        impact = self.require_impact()
        plan = impact.classify_changes([], base_sha="a" * 40, merge_sha="b" * 40)

        self.assertFalse(plan["full"])
        self.assertEqual(["documentation"], plan["classes"])
        self.assertEqual(["docs-contracts"], plan["selected_jobs"])
        self.assertEqual([], plan["coverage_scopes"])
        self.assertTrue(impact.validate_plan(plan)[0])

    def test_root_and_feature_markdown_are_documentation(self) -> None:
        for path in ("README.md", "specs/015-selective-ci/spec.md", "changelog.d/001-ci.md"):
            with self.subTest(path=path):
                plan = self.classify({"status": "M", "paths": [path]})
                self.assertEqual(["documentation"], plan["classes"])
                self.assertFalse(plan["full"])

    def test_each_isolated_component_has_only_its_checks_and_coverage(self) -> None:
        cases = {
            "bindings/java/src/main/java/org/kmipkit/Client.java": (
                "java",
                {"language-java", "coverage-java", "coverage-gate"},
                {"java"},
            ),
            "bindings/python/src/kmipkit/client.py": (
                "python",
                {"language-python", "coverage-python", "coverage-gate"},
                {"python"},
            ),
            "bindings/c/tests/extension_registry.c": (
                "c-consumer",
                {"language-c", "ffi-sanitizer-c", "coverage", "coverage-gate"},
                {"rust", "ffi-c"},
            ),
            "bindings/java/native/kmipkit_jni.cpp": (
                "jni",
                {"language-java", "ffi-sanitizer-jni", "coverage-jni", "coverage-gate"},
                {"jni"},
            ),
        }
        for path, (path_class, expected_jobs, expected_scopes) in cases.items():
            with self.subTest(path=path):
                plan = self.classify({"status": "M", "paths": [path]})
                self.assertEqual([path_class], plan["classes"])
                self.assertEqual(expected_jobs, set(plan["selected_jobs"]))
                self.assertEqual(expected_scopes, set(plan["coverage_scopes"]))
                self.assertFalse(plan["full"])

    def test_mixed_component_paths_union_jobs_and_scopes(self) -> None:
        plan = self.classify(
            {"status": "M", "paths": ["docs/development/testing.md"]},
            {"status": "M", "paths": ["bindings/python/tests/test_client.py"]},
            {"status": "M", "paths": ["bindings/java/src/main/java/org/kmipkit/Client.java"]},
        )

        self.assertEqual(["documentation", "java", "python"], plan["classes"])
        self.assertEqual(
            {"docs-contracts", "language-java", "coverage-java", "language-python", "coverage-python", "coverage-gate"},
            set(plan["selected_jobs"]),
        )
        self.assertEqual({"java", "python"}, set(plan["coverage_scopes"]))
        self.assertFalse(plan["full"])

    def test_ci_shared_manifest_generator_normative_and_unknown_paths_force_full(self) -> None:
        paths = (
            ".github/workflows/ci.yml",
            "scripts/coverage_gate.py",
            "Cargo.toml",
            "Cargo.lock",
            "crates/kmipkit-protocol/src/lib.rs",
            "bindings/java/pom.xml",
            "bindings/python/pyproject.toml",
            "bindings/c/CMakeLists.txt",
            "tools/api_manifest/generate.py",
            "specification/catalog/kmip-2.1.json",
            "new-area/unclassified.data",
        )
        for path in paths:
            with self.subTest(path=path):
                plan = self.classify({"status": "M", "paths": [path]})
                self.assertTrue(plan["full"])
                self.assertEqual(set(IMPACT.PULL_REQUEST_JOB_IDS), set(plan["selected_jobs"]))
                self.assertEqual(set(IMPACT.ALL_COVERAGE_SCOPES), set(plan["coverage_scopes"]))

    def test_rename_copy_and_delete_retain_every_relevant_path(self) -> None:
        cases = (
            {"status": "R100", "paths": ["docs/old.md", "bindings/python/src/kmipkit/new.py"]},
            {"status": "C100", "paths": ["docs/source.md", "bindings/java/src/main/java/org/kmipkit/Copy.java"]},
            {"status": "D", "paths": ["bindings/c/tests/removed.c"]},
        )
        plan = self.classify(*cases)

        self.assertTrue({"documentation", "python", "java", "c-consumer"}.issubset(plan["classes"]))
        self.assertEqual(
            {
                "docs-contracts",
                "language-python",
                "coverage-python",
                "language-java",
                "coverage-java",
                "language-c",
                "ffi-sanitizer-c",
                "coverage",
                "coverage-gate",
            },
            set(plan["selected_jobs"]),
        )
        self.assertFalse(plan["full"])

    def test_name_status_parser_uses_nul_records_for_renames_and_newlines(self) -> None:
        impact = self.require_impact()
        parsed = impact.parse_name_status_z(
            b"M\0docs/a file.md\0R100\0docs/old.md\0bindings/python/new.py\0D\0bindings/c/old.c\0"
        )

        self.assertEqual(
            [
                {"status": "M", "paths": ["docs/a file.md"]},
                {"status": "R100", "paths": ["docs/old.md", "bindings/python/new.py"]},
                {"status": "D", "paths": ["bindings/c/old.c"]},
            ],
            parsed,
        )

    def test_malformed_name_status_records_are_rejected(self) -> None:
        impact = self.require_impact()
        for output in (b"M\0", b"R100\0old\0", b"Q\0path\0", b"M\0path\0trailing"):
            with self.subTest(output=output), self.assertRaises(impact.ImpactPlanError):
                impact.parse_name_status_z(output)

    def test_invalid_and_traversing_paths_force_full(self) -> None:
        for path in ("../outside.md", "/absolute/file.md", "bindings\\python\\src\\x.py"):
            with self.subTest(path=path):
                plan = self.classify({"status": "M", "paths": [path]})
                self.assertTrue(plan["full"])
                self.assertTrue(plan["fallback"])

    def test_plan_is_deterministic_and_binds_exact_commit_shas(self) -> None:
        change = {"status": "M", "paths": ["bindings/python/src/kmipkit/client.py"]}
        first = self.classify(change)
        second = self.classify(change)

        self.assertEqual(first, second)
        self.assertEqual("a" * 40, first["base_sha"])
        self.assertEqual("b" * 40, first["merge_sha"])
        self.assertEqual(1, first["schema_version"])

    @staticmethod
    def git(root: Path, *arguments: str) -> str:
        result = subprocess.run(
            ["git", "-C", str(root), *arguments],
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip()

    def test_classifier_uses_exact_base_and_merge_trees(self) -> None:
        impact = self.require_impact()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.git(root, "init", "--quiet")
            self.git(root, "config", "user.name", "CI Contract")
            self.git(root, "config", "user.email", "ci-contract@example.invalid")
            (root / "README.md").write_text("base\n", encoding="utf-8")
            self.git(root, "add", "README.md")
            self.git(root, "commit", "--quiet", "-m", "base")
            base = self.git(root, "rev-parse", "HEAD")
            (root / "bindings/python/src/kmipkit").mkdir(parents=True)
            (root / "bindings/python/src/kmipkit/client.py").write_text("value = 1\n", encoding="utf-8")
            self.git(root, "add", ".")
            self.git(root, "commit", "--quiet", "-m", "python change")
            merge = self.git(root, "rev-parse", "HEAD")

            plan = impact.classify_diff(root, base, merge)

        self.assertEqual({"python"}, set(plan["classes"]))
        self.assertEqual({"language-python", "coverage-python", "coverage-gate"}, set(plan["selected_jobs"]))
        self.assertEqual(base, plan["base_sha"])
        self.assertEqual(merge, plan["merge_sha"])

    def test_diff_failures_return_explicit_full_fallback(self) -> None:
        impact = self.require_impact()
        with tempfile.TemporaryDirectory() as temporary:
            plan = impact.classify_diff(Path(temporary), "x" * 40, "y" * 40)

        self.assertTrue(plan["full"])
        self.assertTrue(plan["fallback"])
        self.assertEqual(set(IMPACT.PULL_REQUEST_JOB_IDS), set(plan["selected_jobs"]))
        self.assertTrue(plan["reason"])


if __name__ == "__main__":
    unittest.main()
