"""Print ABI libraries and executed FFI coverage tests from Cargo, NUL separated."""

from __future__ import annotations

import json
import os
from collections.abc import Iterable
from pathlib import Path
import sys


def coverage_objects(manifest: Iterable[str], target_root: Path) -> list[Path]:
    """Resolve exact Cargo artifacts constrained to the configured target tree."""
    target_root = target_root.resolve()
    objects: dict[str, set[Path]] = {
        "dso": set(),
        "staticlib": set(),
        "unit": set(),
        "consumer": set(),
        "parity": set(),
    }
    for line in manifest:
        message = json.loads(line)
        if message.get("reason") != "compiler-artifact":
            continue
        name = message["target"]["name"]
        if message["profile"]["test"]:
            role = {
                "kmipkit_ffi": "unit",
                "c_api_coverage": "consumer",
                "operation_parity_0053": "parity",
            }.get(name)
            if role and message.get("executable"):
                path = Path(message["executable"]).resolve()
                if not path.is_relative_to(target_root):
                    raise ValueError(
                        f"Required {role} coverage object is outside the Cargo target directory"
                    )
                objects[role].add(path)
        elif name == "kmipkit_ffi":
            for filename in message["filenames"]:
                path = Path(filename).resolve()
                role = {
                    "libkmipkit_ffi.so": "dso",
                    "libkmipkit_ffi.a": "staticlib",
                }.get(path.name)
                if role:
                    if not path.is_relative_to(target_root):
                        raise ValueError(
                            f"Required {role} coverage object is outside the Cargo target directory"
                        )
                    objects[role].add(path)
    result = []
    for role, paths in objects.items():
        if len(paths) != 1:
            raise ValueError(
                f"Expected exactly one {role} coverage object; found {len(paths)}"
            )
        path = next(iter(paths))
        if not path.is_file():
            raise ValueError(f"Required {role} coverage object is missing: {path}")
        result.append(path)
    return result


def main() -> int:
    if len(sys.argv) != 1:
        print(
            "Usage: ffi_coverage_objects.py < cargo-artifacts.jsonl",
            file=sys.stderr,
        )
        return 2
    target_root = os.environ.get("CARGO_TARGET_DIR")
    if not target_root:
        print("CARGO_TARGET_DIR must identify the Cargo artifact directory", file=sys.stderr)
        return 2
    try:
        objects = coverage_objects(sys.stdin, Path(target_root))
    except (OSError, ValueError, KeyError) as error:
        print(f"Cannot resolve FFI coverage objects: {error}", file=sys.stderr)
        return 1
    for path in objects:
        sys.stdout.buffer.write(bytes(path) + b"\0")
    return 0


if __name__ == "__main__":
    sys.exit(main())
