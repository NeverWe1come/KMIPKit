"""Stable, redacted error categories exposed by KMIPKit."""

from __future__ import annotations


class KmipKitError(Exception):
    """Base class for stable KMIPKit binding errors."""

    category = "kmipkit_error"

    def __init__(self) -> None:
        super().__init__(f"KMIPKit operation failed ({self.category})")


class CompatibilityMismatchError(KmipKitError):
    """The extension compatibility range does not match this runtime."""

    category = "compatibility_mismatch"


class DuplicateKeyError(KmipKitError):
    """An extension definition duplicates a registry identity."""

    category = "duplicate_key"


class InvalidIdentityError(KmipKitError):
    """An extension identity is invalid."""

    category = "invalid_identity"


class InvalidInputError(KmipKitError):
    """An argument or handle is invalid, including a closed handle."""

    category = "invalid_input"


class InvalidExtensionSchemaError(KmipKitError):
    """An extension schema or value does not satisfy its constraints."""

    category = "invalid_schema"


class ResourceLimitError(KmipKitError):
    """An operation exceeded a configured or hard resource limit."""

    category = "resource_limit"


_ERROR_TYPES: dict[int, type[KmipKitError]] = {
    1: CompatibilityMismatchError,
    2: DuplicateKeyError,
    3: InvalidIdentityError,
    4: InvalidInputError,
    5: InvalidExtensionSchemaError,
    6: ResourceLimitError,
}


def raise_for_status(status: int) -> None:
    """Raise the manifest-defined exception for a nonzero C ABI status."""
    if status == 0:
        return
    exception_type = _ERROR_TYPES.get(int(status), KmipKitError)
    raise exception_type()
