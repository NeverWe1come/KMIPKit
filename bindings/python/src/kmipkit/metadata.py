"""Runtime version and capability metadata."""

from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum

from . import errors
from ._ffi import ffi, lib


class _OpenCode(IntEnum):
    @classmethod
    def _missing_(cls, value: object) -> _OpenCode | None:
        if type(value) is not int or not 0 <= value <= 0xFFFFFFFF:
            return None
        member = int.__new__(cls, value)
        member._name_ = f"UNKNOWN_{value}"
        member._value_ = value
        cls._value2member_map_[value] = member
        return member


class _ClientErrorCategory(_OpenCode):
    """Stable category of the last C ABI error; unknown values are preserved."""

    NONE = 0
    VALIDATION = 1
    PROTOCOL = 2
    TRANSPORT = 3
    SERVER_RESULT = 4
    OTHER = 5


class _ClientCauseCategory(_OpenCode):
    """Safe cause category of the last C ABI error."""

    NONE = 0
    INVALID_INPUT = 1
    OTHER = 10


class _RequestDeliveryState(_OpenCode):
    """Request delivery evidence associated with an error snapshot."""

    NONE = 0
    NOT_SENT = 1
    POSSIBLY_SENT = 2
    RESPONSE_STARTED = 3


@dataclass(frozen=True, slots=True)
class Version:
    """KMIPKit product and stable C ABI versions."""

    major: int
    minor: int
    patch: int
    abi_version: int


@dataclass(frozen=True, slots=True)
class _FfiErrorSnapshot:
    """Internal non-destructive snapshot without native source text or payloads."""

    status: int
    category: _ClientErrorCategory
    cause: _ClientCauseCategory
    delivery: _RequestDeliveryState


def get_version() -> Version:
    """Return the native package version and stable C ABI revision."""

    major = ffi.new("uint32_t *")
    minor = ffi.new("uint32_t *")
    patch = ffi.new("uint32_t *")
    abi_version = ffi.new("uint32_t *")
    errors.raise_for_status(lib.kmipkit_version_get(major, minor, patch, abi_version))
    return Version(int(major[0]), int(minor[0]), int(patch[0]), int(abi_version[0]))


def capability_is_available(capability_id: str) -> bool:
    """Return whether the reviewed manifest has an unblocked C mapping.

    Raises ``InvalidInputError`` for text that is not valid UTF-8 (including
    unpaired surrogates), and ``ResourceLimitError`` when the UTF-8 identifier
    exceeds 128 bytes.
    """

    if not isinstance(capability_id, str):
        raise TypeError("capability_id must be str")
    try:
        encoded = capability_id.encode("utf-8")
    except UnicodeEncodeError:
        raise errors.InvalidInputError() from None
    data = ffi.new("uint8_t[]", encoded if encoded else b"\0")
    available = ffi.new("uint8_t *")
    status = lib.kmipkit_capability_is_available(data, len(encoded), available)
    errors.raise_for_status(status)
    return bool(available[0])


def get_api_levels() -> tuple[str, ...]:
    """Return the local API-level taxonomy without sending a KMIP request.

    The taxonomy query describes KMIPKit's facade surface, is distinct from
    the KMIP Discover Versions operation, and does not assert that every KMIP
    operation is mapped or complete.
    """

    high_level = ffi.new("kmipkit_byte_span_v1_t *")
    typed_protocol = ffi.new("kmipkit_byte_span_v1_t *")
    generic_ttlv = ffi.new("kmipkit_byte_span_v1_t *")
    errors.raise_for_status(
        lib.kmipkit_api_levels_get_v1(high_level, typed_protocol, generic_ttlv)
    )
    spans = (high_level[0], typed_protocol[0], generic_ttlv[0])
    return tuple(ffi.string(span.data, int(span.length)).decode("utf-8") for span in spans)


def _last_error_snapshot() -> _FfiErrorSnapshot:
    """Read the native calling thread's safe error snapshot without clearing it."""

    status = ffi.new("int32_t *")
    category = ffi.new("uint32_t *")
    cause = ffi.new("uint32_t *")
    delivery = ffi.new("uint32_t *")
    result = lib.kmipkit_last_error_get(status, category, cause, delivery)
    errors.raise_for_status(result)
    return _FfiErrorSnapshot(
        status=int(status[0]),
        category=_ClientErrorCategory(int(category[0])),
        cause=_ClientCauseCategory(int(cause[0])),
        delivery=_RequestDeliveryState(int(delivery[0])),
    )


def _clear_last_error() -> None:
    """Clear the native calling thread's error snapshot for adapter internals."""

    errors.raise_for_status(lib.kmipkit_last_error_clear())


__all__ = [
    "Version",
    "capability_is_available",
    "get_version",
    "get_api_levels",
]
