#!/usr/bin/env python3
"""Verify the installed wheel's pinned release provenance and digest."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import sys


PROJECT = Path(__file__).resolve().parents[1]
PIN = json.loads((PROJECT / "library-pin.json").read_text(encoding="utf-8"))
MANIFEST = PROJECT / "build" / "artifact-manifest.json"


def verify() -> dict[str, str]:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    wheel = Path(manifest["artifact_file"]).resolve(strict=True)
    actual = hashlib.sha256(wheel.read_bytes()).hexdigest()
    if (
        manifest.get("release_ref") != "release/1.0.0"
        or manifest.get("library_commit") != PIN["commit"]
        or manifest.get("artifact_sha256") != actual
        or manifest.get("package_version") != "1.0.0"
    ):
        raise ValueError("artifact provenance mismatch")
    from importlib.metadata import distribution

    package = distribution("kmipkit")
    if package.version != "1.0.0":
        raise ValueError("installed package version mismatch")
    return {"library_commit": PIN["commit"], "artifact_sha256": actual}


if __name__ == "__main__":
    try:
        print(json.dumps(verify(), sort_keys=True))
    except (OSError, ValueError, KeyError, ImportError, json.JSONDecodeError):
        print("pinned Python package verification failed", file=sys.stderr)
        raise SystemExit(1) from None
