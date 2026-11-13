#!/usr/bin/env python3
"""Build an isolated public-API harness for the pinned local test-server.

The upstream CLI blocks its current-thread async executor while joining stdin;
this harness supplies the existing async shutdown receiver instead.
"""
import io
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "interop/rust-kmip-validator/tools"))
from artifact_paths import test_server_binary, test_server_target_dir

LOCAL = ROOT / "interop/.local/rust-test-server"
SOURCE = LOCAL / "release-source"
HARNESS = LOCAL / "harness"
PIN = "9a16cc90a12205101d22a38501a5e74c665df4dc"
LOCAL.mkdir(parents=True, exist_ok=True)
pin_marker = SOURCE / "library-commit"
if not pin_marker.is_file() or pin_marker.read_text().strip() != PIN:
    shutil.rmtree(SOURCE, ignore_errors=True)
    SOURCE.mkdir()
    archive = subprocess.check_output(["git", "-C", str(ROOT), "archive", PIN])
    with tarfile.open(fileobj=io.BytesIO(archive)) as source:
        source.extractall(SOURCE, filter="data")
    pin_marker.write_text(PIN + "\n")
(HARNESS / "src").mkdir(parents=True, exist_ok=True)
shutil.copyfile(ROOT / "interop/rust-kmip-validator/tools/server_harness.rs", HARNESS / "src/main.rs")
(HARNESS / "Cargo.toml").write_text('''[package]
name = "kmipkit-interop-local-server"
version = "0.0.0"
edition = "2024"
rust-version = "1.94"
[workspace]
[dependencies]
kmipkit-test-server = { path = "../release-source/crates/kmipkit-test-server" }
tokio = { version = "=1.53.3", features = ["macros", "rt", "sync"] }
[patch.crates-io]
aws-lc-sys = { path = "../release-source/vendor/aws-lc-sys-0.45.0" }
''')
target = test_server_target_dir(ROOT)
subprocess.run(["cargo", "+1.94", "build", "--manifest-path", str(HARNESS / "Cargo.toml"),
                "--target-dir", str(target)], check=True)
print(test_server_binary(ROOT))
