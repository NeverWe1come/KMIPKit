"""Raw-preserving KMIP credential discriminators."""

from __future__ import annotations

import ctypes
from collections.abc import Callable, Sequence
from functools import lru_cache
from typing import Any, TypeVar
from weakref import WeakKeyDictionary

from . import errors
from ._ffi import ffi, lib, secret_callback_library
from ._handles import (
    NativeHandle,
    _integer_range,
    _new_handle,
    _new_handle_from_array,
    _wrap_owned_handle,
    synchronized_native_call,
)
from .ttlv import Structure, TtlvStructureView, Value

_T = TypeVar("_T")
_ACTIVE_NONCE_VIEWS: WeakKeyDictionary[object, int] = WeakKeyDictionary()
_ACTIVE_CREDENTIAL_VIEWS: WeakKeyDictionary[object, int] = WeakKeyDictionary()
_ACTIVE_ATTESTATION_CREDENTIAL_VIEWS: WeakKeyDictionary[object, int] = WeakKeyDictionary()
_ACTIVE_DEVICE_CREDENTIAL_VIEWS: WeakKeyDictionary[object, int] = WeakKeyDictionary()
_ACTIVE_USERNAME_PASSWORD_CREDENTIAL_VIEWS: WeakKeyDictionary[object, int] = WeakKeyDictionary()
_ACTIVE_HASHED_PASSWORD_CREDENTIAL_VIEWS: WeakKeyDictionary[object, int] = WeakKeyDictionary()
_ACTIVE_AUTHENTICATION_VIEWS: WeakKeyDictionary[object, int] = WeakKeyDictionary()
_MAX_AUTHENTICATION_CREDENTIALS = 100_000
_SECRET_SPAN_CALLBACK = ctypes.CFUNCTYPE(
    None,
    ctypes.POINTER(ctypes.c_uint8),
    ctypes.c_uint64,
    ctypes.c_void_p,
)


@lru_cache(maxsize=2)
def _secret_callback_entrypoint(symbol: str) -> Any:
    """Configure a synchronous C callback entrypoint from the selected library."""
    function = getattr(secret_callback_library(), symbol)
    function.argtypes = [
        ctypes.c_void_p,
        _SECRET_SPAN_CALLBACK,
        ctypes.c_void_p,
    ]
    function.restype = ctypes.c_int32
    return function


def _copy_secret_span(
    data: ctypes.POINTER(ctypes.c_uint8) | None, length: int
) -> bytearray:
    """Copy a borrowed native span directly into mutable Python-owned storage."""
    if length < 0 or (length > 0 and data is None):
        errors.raise_for_status(4)
    staging = bytearray(length)
    if length:
        destination = (ctypes.c_uint8 * length).from_buffer(staging)
        ctypes.memmove(destination, data, length)
    return staging


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


class SecretBytes(NativeHandle):
    """Own caller-provided secret bytes in native zeroizing memory."""

    __slots__ = ()
    _release_name = "kmipkit_secret_bytes_release"

    @classmethod
    def from_bytes(cls, value: bytes | bytearray | memoryview) -> SecretBytes:
        """Copy a bytes-like value into a native zeroizing owner."""
        if not isinstance(value, (bytes, bytearray, memoryview)):
            errors.raise_for_status(4)
        try:
            staging = bytearray(value)
        except (TypeError, ValueError, BufferError):
            errors.raise_for_status(4)
        try:
            if staging:
                source = ffi.from_buffer("const uint8_t[]", staging)
            else:
                source = ffi.NULL
            handle = _new_handle(
                "kmipkit_secret_bytes_t",
                lib.kmipkit_secret_bytes_create,
                source,
                len(staging),
            )
            return _wrap_owned_handle(cls, handle)
        finally:
            for index in range(len(staging)):
                staging[index] = 0

    @synchronized_native_call
    def with_bytes(self, callback: Callable[[memoryview], _T]) -> _T:
        """Lend a read-only view of a temporary copy, then clear and release it.

        The view's ``obj`` exposes the mutable temporary bytearray; changing it
        never changes the native secret. The bytearray is zeroed before return.
        Copies deliberately made by the callback are caller-owned.
        """
        if not callable(callback):
            errors.raise_for_status(4)
        result: list[_T] = []
        failures: list[BaseException] = []

        def invoke(
            data: ctypes.POINTER(ctypes.c_uint8) | None,
            length: int,
            _context: ctypes.c_void_p | None,
        ) -> None:
            staging = bytearray()
            view: memoryview | None = None
            try:
                staging = _copy_secret_span(data, int(length))
                view = memoryview(staging).toreadonly()
                result.append(callback(view))
            except BaseException as exception:
                failures.append(exception)
            finally:
                if view is not None:
                    try:
                        view.release()
                    except ValueError:
                        pass
                for index in range(len(staging)):
                    staging[index] = 0
                data = None
                length = 0
                _context = None

        native_callback = _SECRET_SPAN_CALLBACK(invoke)
        status = _secret_callback_entrypoint("kmipkit_secret_bytes_with_bytes")(
            ctypes.c_void_p(int(ffi.cast("uintptr_t", self._pointer()))),
            native_callback,
            None,
        )
        errors.raise_for_status(status)
        if failures:
            raise failures[0]
        if not result:
            errors.raise_for_status(4)
        return result[0]

    @synchronized_native_call
    def into_ttlv_value(self) -> Value:
        """Move the native secret allocation into a zeroizing TTLV Byte String."""
        handle = _new_handle(
            "kmipkit_ttlv_value_t",
            lib.kmipkit_secret_bytes_into_ttlv_value,
            self,
            consumed=(0,),
            restore_on_statuses=(errors._SHARED_OWNER_STATUS,),
        )
        return _wrap_owned_handle(Value, handle)

    def __repr__(self) -> str:
        return "SecretBytes([REDACTED])"


class SecretText(NativeHandle):
    """Own caller-provided UTF-8 text in native zeroizing memory."""

    __slots__ = ()
    _release_name = "kmipkit_secret_text_release"

    @classmethod
    def from_text(cls, value: str) -> SecretText:
        """Strictly encode text and copy it into a native zeroizing owner."""
        if not isinstance(value, str):
            errors.raise_for_status(4)
        try:
            staging = bytearray(value, "utf-8", "strict")
        except UnicodeEncodeError:
            errors.raise_for_status(4)
        try:
            if staging:
                source = ffi.from_buffer("const uint8_t[]", staging)
            else:
                source = ffi.NULL
            handle = _new_handle(
                "kmipkit_secret_text_t",
                lib.kmipkit_secret_text_create,
                source,
                len(staging),
            )
            return _wrap_owned_handle(cls, handle)
        finally:
            for index in range(len(staging)):
                staging[index] = 0

    @synchronized_native_call
    def with_str(self, callback: Callable[[str], _T]) -> _T:
        """Lend strict-decoded text, then clear the temporary UTF-8 byte copy.

        The callback receives an immutable Python ``str`` copy that cannot be
        reliably erased. Any further copies remain caller-owned.
        """
        if not callable(callback):
            errors.raise_for_status(4)
        result: list[_T] = []
        failures: list[BaseException] = []

        def invoke(
            utf8: ctypes.POINTER(ctypes.c_uint8) | None,
            length: int,
            _context: ctypes.c_void_p | None,
        ) -> None:
            staging = bytearray()
            try:
                staging = _copy_secret_span(utf8, int(length))
                result.append(callback(staging.decode("utf-8", "strict")))
            except BaseException as exception:
                failures.append(exception)
            finally:
                for index in range(len(staging)):
                    staging[index] = 0
                utf8 = None
                length = 0
                _context = None

        native_callback = _SECRET_SPAN_CALLBACK(invoke)
        status = _secret_callback_entrypoint("kmipkit_secret_text_with_str")(
            ctypes.c_void_p(int(ffi.cast("uintptr_t", self._pointer()))),
            native_callback,
            None,
        )
        errors.raise_for_status(status)
        if failures:
            raise failures[0]
        if not result:
            errors.raise_for_status(4)
        return result[0]

    @synchronized_native_call
    def into_ttlv_value(self) -> Value:
        """Move the native secret allocation into a zeroizing TTLV Text String."""
        handle = _new_handle(
            "kmipkit_ttlv_value_t",
            lib.kmipkit_secret_text_into_ttlv_value,
            self,
            consumed=(0,),
            restore_on_statuses=(errors._SHARED_OWNER_STATUS,),
        )
        return _wrap_owned_handle(Value, handle)

    def __repr__(self) -> str:
        return "SecretText([REDACTED])"


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
            credential_validation_error=True,
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
            credential_validation_error=True,
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
        """Consume ordered Credential handles and report native validation categories."""
        if isinstance(credentials, (str, bytes, bytearray)) or not isinstance(
            credentials, Sequence
        ):
            errors.raise_for_status(4)
        if len(credentials) > _MAX_AUTHENTICATION_CREDENTIALS:
            errors.raise_for_status(6)
        values = tuple(credentials)
        handle = _new_handle_from_array(
            "kmipkit_credential_t",
            "kmipkit_authentication_t",
            lib.kmipkit_authentication_create,
            values,
            credential_validation_error=True,
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
            credential_validation_error=True,
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
            credential_validation_error=True,
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
            credential_validation_error=True,
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


class UsernameAndPasswordCredential(NativeHandle):
    """Validated Username and Password Credential Value preserving its exact TTLV."""

    __slots__ = ("__weakref__",)

    _release_name = "kmipkit_username_and_password_credential_release"

    @classmethod
    def from_ttlv(cls, structure: Structure) -> UsernameAndPasswordCredential:
        """Consume and validate a TTLV Structure using KMIP 2.1 Table 411 rules."""
        handle = _new_handle(
            "kmipkit_username_and_password_credential_t",
            lib.kmipkit_username_and_password_credential_from_ttlv,
            structure,
            consumed=(0,),
            restore_on_statuses=(errors._SHARED_OWNER_STATUS,),
            credential_validation_error=True,
        )
        return _wrap_owned_handle(cls, handle)

    @synchronized_native_call
    def with_ttlv(self, callback: Callable[[TtlvStructureView], _T]) -> _T:
        """Lend the original ordered TTLV and close the read-only view on return."""
        if not callable(callback):
            errors.raise_for_status(4)
        view_handle = _new_handle(
            "kmipkit_ttlv_structure_view_t",
            lib.kmipkit_username_and_password_credential_ttlv_view,
            self._pointer(),
        )
        view = _wrap_owned_handle(TtlvStructureView, view_handle, owner=self)
        previous_view_count = _ACTIVE_USERNAME_PASSWORD_CREDENTIAL_VIEWS.get(self, 0)
        _ACTIVE_USERNAME_PASSWORD_CREDENTIAL_VIEWS[self] = previous_view_count + 1
        try:
            return callback(view)
        finally:
            try:
                view.close()
            finally:
                if previous_view_count:
                    _ACTIVE_USERNAME_PASSWORD_CREDENTIAL_VIEWS[self] = previous_view_count
                else:
                    _ACTIVE_USERNAME_PASSWORD_CREDENTIAL_VIEWS.pop(self, None)

    @synchronized_native_call
    def into_ttlv(self) -> Structure:
        """Transfer the original ordered TTLV Structure without rebuilding it."""
        if _ACTIVE_USERNAME_PASSWORD_CREDENTIAL_VIEWS.get(self, 0):
            errors.raise_for_status(4)
        handle = _new_handle(
            "kmipkit_ttlv_structure_t",
            lib.kmipkit_username_and_password_credential_into_ttlv,
            self,
            consumed=(0,),
            restore_on_statuses=(4,),
        )
        return _wrap_owned_handle(Structure, handle)


class HashedPasswordCredential(NativeHandle):
    """Validated Hashed Password Credential Value preserving its exact TTLV."""

    __slots__ = ("__weakref__",)

    _release_name = "kmipkit_hashed_password_credential_release"

    @classmethod
    def from_ttlv(cls, structure: Structure) -> HashedPasswordCredential:
        """Consume and validate a TTLV Structure using KMIP 2.1 Table 415 rules."""
        handle = _new_handle(
            "kmipkit_hashed_password_credential_t",
            lib.kmipkit_hashed_password_credential_from_ttlv,
            structure,
            consumed=(0,),
            restore_on_statuses=(errors._SHARED_OWNER_STATUS,),
            credential_validation_error=True,
        )
        return _wrap_owned_handle(cls, handle)

    @synchronized_native_call
    def hashing_algorithm_raw(self) -> int | None:
        """Return the explicit raw Hashing Algorithm, retaining unknown values."""
        present = ffi.new("uint8_t *")
        algorithm = ffi.new("uint32_t *")
        status = lib.kmipkit_hashed_password_credential_hashing_algorithm_raw(
            self._pointer(), present, algorithm
        )
        errors.raise_for_status(status)
        return int(algorithm[0]) if present[0] else None

    @synchronized_native_call
    def effective_hashing_algorithm_raw(self) -> int:
        """Return the effective raw algorithm without inserting an omitted field."""
        algorithm = ffi.new("uint32_t *")
        status = lib.kmipkit_hashed_password_credential_effective_hashing_algorithm_raw(
            self._pointer(), algorithm
        )
        errors.raise_for_status(status)
        return int(algorithm[0])

    @synchronized_native_call
    def with_ttlv(self, callback: Callable[[TtlvStructureView], _T]) -> _T:
        """Lend the original ordered TTLV and close the read-only view on return."""
        if not callable(callback):
            errors.raise_for_status(4)
        view_handle = _new_handle(
            "kmipkit_ttlv_structure_view_t",
            lib.kmipkit_hashed_password_credential_ttlv_view,
            self._pointer(),
        )
        view = _wrap_owned_handle(TtlvStructureView, view_handle, owner=self)
        previous_view_count = _ACTIVE_HASHED_PASSWORD_CREDENTIAL_VIEWS.get(self, 0)
        _ACTIVE_HASHED_PASSWORD_CREDENTIAL_VIEWS[self] = previous_view_count + 1
        try:
            return callback(view)
        finally:
            try:
                view.close()
            finally:
                if previous_view_count:
                    _ACTIVE_HASHED_PASSWORD_CREDENTIAL_VIEWS[self] = previous_view_count
                else:
                    _ACTIVE_HASHED_PASSWORD_CREDENTIAL_VIEWS.pop(self, None)

    @synchronized_native_call
    def into_ttlv(self) -> Structure:
        """Transfer the original ordered TTLV Structure without rebuilding it."""
        if _ACTIVE_HASHED_PASSWORD_CREDENTIAL_VIEWS.get(self, 0):
            errors.raise_for_status(4)
        handle = _new_handle(
            "kmipkit_ttlv_structure_t",
            lib.kmipkit_hashed_password_credential_into_ttlv,
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
    "HashedPasswordCredential",
    "Nonce",
    "SecretBytes",
    "SecretText",
    "UsernameAndPasswordCredential",
]
