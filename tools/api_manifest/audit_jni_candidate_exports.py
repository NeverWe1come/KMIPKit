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
        expected_jni = set(fixture["candidateOnlyNativeExports"]["javaJni"])
        candidate_c_abi = set(fixture["candidateImplementationMappings"]["c"])
        exports = set(read_native_exports(arguments.binary))
        observed_jni = exports & expected_jni
        observed_candidate = exports & (expected_jni | candidate_c_abi)
    except (KeyError, OSError, ValueError, json.JSONDecodeError) as error:
        parser.error(f"could not audit the private JNI candidate inventory: {error}")

    if arguments.mode == "default" and observed_candidate:
        print(
            "default JNI library exposes owner-gated candidate symbols: "
            + ", ".join(sorted(observed_candidate)),
            file=sys.stderr,
        )
        return 1

    if arguments.mode == "candidate" and observed_jni != expected_jni:
        missing = sorted(expected_jni - observed_jni)
        print(
            "candidate JNI library is missing private harness symbols: "
            + ", ".join(missing),
            file=sys.stderr,
        )
        return 1

    print(
        f"{arguments.mode} JNI candidate export check passed: "
        f"{len(observed_jni)} candidate JNI symbols observed"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
