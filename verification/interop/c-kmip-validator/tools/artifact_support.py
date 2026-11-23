"""Shared pin, paths, hashing, and provenance checks for the C artifact scripts."""

import hashlib
import json
import os
import re
from pathlib import Path

PIN = os.environ.get("KMIPKIT_INTEROP_LIBRARY_COMMIT", "992efad552ec7f5a8d269daf49b432b9c2f2339e")
if not re.fullmatch(r"[0-9a-f]{40}", PIN):
    raise ValueError("expected a full source commit")
ROOT = Path(__file__).resolve().parents[4]
LOCAL = ROOT / "verification/interop/.local/c-live"
BUNDLE = LOCAL / f"kmipkit-c-1.0.0-{PIN[:12]}.tar.gz"
INSTALL = LOCAL / "install" / PIN


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def verify_installed_artifact() -> dict[str, object]:
    data = json.loads((INSTALL / "provenance.json").read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError("C artifact provenance is not an object")
    if data.get("library_commit") != PIN:
        raise SystemExit("C artifact commit provenance mismatch")
    if (INSTALL / "library-commit").read_text(encoding="utf-8").strip() != PIN:
        raise SystemExit("C artifact source identity mismatch")
    files = data["files"]
    if not isinstance(files, dict):
        raise ValueError("C artifact file digests are invalid")
    for name, expected in files.items():
        if not isinstance(name, str) or not isinstance(expected, str):
            raise ValueError("C artifact file digest entry is invalid")
        if sha256(INSTALL / name) != expected:
            raise SystemExit(f"C artifact digest mismatch: {name}")
    return data
