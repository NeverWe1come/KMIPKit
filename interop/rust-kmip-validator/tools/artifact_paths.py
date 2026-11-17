"""Return build paths scoped to the installed library source commit."""
import json
from pathlib import Path
import re


def installed_commit(root: Path) -> str:
    provenance = root / "interop/.local/rust-live/provenance.json"
    data = json.loads(provenance.read_bytes())
    commit = data.get("library_commit")
    if not isinstance(commit, str) or not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise ValueError("installed library provenance has no valid source commit")
    return commit


def rust_target_dir(root: Path) -> Path:
    return root / "interop/rust-kmip-validator/target" / f"consumer-{installed_commit(root)}"


def coverage_target_dir(root: Path) -> Path:
    return root / "interop/rust-kmip-validator/target" / f"coverage-{installed_commit(root)}"


def test_server_target_dir(root: Path) -> Path:
    return root / "interop/.local/rust-test-server" / f"harness-target-{installed_commit(root)}"


def validator_binary(root: Path) -> Path:
    return rust_target_dir(root) / "debug/rust-kmip-validator"


def test_server_binary(root: Path) -> Path:
    return test_server_target_dir(root) / "debug/kmipkit-interop-local-server"
