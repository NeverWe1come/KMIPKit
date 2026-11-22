#!/usr/bin/env python3
"""Build an isolated public-API harness for the current local test-server.

The upstream CLI blocks its current-thread async executor while joining stdin;
this harness supplies the existing async shutdown receiver instead.
"""
from pathlib import Path
import shutil
import subprocess
ROOT = Path(__file__).resolve().parents[4]
LOCAL = ROOT / "verification/interop/.local/rust-test-server"
HARNESS = LOCAL / "harness"
(HARNESS / "src").mkdir(parents=True, exist_ok=True)
shutil.copyfile(ROOT / "verification/interop/rust-kmip-validator/tools/server_harness.rs", HARNESS / "src/main.rs")
(HARNESS / "Cargo.toml").write_text('''[package]
name = "kmipkit-interop-local-server"
version = "0.0.0"
edition = "2024"
rust-version = "1.94"
[workspace]
[dependencies]
kmipkit-test-server = { path = "../../../../../crates/kmipkit-test-server" }
tokio = { version = "=1.53.3", features = ["macros", "rt", "sync"] }
''')
subprocess.run(["cargo", "+1.94", "build", "--manifest-path", str(HARNESS / "Cargo.toml")], check=True)
print(HARNESS / "target/debug/kmipkit-interop-local-server")
