"""Require compiled JNI libraries to export manifest-backed PKCS#11 symbols."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

from audit_binding_exports import read_native_exports


ROOT = Path(__file__).resolve().parents[2]
PARITY_FIXTURE = ROOT / "tests" / "fixtures" / "api" / "pkcs11-parity.json"
PUBLIC_API_MANIFEST = ROOT / "specification" / "api" / "public-api.json"
JNI_EXPORT_PREFIX = "Java_org_kmipkit_"
NATIVE_REGISTRY_EXPORT_PREFIX = "org.kmipkit.NativeExtensionRegistry."


def expected_exports() -> set[str]:
    fixture = json.loads(PARITY_FIXTURE.read_text(encoding="utf-8"))
    exports = set(fixture["publicNativeExports"]["javaJni"])
    if not exports or any(not name.startswith(JNI_EXPORT_PREFIX) for name in exports):
        raise ValueError("the fixture has no valid public KMIPKit JNI exports")
    return exports


def approved_internal_registry_exports() -> set[str]:
    fixture = json.loads(PARITY_FIXTURE.read_text(encoding="utf-8"))
    manifest = json.loads(PUBLIC_API_MANIFEST.read_text(encoding="utf-8"))
    exclusions = manifest.get("approvedExclusions")
    if not isinstance(exclusions, list):
        raise ValueError("the public API manifest has no approved exclusion list")
    excluded_names = {
        row["exportName"]
        for row in exclusions
        if row.get("packageId") == "java-kmipkit"
        and isinstance(row.get("exportName"), str)
    }
    entries = fixture.get("approvedInternalNativeExports", {}).get("javaJni")
    if not isinstance(entries, list):
        raise ValueError("the PKCS#11 fixture has no approved internal JNI exports")

    exports: set[str] = set()
    for entry in entries:
        if not isinstance(entry, dict):
            raise ValueError("an approved internal JNI export is malformed")
        symbol = entry.get("symbol")
        manifest_name = entry.get("manifestExport")
        if not isinstance(symbol, str) or not symbol.startswith(JNI_EXPORT_PREFIX):
            raise ValueError("an approved internal JNI symbol is invalid")
        if not isinstance(manifest_name, str) or not manifest_name.startswith(
            NATIVE_REGISTRY_EXPORT_PREFIX
        ):
            raise ValueError("an approved internal JNI manifest name is invalid")
        expected_symbol = (
            JNI_EXPORT_PREFIX
            + "NativeExtensionRegistry_"
            + manifest_name[len(NATIVE_REGISTRY_EXPORT_PREFIX) :]
        )
        if symbol != expected_symbol or manifest_name not in excluded_names:
            raise ValueError("an internal JNI export lacks its exact approved exclusion")
        exports.add(symbol)
    return exports


def audit(binary: Path) -> int:
    if not binary.is_file():
        print(f"JNI library does not exist: {binary}", file=sys.stderr)
        return 2

    try:
        expected = expected_exports()
        approved_internal = approved_internal_registry_exports()
        observed = set(read_native_exports(binary))
    except (KeyError, OSError, ValueError, json.JSONDecodeError) as error:
        print(f"could not audit the public JNI inventory: {error}", file=sys.stderr)
        return 2

    missing = sorted(expected - observed)
    if missing:
        print(
            "JNI library is missing public manifest-backed symbols: "
            + ", ".join(missing),
            file=sys.stderr,
        )
        return 1

    # The full JNI library also exports methods for other, independently
    # inventoried Java APIs and approved internal handle helpers. Keep this
    # audit scoped to the PKCS#11 surface: an extra candidate or otherwise
    # unlisted PKCS#11 JNI symbol is an accidental public API expansion.
    expected_pkcs11 = {name for name in expected if "pkcs11" in name.casefold()}
    observed_pkcs11 = {
        name
        for name in observed
        if name.startswith(JNI_EXPORT_PREFIX) and "pkcs11" in name.casefold()
    }
    extra = sorted(observed_pkcs11 - expected_pkcs11 - approved_internal)
    candidate = sorted(
        name
        for name in observed
        if name.startswith(JNI_EXPORT_PREFIX) and "candidate" in name.casefold()
    )
    if extra or candidate:
        unexpected = sorted(set(extra) | set(candidate))
        print(
            "JNI library exports unlisted PKCS#11 or candidate symbols: "
            + ", ".join(unexpected),
            file=sys.stderr,
        )
        return 1

    print(f"public JNI export check passed: {len(expected)} required symbols present")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    arguments = parser.parse_args()
    return audit(arguments.binary)


if __name__ == "__main__":
    raise SystemExit(main())
