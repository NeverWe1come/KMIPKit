"""Exercise the FFI coverage collector's Cargo report boundary."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(sys.platform == "linux", "The FFI coverage collector supports Linux only")
class FfiCoverageCollectorTests(unittest.TestCase):
    def run_collector(self, *, write_report=True):
        with tempfile.TemporaryDirectory(prefix="ffi coverage ") as temporary:
            root = Path(temporary)
            tools = root / "bin"
            tools.mkdir()
            invocation = root / "cargo-invocation.json"
            document = {
                "type": "llvm.coverage.json.export",
                "version": "3.0.1",
                "data": [{
                    "functions": [],
                    "files": [{
                        "filename": str(ROOT / "crates/kmipkit-ffi/src/lib.rs"),
                        "segments": [
                            [1, 1, 1, True, True, False],
                            [2, 1, 0, False, False, False],
                        ],
                        "summary": {"lines": {"count": 1, "covered": 1}},
                    }],
                }],
            }
            cargo_source = f"""import json, os, sys
from pathlib import Path
arguments = sys.argv[1:]
Path({str(invocation)!r}).write_text(json.dumps({{
    "arguments": arguments,
    "cargo_target_dir": os.environ.get("CARGO_TARGET_DIR"),
}}))
if {write_report!r}:
    output = Path(arguments[arguments.index("--output-path") + 1])
    output.write_text({json.dumps(document)!r})
print("cargo llvm-cov fixture")
"""
            cargo = tools / "cargo"
            cargo.write_text(f"#!{sys.executable}\n{cargo_source}")
            cargo.chmod(0o755)
            environment = dict(os.environ)
            environment.update(
                PATH=f"{tools}{os.pathsep}{environment['PATH']}",
                CARGO_TARGET_DIR=str(root / "target"),
            )
            report = root / "report"
            result = subprocess.run(
                ["bash", str(ROOT / "scripts/collect_ffi_coverage.sh"), str(report)],
                cwd=ROOT,
                env=environment,
                capture_output=True,
                text=True,
                check=False,
            )
            called = json.loads(invocation.read_text()) if invocation.exists() else {}
            raw_report_exists = (report / "coverage-raw.json").is_file()
            normalized_report_exists = (report / "coverage.json").is_file()
            return result, called, report, raw_report_exists, normalized_report_exists

    def test_normalizes_json_report_created_by_cargo_llvm_cov(self):
        result, called, report, raw_report_exists, normalized_report_exists = self.run_collector()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        arguments = called["arguments"]
        self.assertEqual(arguments[0], "llvm-cov")
        for required in ("-p", "kmipkit-ffi", "--all-targets", "--all-features",
                         "--locked", "--json", "--output-path"):
            self.assertIn(required, arguments)
        raw_index = arguments.index("--output-path") + 1
        self.assertEqual(arguments[raw_index], str(report / "coverage-raw.json"))
        self.assertEqual(called["cargo_target_dir"], str(report.parent / "target" / "ffi-coverage"))
        self.assertTrue(raw_report_exists)
        self.assertTrue(normalized_report_exists)

    def test_fails_when_cargo_reports_success_without_creating_json(self):
        result, called, report, raw_report_exists, normalized_report_exists = self.run_collector(
            write_report=False
        )
        self.assertEqual(called["arguments"][0], "llvm-cov")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("coverage-raw.json", result.stderr)
        self.assertFalse(raw_report_exists)
        self.assertFalse(normalized_report_exists)


if __name__ == "__main__":
    unittest.main()
