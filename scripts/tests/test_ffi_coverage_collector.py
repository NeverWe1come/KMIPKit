"""Exercise the collector's boundary with Cargo's reported executable paths."""

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
    def run_collector(self, layout, *, missing_consumer=False):
        with tempfile.TemporaryDirectory(prefix="ffi coverage ") as temporary:
            root = Path(temporary)
            target = root / "target" / "llvm-cov-target"
            tools = root / "bin"
            tools.mkdir()
            rustlib = root / "toolchain/lib/rustlib/x86_64-unknown-linux-gnu/bin"
            rustlib.mkdir(parents=True)
            names = ("kmipkit_ffi-012abc", "c_api_coverage-456def")
            if layout == "deps":
                binaries = [target / "debug/deps" / name for name in names]
            else:
                binaries = [
                    target / "debug/build/kmipkit-ffi" / name.split("-")[-1] / "out" / name
                    for name in names
                ]
            for index, binary in enumerate(binaries):
                binary.parent.mkdir(parents=True, exist_ok=True)
                if index == 1 and missing_consumer:
                    continue
                binary.write_text("fixture executable\n")
                binary.chmod(0o755)
            (target / "KMIPKit.profdata").write_bytes(b"profile fixture")
            log = "\n".join(
                f"     Running {label} ({binary})"
                for label, binary in zip(
                    ("unittests src/lib.rs", "tests/c_api_coverage.rs"), binaries
                )
            )
            calls = root / "llvm-arguments.json"
            document = {"type": "llvm.coverage.json.export", "version": "3.0.1",
                        "data": [{"functions": [], "files": [{
                "filename": str(ROOT / "crates/kmipkit-ffi/src/lib.rs"),
                "segments": [[1, 1, 1, True, True, False], [2, 1, 0, False, False, False]],
                "summary": {"lines": {"count": 1, "covered": 1}},
            }]}]}
            programs = {
                tools / "cargo": f"print({log!r})\n",
                tools / "rustc": (
                    "import sys\n"
                    f"print({str(root / 'toolchain')!r} if '--print' in sys.argv "
                    "else 'host: x86_64-unknown-linux-gnu')\n"
                ),
                rustlib / "llvm-cov": (
                    "import json,sys\nfrom pathlib import Path\n"
                    f"Path({str(calls)!r}).write_text(json.dumps(sys.argv[1:]))\n"
                    f"print({json.dumps(document)!r})\n"
                ),
            }
            for path, source in programs.items():
                path.write_text(f"#!{sys.executable}\n{source}")
                path.chmod(0o755)
            environment = dict(os.environ)
            environment.update(PATH=f"{tools}{os.pathsep}{environment['PATH']}",
                               CARGO_TARGET_DIR=str(root / "target"))
            report = root / "report"
            result = subprocess.run(
                ["bash", str(ROOT / "scripts/collect_ffi_coverage.sh"), str(report)],
                cwd=ROOT, env=environment, capture_output=True, text=True, check=False,
            )
            arguments = json.loads(calls.read_text()) if calls.exists() else []
            return result, arguments, [str(binary) for binary in binaries], (report / "coverage.json").exists()

    def test_exports_reported_executables_from_deps_directory(self):
        result, arguments, binaries, report_exists = self.run_collector("deps")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual([arguments[index + 1] for index, value in enumerate(arguments)
                          if value == "-object"], binaries)
        self.assertTrue(report_exists)

    def test_exports_reported_executables_from_rust_1100_build_directory(self):
        result, arguments, binaries, report_exists = self.run_collector("build")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual([arguments[index + 1] for index, value in enumerate(arguments)
                          if value == "-object"], binaries)
        self.assertTrue(report_exists)

    def test_rejects_missing_reported_consumer_before_export(self):
        result, arguments, _, report_exists = self.run_collector("build", missing_consumer=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("test executable is missing", result.stderr)
        self.assertEqual(arguments, [])
        self.assertFalse(report_exists)


if __name__ == "__main__":
    unittest.main()
