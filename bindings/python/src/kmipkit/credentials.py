"""Raw-preserving KMIP credential discriminators."""

from __future__ import annotations

from . import errors
from ._client_ffi import ffi, lib
from ._handles import _integer_range


class CredentialType:
    """A Credential Type that preserves known, extension, and unknown raw values."""

    USERNAME_AND_PASSWORD = 1
    DEVICE = 2
    ATTESTATION = 3
    ONE_TIME_PASSWORD = 4
    HASHED_PASSWORD = 5
    TICKET = 6

    def __init__(self, raw: int) -> None:
        checked = _integer_range(raw, 0, 0xFFFF_FFFF)
        out_raw = ffi.new("uint32_t *")
        errors.raise_for_status(lib.kmipkit_credential_type_from_raw(checked, out_raw))
        self._value = int(out_raw[0])

    @classmethod
    def from_raw(cls, raw: int) -> CredentialType:
        """Create a canonical Rust-backed view from an unsigned 32-bit value."""
        return cls(raw)

    def raw(self) -> int:
        """Return the original unsigned 32-bit enumeration value."""
        return self._value

    @property
    def kind(self) -> str:
        """Return the known, Extensions, or Unknown category for this value."""
        names = {
            1: "UsernameAndPassword",
            2: "Device",
            3: "Attestation",
            4: "OneTimePassword",
            5: "HashedPassword",
            6: "Ticket",
        }
        if self._value in names:
            return names[self._value]
        if 0x8000_0000 <= self._value <= 0x8FFF_FFFF:
            return "Extensions"
        return "Unknown"


__all__ = ["CredentialType"]
