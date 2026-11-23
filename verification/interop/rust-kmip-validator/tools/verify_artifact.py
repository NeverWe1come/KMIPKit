#!/usr/bin/env python3
"""Verify provenance and every installed package/supplement against the bundle."""
import hashlib
import io
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import tarfile

ROOT = Path(__file__).resolve().parents[4]
LOCAL = ROOT / "verification/interop/.local/rust-live"


def source_commit():
    selected = os.environ.get("KMIPKIT_INTEROP_LIBRARY_COMMIT")
    if selected is not None and not re.fullmatch(r"[0-9a-f]{40}", selected):
        raise ValueError("expected a full source commit")
    return subprocess.check_output(
        ["git", "-C", str(ROOT), "rev-parse", "--verify", f"{selected or 'HEAD'}^{{commit}}"], text=True,
    ).strip()


def verify():
    provenance = json.loads((LOCAL / "provenance.json").read_text())
    bundle = LOCAL / "kmipkit-rust-1.0.0.tar.gz"
    if provenance.get("library_commit") != source_commit() or provenance.get("artifact_sha256") != hashlib.sha256(bundle.read_bytes()).hexdigest():
        raise ValueError("artifact identity mismatch")
    expected = {}
    with tarfile.open(bundle) as archive:
        for member in archive:
            if not member.isfile():
                continue
            value = archive.extractfile(member).read()
            expected[member.name] = hashlib.sha256(value).digest()
            if member.name.endswith(".crate"):
                with tarfile.open(fileobj=io.BytesIO(value)) as package:
                    for entry in package:
                        if entry.isfile():
                            expected["crates/" + entry.name] = hashlib.sha256(package.extractfile(entry).read()).digest()
    installed = LOCAL / "install"
    actual = {str(path.relative_to(installed)) for path in installed.rglob("*") if path.is_file()}
    if actual != set(expected):
        raise ValueError("installed package inventory mismatch")
    for name, digest in expected.items():
        if hashlib.sha256((installed / name).read_bytes()).digest() != digest:
            raise ValueError("installed package bytes mismatch")
    if (installed / "library-commit").read_text().strip() != provenance["library_commit"]:
        raise ValueError("installed package source mismatch")
    return provenance


if __name__ == "__main__":
    try:
        result = verify()
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError, tarfile.TarError):
        raise SystemExit("current Rust package verification failed; reinstall the artifact") from None
    if "--quiet" not in sys.argv:
        print(json.dumps(result, sort_keys=True))
