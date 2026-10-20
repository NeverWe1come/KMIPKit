"""Verify Cargo flags cannot bypass the isolated PKCS#11 candidate build gate."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
TARGET_DIRECTORY = REPOSITORY_ROOT / "target" / "candidate-rustflags-gate-probe"


def main() -> int:
    environment = os.environ.copy()
    environment.pop("KMIPKIT_PKCS11_CANDIDATE_TEST_BUILD", None)
    environment.pop("CARGO_ENCODED_RUSTFLAGS", None)
    environment["RUSTFLAGS"] = "--cfg kmipkit_pkcs11_candidate_test"
    environment["CARGO_TARGET_DIR"] = str(TARGET_DIRECTORY)

    result = subprocess.run(
        ["cargo", "check", "--locked", "-p", "kmipkit-client", "--lib"],
        cwd=REPOSITORY_ROOT,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
        timeout=300,
    )
    output = f"{result.stdout}\n{result.stderr}"
    if result.returncode == 0:
        print("Cargo accepted a direct candidate cfg without the isolated-build opt-in.", file=sys.stderr)
        return 1
    if "candidate cfg must be enabled only through the isolated build opt-in" not in output:
        print(output, file=sys.stderr)
        print("Cargo failed for a reason other than the candidate cfg guard.", file=sys.stderr)
        return 1
    print("Direct candidate cfg injection was rejected by the build gate.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
