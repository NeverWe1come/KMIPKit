"""Raw-preserving KMIP credential discriminators."""

from __future__ import annotations

from collections.abc import Callable
from threading import RLock
from typing import TypeVar

from . import errors
from ._ffi import ffi, lib
from ._handles import (
    NativeHandle,
    _integer_range,
    _new_handle,
    _wrap_owned_handle,
    synchronized_native_call,
)
from .ttlv import Structure, TtlvStructureView

_T = TypeVar("_T")


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


class Nonce(NativeHandle):
    """Server-sourced Attestation Nonce preserving its original TTLV structure."""

    _release_name = "kmipkit_nonce_release"

    @classmethod
    def from_ttlv(cls, structure: Structure) -> Nonce:
        """Consume and validate a TTLV Structure containing Nonce ID and Nonce Value."""
        handle = _new_handle(
            "kmipkit_nonce_t",
            lib.kmipkit_nonce_from_ttlv,
            structure,
            consumed=(0,),
        )
        nonce = _wrap_owned_handle(cls, handle)
        nonce._view_lock = RLock()
        nonce._active_views = 0
        return nonce

    @synchronized_native_call
    def with_ttlv(self, callback: Callable[[TtlvStructureView], _T]) -> _T:
        """Lend a read-only TTLV view and close it when the callback returns."""
        if not callable(callback):
            errors.raise_for_status(4)
        view_handle = _new_handle(
            "kmipkit_ttlv_structure_view_t",
            lib.kmipkit_nonce_ttlv_view,
            self._pointer(),
        )
        view = _wrap_owned_handle(TtlvStructureView, view_handle, owner=self)
        with self._view_lock:
            self._active_views += 1
        try:
            return callback(view)
        finally:
            try:
                view.close()
            finally:
                with self._view_lock:
                    self._active_views -= 1

    @synchronized_native_call
    def into_ttlv(self) -> Structure:
        """Transfer the original ordered TTLV Structure without rebuilding it."""
        with self._view_lock:
            if self._active_views:
                errors.raise_for_status(4)
        handle = _new_handle(
            "kmipkit_ttlv_structure_t",
            lib.kmipkit_nonce_into_ttlv,
            self,
            consumed=(0,),
        )
        return _wrap_owned_handle(Structure, handle)


__all__ = ["CredentialType", "Nonce"]
