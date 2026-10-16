"""Stable, redacted error categories exposed by KMIPKit."""

from __future__ import annotations

from enum import IntEnum as _IntEnum

from . import ttlv as _ttlv


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


class ClientErrorCategory(_IntEnum):
    """Stable client execution categories returned by the C ABI."""

    VALIDATION = 1
    PROTOCOL = 2
    TRANSPORT = 3
    SERVER_RESULT = 4
    OTHER = 5


class ClientErrorCause(_IntEnum):
    """Redacted client failure cause classifications returned by the C ABI."""

    NONE = 0
    INVALID_INPUT = 1
    PROTOCOL_INVALID_VALUE = 2
    PROTOCOL_INVALID_ENCODING = 3
    PROTOCOL_OTHER = 4
    TRANSPORT_IO = 5
    TRANSPORT_TLS = 6
    TRANSPORT_HTTP = 7
    TIMEOUT = 8
    TRANSPORT_OTHER = 9
    OTHER = 10


class DeliveryState(_IntEnum):
    """Whether a failed client request reached the remote endpoint."""

    NONE = 0
    NOT_SENT = 1
    POSSIBLY_SENT = 2
    RESPONSE_STARTED = 3


def _known_or_raw(enum_type: type[_IntEnum], value: int) -> _IntEnum | int:
    try:
        return enum_type(value)
    except ValueError:
        return value


class ClientExecutionError(KmipKitError):
    """A client call failed with sanitized metadata and optional approved UID-error TTLV.

    ``response_ttlv`` is owned by this exception and may be closed by the caller. It is present
    only for a missing, repeated, or malformed required Unique Identifier in a successful MAC,
    MAC Verify, Sign, or Signature Verify response; all other failures expose no response TTLV.
    """

    category = "client_execution"

    def __init__(
        self,
        *,
        status: int,
        category: ClientErrorCategory | int,
        cause: ClientErrorCause | int,
        delivery: DeliveryState | int,
        response_ttlv: _ttlv.Structure | None = None,
    ) -> None:
        self.status = int(status)
        self.client_category = _known_or_raw(ClientErrorCategory, int(category))
        self.cause = _known_or_raw(ClientErrorCause, int(cause))
        self.delivery = _known_or_raw(DeliveryState, int(delivery))
        self.response_ttlv = response_ttlv
        super().__init__()
        self.args = (
            f"KMIPKit client operation failed ({self.client_category!s}; "
            f"delivery={self.delivery!s})",
        )


_ERROR_TYPES: dict[int, type[KmipKitError]] = {
    1: CompatibilityMismatchError,
    2: DuplicateKeyError,
    3: InvalidIdentityError,
    4: InvalidInputError,
    5: InvalidExtensionSchemaError,
    6: ResourceLimitError,
    7: KmipKitError,
}


def raise_for_status(status: int) -> None:
    """Raise the manifest-defined exception for a nonzero C ABI status."""
    if status == 0:
        return
    exception_type = _ERROR_TYPES.get(int(status), KmipKitError)
    raise exception_type()
