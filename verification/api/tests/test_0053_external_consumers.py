"""Contract checks for the standalone KMIPKIT-0053 package consumers."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
CONSUMERS = ROOT / "tests" / "consumers" / "0053-parity"


class ExternalConsumerProjectTests(unittest.TestCase):
    def test_reproducible_script_runs_all_four_clean_consumers(self) -> None:
        for script_name in ("verify.ps1", "verify.sh"):
            script = CONSUMERS / script_name
            self.assertTrue(script.is_file(), f"{script_name} is missing")
            source = script.read_text(encoding="utf-8").replace("\\", "/")
            for language in ("rust", "c", "java", "python"):
                with self.subTest(script=script_name, language=language):
                    self.assertIn(f"{language}/", source)
                    self.assertTrue((CONSUMERS / language).is_dir())
            self.assertIn("cargo +1.94 run", source)

    def test_each_consumer_has_a_minimal_public_package_entrypoint(self) -> None:
        expected = {
            "rust": ("Cargo.toml", "src/main.rs"),
            "c": ("CMakeLists.txt", "main.c"),
            "java": ("pom.xml", "src/main/java/Consumer.java"),
            "python": ("pyproject.toml", "main.py"),
        }
        for language, files in expected.items():
            with self.subTest(language=language):
                for relative in files:
                    self.assertTrue(
                        (CONSUMERS / language / relative).is_file(),
                        f"{language} consumer is missing {relative}",
                    )


if __name__ == "__main__":
    unittest.main()
