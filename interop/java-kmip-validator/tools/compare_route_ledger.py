#!/usr/bin/env python3
"""Compare stable Java route coverage while retaining per-build artifact hashes."""

import argparse
import json
from pathlib import Path
import re


_SHA256 = re.compile(r"[0-9a-f]{64}\Z")


def _validated_artifact_digest(ledger: dict) -> str:
    artifact_sha256 = ledger.get("artifact_sha256")
    if not isinstance(artifact_sha256, str) or not _SHA256.fullmatch(artifact_sha256):
        raise ValueError("route ledger has an invalid artifact digest")
    return artifact_sha256


def _coverage_projection(ledger: dict) -> dict:
    return {key: value for key, value in ledger.items() if key != "artifact_sha256"}


def compare_ledgers(reference: dict, generated: dict) -> bool:
    """Require equal coverage and provenance while allowing build-specific artifacts."""
    _validated_artifact_digest(reference)
    _validated_artifact_digest(generated)
    if _coverage_projection(reference) != _coverage_projection(generated):
        raise ValueError("generated route ledger differs from the checked-in coverage reference")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--generated", type=Path, required=True)
    args = parser.parse_args()
    reference = json.loads(args.reference.read_text(encoding="utf-8"))
    generated = json.loads(args.generated.read_text(encoding="utf-8"))
    compare_ledgers(reference, generated)
    print("Java route coverage matches the checked-in reference.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
