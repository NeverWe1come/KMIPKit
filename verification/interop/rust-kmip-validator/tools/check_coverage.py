#!/usr/bin/env python3
"""Measure Rust plus subprocess CLI/controlled-peer coverage in an isolated target."""
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
MANIFEST = ROOT / "verification/interop/rust-kmip-validator/Cargo.toml"
TARGET = ROOT / "verification/interop/rust-kmip-validator/target/coverage-build"
OUTPUT = ROOT / "verification/interop/rust-kmip-validator/target/coverage.json"
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
env["KMIPKIT_REQUIRED_INTEROP_VALIDATOR"] = "rust"
env["KMIPKIT_VALIDATOR_BIN"] = str(TARGET / "debug/rust-kmip-validator")
commands = [
    ["cargo", "+1.94", "build", "--locked", "--manifest-path", str(MANIFEST)],
    ["cargo", "+1.94", "test", "--locked", "--manifest-path", str(MANIFEST), "--all-features"],
    [sys.executable, "-m", "unittest", "discover", "-s", "verification/interop/rust-kmip-validator/tests", "-p", "test_live.py", "-v"],
    [sys.executable, "-m", "unittest", "discover", "-s", "verification/interop/shared/tests",
     "-p", "test_incoming_validators.py", "-v"],
]
for command in commands:
    subprocess.run(command, cwd=ROOT, env=env, check=True)

tools = ROOT / "verification/interop/shared/tools"
runner = ["--runner", "rust", str(TARGET / "debug/kmip30-campaign")]
with tempfile.TemporaryDirectory(prefix="table-fixtures-", dir=TARGET) as directory:
    reports = Path(directory)
    for name in ("table_ttlv", "table_key_material", "table_typed_attributes", "table_managed_objects"):
        subprocess.run(
            [sys.executable, str(tools / f"{name}.py"), *runner,
             "--output", str(reports / f"{name}.json")],
            cwd=ROOT, env=env, check=True,
        )
    for name, options in (("decode", []), ("construct", ["--construct"])):
        subprocess.run(
            [sys.executable, str(tools / "oasis_ttlv.py"), "--version", "2.0", "--version", "2.1",
             "--require-complete", *options, *runner, "--output", str(reports / f"oasis-{name}.jsonl")],
            cwd=ROOT, env=env, check=True,
        )

# Regeneration and the model suite verify the generated dispatch table.
subprocess.run(
    ["cargo", "+1.94", "llvm-cov", "report", "--manifest-path", str(MANIFEST),
     "--ignore-filename-regex", r"local_cases_generated\.rs", "--json", "--output-path", str(OUTPUT)],
    cwd=ROOT, env=env, check=True,
)
data = json.loads(OUTPUT.read_text())
lines = data["data"][0]["totals"]["lines"]
print(f"Handwritten consumer line coverage: {lines['percent']:.2f}% ({lines['covered']}/{lines['count']})")
if lines["percent"] < 90:
    raise SystemExit("Rust consumer line coverage is below the required 90 percent")
