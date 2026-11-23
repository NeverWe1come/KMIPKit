"""Read the pinned PKCS #11 values referenced by KMIP 2.x and 3.0."""

from collections.abc import Iterator
import hashlib
from pathlib import Path
import re

from . import OASIS_FIXTURES

HEADER_DIR = OASIS_FIXTURES / "pkcs11-3.0/include"
PKCS11_HEADERS = {
    "pkcs11.h": "260bbef88360ddbc8f7f668622c688ed7e6dab939e1eb2fdd6392abb571637e5",
    "pkcs11f.h": "c52bdac9e30f982cb5ad57db946a8ff6076efea744a90d684835d7269babab9b",
    "pkcs11t.h": "1f6763a72dc33a97d964d3d4b860870e1b4d4bf76b8705bdee26078caf4994cc",
}


def referenced_values(repository_root: Path) -> Iterator[tuple[str, str, str, str, str, str]]:
    """Yield each value's stable suffix, family, name, wire value, allocation and source header."""
    header_dir = repository_root / HEADER_DIR
    contents = {}
    for name, expected_digest in PKCS11_HEADERS.items():
        source_path = header_dir / name
        source_bytes = source_path.read_bytes()
        actual_digest = hashlib.sha256(source_bytes).hexdigest()
        if actual_digest != expected_digest:
            raise ValueError(f"Pinned OASIS PKCS #11 source digest mismatch: {source_path}")
        contents[name] = source_bytes.decode("ascii")

    functions = re.findall(
        r"^CK_PKCS11_FUNCTION_INFO\((C_[A-Za-z0-9_]+)\)", contents["pkcs11f.h"], re.MULTILINE
    )
    if not functions or len(functions) != len(set(functions)):
        raise ValueError("PKCS #11 function inventory is empty or contains duplicate ordered members")
    return_codes = re.findall(
        r"^#define\s+(CKR_[A-Za-z0-9_]+)\s+(0x[0-9A-Fa-f]+)UL\b",
        contents["pkcs11t.h"],
        re.MULTILINE,
    )
    if not return_codes or len(return_codes) != len({name for name, _ in return_codes}):
        raise ValueError("PKCS #11 return-code inventory is empty or contains duplicate names")

    for ordinal, name in enumerate(functions, start=1):
        yield f"FUNCTION-{ordinal:03d}", "PKCS#11 Function", name, f"{ordinal:08X}", "assigned", "pkcs11f.h"
    for name, raw_value in return_codes:
        suffix = re.sub(r"[^A-Za-z0-9]+", "-", name.removeprefix("CKR_")).strip("-")
        allocation = "extension" if name == "CKR_VENDOR_DEFINED" else "assigned"
        yield f"RETURN-{suffix}", "PKCS#11 Return Code", name, f"{int(raw_value, 16):08X}", allocation, "pkcs11t.h"
