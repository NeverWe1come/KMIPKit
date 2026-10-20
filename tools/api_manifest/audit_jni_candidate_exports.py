"""Audit owner-gated PKCS#11 JNI exports without publishing them in the API manifest."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

from audit_binding_exports import read_native_exports


ROOT = Path(__file__).resolve().parents[2]
PRIVATE_INVENTORY = ROOT / "tests" / "fixtures" / "api" / "pkcs11-parity.json"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=("default", "candidate"), required=True)
    parser.add_argument("--binary", type=Path, required=True)
    arguments = parser.parse_args()

    if not arguments.binary.is_file():
        parser.error(f"JNI library does not exist: {arguments.binary}")

    try:
        fixture = json.loads(PRIVATE_INVENTORY.read_text(encoding="utf-8"))
        expected = set(fixture["candidateOnlyNativeExports"]["javaJni"])
        observed = set(read_native_exports(arguments.binary)) & expected
    except (KeyError, OSError, ValueError, json.JSONDecodeError) as error:
        parser.error(f"could not audit the private JNI candidate inventory: {error}")

    if arguments.mode == "default" and observed:
        print(
            "default JNI library exposes owner-gated candidate symbols: "
            + ", ".join(sorted(observed)),
            file=sys.stderr,
        )
        return 1

    if arguments.mode == "candidate" and observed != expected:
        missing = sorted(expected - observed)
        print(
            "candidate JNI library is missing private harness symbols: "
            + ", ".join(missing),
            file=sys.stderr,
        )
        return 1

    print(
        f"{arguments.mode} JNI candidate export check passed: "
        f"{len(observed)} candidate symbols observed"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
