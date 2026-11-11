#!/usr/bin/env python3
"""Measure Rust plus subprocess CLI/controlled-peer coverage in an isolated target."""
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "interop/rust-kmip-validator/Cargo.toml"
TARGET = ROOT / "interop/rust-kmip-validator/target/coverage-build"
OUTPUT = ROOT / "interop/rust-kmip-validator/target/coverage.json"
env = os.environ.copy()
env["CARGO_TARGET_DIR"] = str(TARGET)
settings = subprocess.check_output(["cargo", "+1.94", "llvm-cov", "show-env", "--manifest-path", str(MANIFEST), "--sh"], env=env, text=True)
for line in settings.splitlines():
    if line.startswith("export "):
        key, value = line[7:].split("=", 1)
        env[key] = shlex.split(value)[0]
# Only this dedicated coverage directory is cleared; other builds stay intact.
for profile in TARGET.glob("*.profraw"):
    profile.unlink()
env["KMIPKIT_VALIDATOR_BIN"] = str(TARGET / "debug/rust-kmip-validator")
commands = [
    ["cargo", "+1.94", "build", "--locked", "--manifest-path", str(MANIFEST)],
    ["cargo", "+1.94", "test", "--locked", "--manifest-path", str(MANIFEST), "--all-features"],
    [sys.executable, "-m", "unittest", "discover", "-s", "interop/rust-kmip-validator/tests", "-p", "test_live.py", "-v"],
    # This exhaustive generated lookup table is verified against every local
    # catalog case and regeneration checks; it is excluded from handwritten-code coverage.
    ["cargo", "+1.94", "llvm-cov", "report", "--manifest-path", str(MANIFEST), "--ignore-filename-regex", r"local_cases_generated\.rs", "--json", "--output-path", str(OUTPUT)],
]
for command in commands:
    subprocess.run(command, cwd=ROOT, env=env, check=True)
data = json.loads(OUTPUT.read_text())
lines = data["data"][0]["totals"]["lines"]
print(f"Handwritten consumer line coverage: {lines['percent']:.2f}% ({lines['covered']}/{lines['count']})")
if lines["percent"] < 90:
    raise SystemExit("Rust consumer line coverage is below the required 90 percent")
