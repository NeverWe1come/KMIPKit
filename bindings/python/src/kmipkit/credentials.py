"""Raw-preserving KMIP credential discriminators."""

from __future__ import annotations

from collections.abc import Callable, Sequence
from typing import TypeVar
from weakref import WeakKeyDictionary

from . import errors
from ._ffi import ffi, lib
from ._handles import (
    NativeHandle,
    _integer_range,
    _new_handle,
    _new_handle_from_array,
    _wrap_owned_handle,
    synchronized_native_call,
)
from .ttlv import Structure, TtlvStructureView

_T = TypeVar("_T")
_ACTIVE_NONCE_VIEWS: WeakKeyDictionary[object, int] = WeakKeyDictionary()
_ACTIVE_CREDENTIAL_VIEWS: WeakKeyDictionary[object, int] = WeakKeyDictionary()
_ACTIVE_ATTESTATION_CREDENTIAL_VIEWS: WeakKeyDictionary[object, int] = WeakKeyDictionary()
_ACTIVE_DEVICE_CREDENTIAL_VIEWS: WeakKeyDictionary[object, int] = WeakKeyDictionary()
_ACTIVE_AUTHENTICATION_VIEWS: WeakKeyDictionary[object, int] = WeakKeyDictionary()
_MAX_AUTHENTICATION_CREDENTIALS = 100_000


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

    __slots__ = ("__weakref__",)

    _release_name = "kmipkit_nonce_release"

    @classmethod
    def from_ttlv(cls, structure: Structure) -> Nonce:
        """Consume and validate a TTLV Structure containing Nonce ID and Nonce Value."""
        handle = _new_handle(
            "kmipkit_nonce_t",
            lib.kmipkit_nonce_from_ttlv,
            structure,
            consumed=(0,),
            restore_on_statuses=(errors._SHARED_OWNER_STATUS,),
        )
        nonce = _wrap_owned_handle(cls, handle)
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
        previous_view_count = _ACTIVE_NONCE_VIEWS.get(self, 0)
        _ACTIVE_NONCE_VIEWS[self] = previous_view_count + 1
        try:
            return callback(view)
        finally:
            try:
                view.close()
            finally:
                if previous_view_count:
                    _ACTIVE_NONCE_VIEWS[self] = previous_view_count
                else:
                    _ACTIVE_NONCE_VIEWS.pop(self, None)

    @synchronized_native_call
    def into_ttlv(self) -> Structure:
        """Transfer the original ordered TTLV Structure without rebuilding it."""
        if _ACTIVE_NONCE_VIEWS.get(self, 0):
            errors.raise_for_status(4)
        handle = _new_handle(
            "kmipkit_ttlv_structure_t",
            lib.kmipkit_nonce_into_ttlv,
            self,
            consumed=(0,),
            # The native ABI reports an active shared view before removing the owner.
            restore_on_statuses=(4,),
        )
        return _wrap_owned_handle(Structure, handle)


class Credential(NativeHandle):
    """Generic KMIP Credential envelope preserving its original ordered TTLV."""

    __slots__ = ("__weakref__",)

    _release_name = "kmipkit_credential_release"

    @classmethod
    def from_ttlv(cls, structure: Structure) -> Credential:
        """Consume and validate the outer Credential Type and Credential Value fields."""
        handle = _new_handle(
            "kmipkit_credential_t",
            lib.kmipkit_credential_from_ttlv,
            structure,
            consumed=(0,),
            restore_on_statuses=(errors._SHARED_OWNER_STATUS,),
        )
        return _wrap_owned_handle(cls, handle)

    @property
    @synchronized_native_call
    def credential_type(self) -> CredentialType:
        """Return a raw-preserving typed view of the Credential Type."""
        out_raw = ffi.new("uint32_t *")
        errors.raise_for_status(
            lib.kmipkit_credential_type(self._pointer(), out_raw)
        )
        return CredentialType.from_raw(int(out_raw[0]))

    @synchronized_native_call
    def credential_type_raw(self) -> int:
        """Return the original unsigned 32-bit Credential Type value."""
        out_raw = ffi.new("uint32_t *")
        errors.raise_for_status(
            lib.kmipkit_credential_type_raw(self._pointer(), out_raw)
        )
        return int(out_raw[0])

    @synchronized_native_call
    def with_ttlv(self, callback: Callable[[TtlvStructureView], _T]) -> _T:
        """Lend a read-only TTLV view and close it when the callback returns."""
        if not callable(callback):
            errors.raise_for_status(4)
        view_handle = _new_handle(
            "kmipkit_ttlv_structure_view_t",
            lib.kmipkit_credential_ttlv_view,
            self._pointer(),
        )
        view = _wrap_owned_handle(TtlvStructureView, view_handle, owner=self)
        previous_view_count = _ACTIVE_CREDENTIAL_VIEWS.get(self, 0)
        _ACTIVE_CREDENTIAL_VIEWS[self] = previous_view_count + 1
        try:
            return callback(view)
        finally:
            try:
                view.close()
            finally:
                if previous_view_count:
                    _ACTIVE_CREDENTIAL_VIEWS[self] = previous_view_count
                else:
                    _ACTIVE_CREDENTIAL_VIEWS.pop(self, None)

    @synchronized_native_call
    def into_ttlv(self) -> Structure:
        """Transfer the original ordered TTLV Structure without rebuilding it."""
        if _ACTIVE_CREDENTIAL_VIEWS.get(self, 0):
            errors.raise_for_status(4)
        handle = _new_handle(
            "kmipkit_ttlv_structure_t",
            lib.kmipkit_credential_into_ttlv,
            self,
            consumed=(0,),
            restore_on_statuses=(4,),
        )
        return _wrap_owned_handle(Structure, handle)


class Authentication(NativeHandle):
    """Ordered KMIP Authentication preserving its original TTLV structure."""

    __slots__ = ("__weakref__",)

    _release_name = "kmipkit_authentication_release"

    @classmethod
    def create(cls, credentials: Sequence[Credential]) -> Authentication:
        """Consume a nonempty ordered sequence of Credential handles."""
        if isinstance(credentials, (str, bytes, bytearray)) or not isinstance(
            credentials, Sequence
        ):
            errors.raise_for_status(4)
        if len(credentials) > _MAX_AUTHENTICATION_CREDENTIALS:
            errors.raise_for_status(6)
        values = tuple(credentials)
        if not values:
            errors.raise_for_status(4)
        handle = _new_handle_from_array(
            "kmipkit_credential_t",
            "kmipkit_authentication_t",
            lib.kmipkit_authentication_create,
            values,
            restore_on_statuses=(4, errors._SHARED_OWNER_STATUS),
        )
        return _wrap_owned_handle(cls, handle)

    @classmethod
    def from_ttlv(cls, structure: Structure) -> Authentication:
        """Consume and validate a TTLV Structure containing Authentication."""
        handle = _new_handle(
            "kmipkit_authentication_t",
            lib.kmipkit_authentication_from_ttlv,
            structure,
            consumed=(0,),
            restore_on_statuses=(errors._SHARED_OWNER_STATUS,),
        )
        return _wrap_owned_handle(cls, handle)

    @synchronized_native_call
    def _credential_count(self) -> int:
        """Return the number of ordered Credential entries."""
        out_count = ffi.new("uint64_t *")
        errors.raise_for_status(
            lib.kmipkit_authentication_credential_count(self._pointer(), out_count)
        )
        return int(out_count[0])

    @property
    @synchronized_native_call
    def credentials(self) -> tuple[CredentialView, ...]:
        """Return owner-backed views in the Authentication wire order."""
        result: list[CredentialView] = []
        try:
            for index in range(self._credential_count()):
                handle = _new_handle(
                    "kmipkit_credential_view_t",
                    lib.kmipkit_authentication_credential_at,
                    self._pointer(),
                    index,
                )
                result.append(_wrap_owned_handle(CredentialView, handle, owner=self))
        except BaseException:
            for view in result:
                view.close()
            raise
        return tuple(result)

    @synchronized_native_call
    def with_ttlv(self, callback: Callable[[TtlvStructureView], _T]) -> _T:
        """Lend a read-only TTLV view and close it when the callback returns."""
        if not callable(callback):
            errors.raise_for_status(4)
        view_handle = _new_handle(
            "kmipkit_ttlv_structure_view_t",
            lib.kmipkit_authentication_ttlv_view,
            self._pointer(),
        )
        view = _wrap_owned_handle(TtlvStructureView, view_handle, owner=self)
        previous_view_count = _ACTIVE_AUTHENTICATION_VIEWS.get(self, 0)
        _ACTIVE_AUTHENTICATION_VIEWS[self] = previous_view_count + 1
        try:
            return callback(view)
        finally:
            try:
                view.close()
            finally:
                if previous_view_count:
                    _ACTIVE_AUTHENTICATION_VIEWS[self] = previous_view_count
                else:
                    _ACTIVE_AUTHENTICATION_VIEWS.pop(self, None)

    @synchronized_native_call
    def into_ttlv(self) -> Structure:
        """Transfer the original ordered Authentication Structure without rebuilding it."""
        if _ACTIVE_AUTHENTICATION_VIEWS.get(self, 0):
            errors.raise_for_status(4)
        handle = _new_handle(
            "kmipkit_ttlv_structure_t",
            lib.kmipkit_authentication_into_ttlv,
            self,
            consumed=(0,),
            # The opaque-model conversion reports a live owner as invalid_input.
            restore_on_statuses=(4,),
        )
        return _wrap_owned_handle(Structure, handle)


class CredentialView(NativeHandle):
    """Owner-backed read-only view of one Authentication Credential entry."""

    __slots__ = ("__weakref__",)

    _release_name = "kmipkit_credential_view_release"

    @property
    @synchronized_native_call
    def credential_type(self) -> CredentialType:
        """Return the raw-preserving typed Credential Type."""
        out_raw = ffi.new("uint32_t *")
        errors.raise_for_status(
            lib.kmipkit_credential_view_type(self._pointer(), out_raw)
        )
        return CredentialType.from_raw(int(out_raw[0]))

    @synchronized_native_call
    def credential_type_raw(self) -> int:
        """Return the original unsigned 32-bit Credential Type value."""
        out_raw = ffi.new("uint32_t *")
        errors.raise_for_status(
            lib.kmipkit_credential_view_type_raw(self._pointer(), out_raw)
        )
        return int(out_raw[0])

    @synchronized_native_call
    def with_ttlv(self, callback: Callable[[TtlvStructureView], _T]) -> _T:
        """Lend the Credential Structure to a callback and close the view on return."""
        if not callable(callback):
            errors.raise_for_status(4)
        view_handle = _new_handle(
            "kmipkit_ttlv_structure_view_t",
            lib.kmipkit_credential_view_ttlv_view,
            self._pointer(),
        )
        view = _wrap_owned_handle(TtlvStructureView, view_handle, owner=self)
        try:
            return callback(view)
        finally:
            view.close()


class AttestationCredential(NativeHandle):
    """Validated Attestation Credential Value preserving its original TTLV structure.

    Byte arrays copied from a TTLV view are Python-managed values and cannot be
    deterministically zeroized by KMIPKit.
    """

    __slots__ = ("__weakref__",)

    _release_name = "kmipkit_attestation_credential_release"

    @classmethod
    def from_ttlv(cls, structure: Structure) -> AttestationCredential:
        """Consume and validate a TTLV Structure containing the attestation value."""
        handle = _new_handle(
            "kmipkit_attestation_credential_t",
            lib.kmipkit_attestation_credential_from_ttlv,
            structure,
            consumed=(0,),
            restore_on_statuses=(errors._SHARED_OWNER_STATUS,),
        )
        return _wrap_owned_handle(cls, handle)

    @synchronized_native_call
    def with_ttlv(self, callback: Callable[[TtlvStructureView], _T]) -> _T:
        """Lend a read-only TTLV view and close it when the callback returns."""
        if not callable(callback):
            errors.raise_for_status(4)
        view_handle = _new_handle(
            "kmipkit_ttlv_structure_view_t",
            lib.kmipkit_attestation_credential_ttlv_view,
            self._pointer(),
        )
        view = _wrap_owned_handle(TtlvStructureView, view_handle, owner=self)
        previous_view_count = _ACTIVE_ATTESTATION_CREDENTIAL_VIEWS.get(self, 0)
        _ACTIVE_ATTESTATION_CREDENTIAL_VIEWS[self] = previous_view_count + 1
        try:
            return callback(view)
        finally:
            try:
                view.close()
            finally:
                if previous_view_count:
                    _ACTIVE_ATTESTATION_CREDENTIAL_VIEWS[self] = previous_view_count
                else:
                    _ACTIVE_ATTESTATION_CREDENTIAL_VIEWS.pop(self, None)

    @synchronized_native_call
    def into_ttlv(self) -> Structure:
        """Transfer the original ordered TTLV Structure without rebuilding it."""
        if _ACTIVE_ATTESTATION_CREDENTIAL_VIEWS.get(self, 0):
            errors.raise_for_status(4)
        handle = _new_handle(
            "kmipkit_ttlv_structure_t",
            lib.kmipkit_attestation_credential_into_ttlv,
            self,
            consumed=(0,),
            restore_on_statuses=(4,),
        )
        return _wrap_owned_handle(Structure, handle)


class DeviceCredential(NativeHandle):
    """Validated Device Credential Value preserving its exact original TTLV."""

    __slots__ = ("__weakref__",)

    _release_name = "kmipkit_device_credential_release"

    @classmethod
    def from_ttlv(cls, structure: Structure) -> DeviceCredential:
        """Consume and validate a TTLV Structure using KMIP 2.1 Table 412 rules."""
        handle = _new_handle(
            "kmipkit_device_credential_t",
            lib.kmipkit_device_credential_from_ttlv,
            structure,
            consumed=(0,),
            restore_on_statuses=(errors._SHARED_OWNER_STATUS,),
        )
        return _wrap_owned_handle(cls, handle)

    @synchronized_native_call
    def with_ttlv(self, callback: Callable[[TtlvStructureView], _T]) -> _T:
        """Lend the original ordered TTLV and close the read-only view on return."""
        if not callable(callback):
            errors.raise_for_status(4)
        view_handle = _new_handle(
            "kmipkit_ttlv_structure_view_t",
            lib.kmipkit_device_credential_ttlv_view,
            self._pointer(),
        )
        view = _wrap_owned_handle(TtlvStructureView, view_handle, owner=self)
        previous_view_count = _ACTIVE_DEVICE_CREDENTIAL_VIEWS.get(self, 0)
        _ACTIVE_DEVICE_CREDENTIAL_VIEWS[self] = previous_view_count + 1
        try:
            return callback(view)
        finally:
            try:
                view.close()
            finally:
                if previous_view_count:
                    _ACTIVE_DEVICE_CREDENTIAL_VIEWS[self] = previous_view_count
                else:
                    _ACTIVE_DEVICE_CREDENTIAL_VIEWS.pop(self, None)

    @synchronized_native_call
    def into_ttlv(self) -> Structure:
        """Transfer the original ordered TTLV Structure without rebuilding it."""
        if _ACTIVE_DEVICE_CREDENTIAL_VIEWS.get(self, 0):
            errors.raise_for_status(4)
        handle = _new_handle(
            "kmipkit_ttlv_structure_t",
            lib.kmipkit_device_credential_into_ttlv,
            self,
            consumed=(0,),
            restore_on_statuses=(4,),
        )
        return _wrap_owned_handle(Structure, handle)


__all__ = [
    "AttestationCredential",
    "Authentication",
    "Credential",
    "CredentialType",
    "CredentialView",
    "DeviceCredential",
    "Nonce",
]
