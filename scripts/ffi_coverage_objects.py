"""Print ABI libraries and executed FFI coverage tests from Cargo, NUL separated."""

from __future__ import annotations

import json
from pathlib import Path
import sys


def coverage_objects(manifest: Path) -> list[Path]:
    """Require exact Cargo artifacts without relying on an intermediate layout."""
    objects: dict[str, set[Path]] = {
        "dso": set(),
        "staticlib": set(),
        "unit": set(),
        "consumer": set(),
        "parity": set(),
    }
    for line in manifest.read_text(encoding="utf-8").splitlines():
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
                objects[role].add(Path(message["executable"]))
        elif name == "kmipkit_ffi":
            for filename in message["filenames"]:
                path = Path(filename)
                role = {
                    "libkmipkit_ffi.so": "dso",
                    "libkmipkit_ffi.a": "staticlib",
                }.get(path.name)
                if role:
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
    try:
        objects = coverage_objects(Path(sys.argv[1]))
    except (OSError, ValueError, KeyError) as error:
        print(f"Cannot resolve FFI coverage objects: {error}", file=sys.stderr)
        return 1
    for path in objects:
        sys.stdout.buffer.write(bytes(path) + b"\0")
    return 0


if __name__ == "__main__":
    sys.exit(main())
