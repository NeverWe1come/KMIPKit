#!/usr/bin/env python3
"""Verify the installed C ABI archive and its individual file digests."""
import json
import sys

from artifact_support import BUNDLE, PIN, sha256, verify_installed_artifact


def verify() -> None:
    verify_installed_artifact()
    print(json.dumps({"library_commit": PIN, "artifact_sha256": sha256(BUNDLE)}, sort_keys=True))


if __name__ == "__main__":
    try:
        verify()
    except (OSError, KeyError, ValueError) as error:
        print(f"invalid C artifact provenance: {error}", file=sys.stderr)
        raise SystemExit(1)
