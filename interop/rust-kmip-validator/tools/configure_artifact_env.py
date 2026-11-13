#!/usr/bin/env python3
"""Emit environment settings for the installed library's isolated build paths."""
import argparse
import os
from pathlib import Path
import shlex
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "interop/rust-kmip-validator/tools"))
from artifact_paths import rust_target_dir, test_server_binary, validator_binary


def assignments(root: Path) -> dict[str, str]:
    return {
        "CARGO_TARGET_DIR": str(rust_target_dir(root)),
        "KMIPKIT_VALIDATOR_BIN": str(validator_binary(root)),
        "KMIPKIT_TEST_SERVER": str(test_server_binary(root)),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--github-env", action="store_true", help="append values to GITHUB_ENV")
    args = parser.parse_args()
    values = assignments(ROOT)
    if args.github_env:
        destination = Path(os.environ["GITHUB_ENV"])
        with destination.open("a") as stream:
            for name, value in values.items():
                stream.write(f"{name}={value}\n")
    else:
        for name, value in values.items():
            print(f"export {name}={shlex.quote(value)}")


if __name__ == "__main__":
    main()
