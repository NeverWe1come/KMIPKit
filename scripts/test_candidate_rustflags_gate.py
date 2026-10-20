"""Verify Cargo flags cannot bypass the isolated PKCS#11 candidate build gate."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
TARGET_DIRECTORY = REPOSITORY_ROOT / "target" / "candidate-rustflags-gate-probe"


def main() -> int:
    probes = (
        (
            "RUSTFLAGS",
            ["cargo", "check", "--locked", "-p", "kmipkit-client", "--lib"],
            "candidate cfg must be enabled only through the isolated build opt-in",
            {"RUSTFLAGS": "--cfg kmipkit_pkcs11_candidate_test"},
        ),
        (
            "Cargo build.rustflags config",
            [
                "cargo",
                "--config",
                'build.rustflags=["--cfg","kmipkit_pkcs11_candidate_test"]',
                "check",
                "--locked",
                "-p",
                "kmipkit-ffi",
                "--lib",
            ],
            "candidate cfg must be enabled only through the isolated build opt-in",
            {},
        ),
    )
    for name, command, expected_message, overrides in probes:
        environment = os.environ.copy()
        environment.pop("KMIPKIT_PKCS11_CANDIDATE_TEST_BUILD", None)
        environment.pop("CARGO_ENCODED_RUSTFLAGS", None)
        environment.pop("RUSTFLAGS", None)
        environment.update(overrides)
        environment["CARGO_TARGET_DIR"] = str(TARGET_DIRECTORY / name.replace(" ", "-"))

        result = subprocess.run(
            command,
            cwd=REPOSITORY_ROOT,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
            timeout=300,
        )
        output = f"{result.stdout}\n{result.stderr}"
        if result.returncode == 0:
            print(f"Cargo accepted candidate cfg injection through {name}.", file=sys.stderr)
            return 1
        if expected_message not in output:
            print(output, file=sys.stderr)
            print(f"Cargo failed for a reason other than the {name} candidate cfg guard.", file=sys.stderr)
            return 1
        print(f"Candidate cfg injection through {name} was rejected.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
