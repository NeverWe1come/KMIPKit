"""OASIS KMIP protocol utilities: version gating, link normalization, NIST lifecycle, and lease context."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Self

from . import errors, ttlv
from ._client_ffi import ffi, lib
from ._ffi import ffi as _base_ffi
from ._handles import (
    _NATIVE_CALL_LOCK,
    NativeHandle,
    _integer_range,
    _wrap_owned_handle,
    synchronized_native_call,
)
from .client import ProtocolVersion
from .extensions import UniqueIdentifier, UniqueIdentifierKind, _native_unique_identifier as _c_unique_identifier
from .protocol_values import EphemeralMode


@dataclass(frozen=True, slots=True)
class VersionSpan:
    """KMIP protocol version range for an operation or tag."""

    introduced_major: int
    introduced_minor: int
    deprecated: ProtocolVersion | None = None
    removed: ProtocolVersion | None = None


class VersionGate:
    """OASIS KMIP version gating for operations, tags, and attribute mutations."""

    @staticmethod
    def validate_operation(major: int, minor: int, operation: int) -> bool:
        """Validate whether an operation is supported in the given KMIP version."""
        raw_op = _integer_range(operation, 0, 0xFFFF_FFFF)
        status = lib.kmipkit_version_gate_validate_operation(int(major), int(minor), raw_op)
        return status == 0

    @staticmethod
    def validate_tag(major: int, minor: int, tag: int) -> bool:
        """Validate whether a tag is supported in the given KMIP version."""
        raw_tag = _integer_range(tag, 0, 0xFFFF_FFFF)
        status = lib.kmipkit_version_gate_validate_tag(int(major), int(minor), raw_tag)
        return status == 0

    @staticmethod
    def validate_attribute_mutation(major: int, minor: int, attribute_tag: int) -> bool:
        """Validate whether an attribute can be mutated in the given KMIP version."""
        raw_tag = _integer_range(attribute_tag, 0, 0xFFFF_FFFF)
        status = lib.kmipkit_version_gate_validate_attribute_mutation(
            int(major), int(minor), raw_tag
        )
        return status == 0

    @staticmethod
    def operation_span(operation: int) -> VersionSpan | None:
        """Return the version lifecycle span for an operation, or None if unknown."""
        raw_op = _integer_range(operation, 0, 0xFFFF_FFFF)
        intro_maj = ffi.new("uint32_t *")
        intro_min = ffi.new("uint32_t *")
        depr_maj = ffi.new("uint32_t *")
        depr_min = ffi.new("uint32_t *")
        rem_maj = ffi.new("uint32_t *")
        rem_min = ffi.new("uint32_t *")
        status = lib.kmipkit_version_gate_operation_span(
            raw_op, intro_maj, intro_min, depr_maj, depr_min, rem_maj, rem_min
        )
        if status != 0:
            return None
        depr = (
            ProtocolVersion(int(depr_maj[0]), int(depr_min[0]))
            if (depr_maj[0] != 0 or depr_min[0] != 0)
            else None
        )
        rem = (
            ProtocolVersion(int(rem_maj[0]), int(rem_min[0]))
            if (rem_maj[0] != 0 or rem_min[0] != 0)
            else None
        )
        return VersionSpan(
            introduced_major=int(intro_maj[0]),
            introduced_minor=int(intro_min[0]),
            deprecated=depr,
            removed=rem,
        )

    @staticmethod
    def tag_span(tag: int) -> VersionSpan | None:
        """Return the version lifecycle span for a tag, or None if unknown."""
        raw_tag = _integer_range(tag, 0, 0xFFFF_FFFF)
        intro_maj = ffi.new("uint32_t *")
        intro_min = ffi.new("uint32_t *")
        depr_maj = ffi.new("uint32_t *")
        depr_min = ffi.new("uint32_t *")
        rem_maj = ffi.new("uint32_t *")
        rem_min = ffi.new("uint32_t *")
        status = lib.kmipkit_version_gate_tag_span(
            raw_tag, intro_maj, intro_min, depr_maj, depr_min, rem_maj, rem_min
        )
        if status != 0:
            return None
        depr = (
            ProtocolVersion(int(depr_maj[0]), int(depr_min[0]))
            if (depr_maj[0] != 0 or depr_min[0] != 0)
            else None
        )
        rem = (
            ProtocolVersion(int(rem_maj[0]), int(rem_min[0]))
            if (rem_maj[0] != 0 or rem_min[0] != 0)
            else None
        )
        return VersionSpan(
            introduced_major=int(intro_maj[0]),
            introduced_minor=int(intro_min[0]),
            deprecated=depr,
            removed=rem,
        )


class LinkNormalizer:
    """OASIS KMIP link normalizer between KMIP 2.x and 3.0 representations."""

    @staticmethod
    def v2_to_v3(item: ttlv.TtlvItem) -> ttlv.TtlvItem:
        """Convert a KMIP 2.x Link TTLV item to its normalized KMIP 3.0 form. Consumes input item."""
        if not isinstance(item, ttlv.TtlvItem):
            errors.raise_for_status(4)
        out = ffi.new("kmipkit_ttlv_item_t **")
        try:
            with _NATIVE_CALL_LOCK:
                ptr = item._pointer()
                status = lib.kmipkit_link_normalizer_v2_to_v3(ptr, out)
            errors.raise_for_status(status)
            return _wrap_owned_handle(ttlv.TtlvItem, out[0])
        finally:
            item.close()

    @staticmethod
    def v3_to_v2(item: ttlv.TtlvItem) -> ttlv.TtlvItem:
        """Convert a KMIP 3.0 Link TTLV item to its normalized KMIP 2.x form. Consumes input item."""
        if not isinstance(item, ttlv.TtlvItem):
            errors.raise_for_status(4)
        out = ffi.new("kmipkit_ttlv_item_t **")
        try:
            with _NATIVE_CALL_LOCK:
                ptr = item._pointer()
                status = lib.kmipkit_link_normalizer_v3_to_v2(ptr, out)
            errors.raise_for_status(status)
            return _wrap_owned_handle(ttlv.TtlvItem, out[0])
        finally:
            item.close()


class NistKeyLifecycle:
    """NIST SP 800-57 key lifecycle cryptographic usage state validation."""

    @staticmethod
    def can_originate(state_enum: int) -> bool:
        """Return True if the state permits cryptographic origination (e.g., encrypt, sign)."""
        raw = _integer_range(state_enum, 0, 0xFFFF_FFFF)
        out = ffi.new("uint8_t *")
        status = lib.kmipkit_nist_key_lifecycle_can_originate(raw, out)
        errors.raise_for_status(status)
        return bool(out[0])

    @staticmethod
    def can_process(state_enum: int) -> bool:
        """Return True if the state permits cryptographic processing (e.g., decrypt, verify)."""
        raw = _integer_range(state_enum, 0, 0xFFFF_FFFF)
        out = ffi.new("uint8_t *")
        status = lib.kmipkit_nist_key_lifecycle_can_process(raw, out)
        errors.raise_for_status(status)
        return bool(out[0])


class _ProtocolHandle(NativeHandle):
    """Native handle released through the client CFFI library."""

    def close(self) -> None:
        with _NATIVE_CALL_LOCK:
            if self.closed:
                return
            handle, owner = self._take()
            getattr(lib, self._release_name)(handle)
            del owner


class LeaseContext(_ProtocolHandle):
    """Client lease lifecycle context tracking key usage authorization expiration."""

    _release_name = "kmipkit_lease_context_release"

    @classmethod
    def create(
        cls,
        unique_identifier: UniqueIdentifier,
        obtained_at_unix_secs: int,
        lease_time_secs: int,
    ) -> Self:
        """Create a client lease context tracking expiration for a leased object."""
        if not isinstance(unique_identifier, UniqueIdentifier):
            errors.raise_for_status(4)
        record, owner = _c_unique_identifier(unique_identifier)
        client_record = ffi.cast(
            "kmipkit_unique_identifier_v1_t *",
            int(_base_ffi.cast("uintptr_t", record)),
        )
        out = ffi.new("kmipkit_lease_context_t **")
        status = lib.kmipkit_lease_context_create(
            client_record,
            int(obtained_at_unix_secs),
            int(lease_time_secs),
            out,
        )
        del owner
        errors.raise_for_status(status)
        return _wrap_owned_handle(cls, out[0])

    @synchronized_native_call
    def unique_identifier(self) -> UniqueIdentifier:
        """Return the unique identifier associated with this lease."""
        out = ffi.new("kmipkit_unique_identifier_v1_t *")
        errors.raise_for_status(
            lib.kmipkit_lease_context_unique_identifier(self._pointer(), out)
        )
        kind = int(out.kind)
        if kind in (int(UniqueIdentifierKind.TEXT_STRING), int(UniqueIdentifierKind.IDENTIFIER), int(UniqueIdentifierKind.REFERENCE), int(UniqueIdentifierKind.NAME_REFERENCE)):
            text = (
                bytes(ffi.buffer(out.text_data, int(out.text_length))).decode(
                    "utf-8", errors="strict"
                )
                if out.text_length
                else ""
            )
            return UniqueIdentifier(UniqueIdentifierKind(kind), text=text)
        if kind == int(UniqueIdentifierKind.ENUMERATION):
            return UniqueIdentifier.enumeration(int(out.enumeration_value))
        if kind == int(UniqueIdentifierKind.INTEGER):
            return UniqueIdentifier.integer(int(out.integer_value))
        errors.raise_for_status(4)

    @synchronized_native_call
    def expires_at(self) -> int:
        """Return the Unix timestamp (seconds) when this lease expires."""
        out = ffi.new("int64_t *")
        errors.raise_for_status(
            lib.kmipkit_lease_context_expires_at(self._pointer(), out)
        )
        return int(out[0])

    @synchronized_native_call
    def is_expired(self, current_time_unix_secs: int) -> bool:
        """Return True if the lease is expired at the given Unix timestamp."""
        out = ffi.new("uint8_t *")
        errors.raise_for_status(
            lib.kmipkit_lease_context_is_expired(
                self._pointer(), int(current_time_unix_secs), out
            )
        )
        return bool(out[0])

    @synchronized_native_call
    def check_valid(self, current_time_unix_secs: int) -> None:
        """Verify that the lease is still valid; raises an error if expired."""
        status = lib.kmipkit_lease_context_check_valid(
            self._pointer(), int(current_time_unix_secs)
        )
        errors.raise_for_status(status)


__all__ = [
    "EphemeralMode",
    "VersionSpan",
    "VersionGate",
    "LinkNormalizer",
    "NistKeyLifecycle",
    "LeaseContext",
]
