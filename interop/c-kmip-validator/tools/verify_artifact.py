#!/usr/bin/env python3
"""Verify the installed C ABI archive and its individual file digests."""
import hashlib
import json
from pathlib import Path
import sys

PIN = "9692057c9ade474fb0c4e0b7130539da368eb794"
ROOT = Path(__file__).resolve().parents[3]
LOCAL = ROOT / "interop/.local/c-live"
BUNDLE = LOCAL / f"kmipkit-c-1.0.0-{PIN[:12]}.tar.gz"
INSTALL = LOCAL / "install" / PIN


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def verify() -> None:
    data = json.loads((INSTALL / "provenance.json").read_text())
    if data.get("library_commit") != PIN or (INSTALL / "library-commit").read_text().strip() != PIN:
        raise SystemExit("C artifact commit provenance mismatch")
    for name, expected in data["files"].items():
        if sha256(INSTALL / name) != expected:
            raise SystemExit(f"C artifact digest mismatch: {name}")
    print(json.dumps({"library_commit": PIN, "artifact_sha256": sha256(BUNDLE)}, sort_keys=True))


if __name__ == "__main__":
    try:
        verify()
    except (OSError, KeyError, ValueError) as error:
        print(f"invalid C artifact provenance: {error}", file=sys.stderr)
        raise SystemExit(1)
