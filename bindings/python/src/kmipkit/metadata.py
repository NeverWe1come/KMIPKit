"""Runtime version, capability, and safe C ABI error metadata."""

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


class ClientErrorCategory(_OpenCode):
    """Stable category of the last C ABI error; unknown values are preserved."""

    NONE = 0
    VALIDATION = 1
    PROTOCOL = 2
    TRANSPORT = 3
    SERVER_RESULT = 4
    OTHER = 5


class ClientCauseCategory(_OpenCode):
    """Safe cause category of the last C ABI error."""

    NONE = 0
    INVALID_INPUT = 1
    OTHER = 10


class RequestDeliveryState(_OpenCode):
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
class FfiErrorSnapshot:
    """A non-destructive snapshot without native source text or payloads."""

    status: int
    category: ClientErrorCategory
    cause: ClientCauseCategory
    delivery: RequestDeliveryState


def get_version() -> Version:
    """Return the native package version and stable C ABI revision."""

    major = ffi.new("uint32_t *")
    minor = ffi.new("uint32_t *")
    patch = ffi.new("uint32_t *")
    abi_version = ffi.new("uint32_t *")
    errors.raise_for_status(lib.kmipkit_version_get(major, minor, patch, abi_version))
    return Version(int(major[0]), int(minor[0]), int(patch[0]), int(abi_version[0]))


def capability_is_available(capability_id: str) -> bool:
    """Return whether the reviewed manifest has an unblocked C mapping."""

    if not isinstance(capability_id, str):
        raise TypeError("capability_id must be str")
    encoded = capability_id.encode("utf-8")
    data = ffi.new("uint8_t[]", encoded if encoded else b"\0")
    available = ffi.new("uint8_t *")
    status = lib.kmipkit_capability_is_available(data, len(encoded), available)
    errors.raise_for_status(status)
    return bool(available[0])


def last_error_snapshot() -> FfiErrorSnapshot:
    """Read the calling thread's safe error snapshot without clearing it."""

    status = ffi.new("int32_t *")
    category = ffi.new("uint32_t *")
    cause = ffi.new("uint32_t *")
    delivery = ffi.new("uint32_t *")
    result = lib.kmipkit_last_error_get(status, category, cause, delivery)
    errors.raise_for_status(result)
    return FfiErrorSnapshot(
        status=int(status[0]),
        category=ClientErrorCategory(int(category[0])),
        cause=ClientCauseCategory(int(cause[0])),
        delivery=RequestDeliveryState(int(delivery[0])),
    )


def clear_last_error() -> None:
    """Clear the calling thread's native error snapshot."""

    errors.raise_for_status(lib.kmipkit_last_error_clear())


__all__ = [
    "ClientCauseCategory",
    "ClientErrorCategory",
    "FfiErrorSnapshot",
    "RequestDeliveryState",
    "Version",
    "capability_is_available",
    "clear_last_error",
    "get_version",
    "last_error_snapshot",
]
