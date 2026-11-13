#!/usr/bin/env python3
"""Compare stable Java route coverage while retaining per-build artifact hashes."""

import argparse
import json
from pathlib import Path
import re


_SHA256 = re.compile(r"[0-9a-f]{64}\Z")
VALIDATOR_ROOT = Path(__file__).resolve().parents[1]
REFERENCE_LEDGER = VALIDATOR_ROOT / "route-evidence-ledger.json"
GENERATED_LEDGER = VALIDATOR_ROOT / "build" / "route-evidence-ledger.json"


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


def refresh_reference(reference: dict, generated: dict) -> dict:
    """Refresh the checked-in matrix from a separately validated execution ledger."""
    _validated_artifact_digest(reference)
    _validated_artifact_digest(generated)
    if generated.get("all_invocations_passed") is not True:
        raise ValueError("cannot refresh the route reference from failed invocations")
    refreshed = dict(generated)
    # The installed artifact digest is build-specific and intentionally is not
    # part of the checked-in route coverage baseline.
    refreshed["artifact_sha256"] = reference["artifact_sha256"]
    return refreshed


def _write_reference_ledger(ledger: dict) -> None:
    """Write only to the checked-in ledger directly under this validator root."""
    root = VALIDATOR_ROOT.resolve(strict=True)
    if REFERENCE_LEDGER.name != "route-evidence-ledger.json":
        raise ValueError("route reference path has an unexpected filename")
    if REFERENCE_LEDGER.parent.resolve(strict=True) != root:
        raise ValueError("route reference path is outside the validator root")
    target = root / "route-evidence-ledger.json"
    try:
        resolved_target = REFERENCE_LEDGER.resolve(strict=False)
    except (OSError, RuntimeError) as error:
        raise ValueError("route reference path cannot be resolved safely") from error
    if REFERENCE_LEDGER.is_symlink() or resolved_target != target:
        raise ValueError("route reference path must not be a symlink or escape the validator root")
    target.write_text(json.dumps(ledger, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--refresh-reference",
        action="store_true",
        help="regenerate the checked-in ledger from the validated build ledger",
    )
    arguments = parser.parse_args()
    reference = json.loads(REFERENCE_LEDGER.read_text(encoding="utf-8"))
    generated = json.loads(GENERATED_LEDGER.read_text(encoding="utf-8"))
    if arguments.refresh_reference:
        refreshed = refresh_reference(reference, generated)
        _write_reference_ledger(refreshed)
        reference = refreshed
    compare_ledgers(reference, generated)
    print("Java route coverage matches the checked-in reference.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
