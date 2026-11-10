#!/usr/bin/env python3
"""Verify provenance and every installed package/supplement against the bundle."""
import hashlib
import io
import json
from pathlib import Path
import sys
import tarfile

PIN = "fdafef8a3eb0c7b4b18d6433b78fde4151493ceb"
ROOT = Path(__file__).resolve().parents[3]
LOCAL = ROOT / "interop/.local/rust-live"


def verify():
    provenance = json.loads((LOCAL / "provenance.json").read_text())
    bundle = LOCAL / "kmipkit-rust-1.0.0.tar.gz"
    if provenance.get("library_commit") != PIN or provenance.get("artifact_sha256") != hashlib.sha256(bundle.read_bytes()).hexdigest():
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
    return provenance


if __name__ == "__main__":
    try:
        result = verify()
    except (OSError, ValueError, KeyError, tarfile.TarError):
        raise SystemExit("pinned Rust package verification failed; reinstall the artifact") from None
    if "--quiet" not in sys.argv:
        print(json.dumps(result, sort_keys=True))
