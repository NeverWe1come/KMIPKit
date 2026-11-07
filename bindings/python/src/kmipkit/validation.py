"""Redacted access to the shared Rust request and schema-aware TTLV validator."""

from __future__ import annotations

from dataclasses import dataclass

from . import errors
from ._ffi import ffi, lib
from ._handles import NativeHandle, _NATIVE_CALL_LOCK, _new_handle, _wrap_owned_handle
from .ttlv import Structure


@dataclass(frozen=True, slots=True)
class ValidationIssue:
    """Value-free, stable metadata describing one validation finding."""

    code: str
    path: str
    severity: str
    source_identifier: str
    source_reference: str


class ValidationReport(NativeHandle):
    """Owned validation findings produced by the shared Rust validator."""

    _release_name = "kmipkit_validation_report_release"

    @classmethod
    def _from_native_handle(cls, handle: object) -> ValidationReport:
        return _wrap_owned_handle(cls, handle)

    @property
    def is_valid(self) -> bool:
        with _NATIVE_CALL_LOCK:
            output = ffi.new("uint8_t *")
            errors.raise_for_status(
                int(lib.kmipkit_validation_report_is_valid(self._pointer(), output))
            )
            return bool(output[0])

    @property
    def issues(self) -> tuple[ValidationIssue, ...]:
        with _NATIVE_CALL_LOCK:
            count = ffi.new("uint64_t *")
            errors.raise_for_status(
                int(lib.kmipkit_validation_report_issue_count(self._pointer(), count))
            )
            if count[0] > 100_000:
                errors.raise_for_status(6)
            values: list[ValidationIssue] = []
            for index in range(int(count[0])):
                view = ffi.new("kmipkit_validation_issue_view_v1_t *")
                view.struct_size = ffi.sizeof("kmipkit_validation_issue_view_v1_t")
                view.abi_version = 1
                errors.raise_for_status(
                    int(
                        lib.kmipkit_validation_report_issue_at_v1(
                            self._pointer(), index, view
                        )
                    )
                )
                severity = {1: "ERROR", 2: "WARNING", 3: "INFO"}.get(
                    int(view.severity), "UNKNOWN"
                )
                values.append(
                    ValidationIssue(
                        _span_text(view.code),
                        _span_text(view.path),
                        severity,
                        _span_text(view.source_identifier),
                        _span_text(view.source_reference),
                    )
                )
            return tuple(values)

    def __repr__(self) -> str:
        return f"ValidationReport(closed={self.closed})"


def _span_text(span: object) -> str:
    length = int(span.length)
    if length == 0:
        return ""
    if span.data == ffi.NULL or length > 4096:
        errors.raise_for_status(4 if span.data == ffi.NULL else 6)
    try:
        return bytes(ffi.buffer(span.data, length)).decode("utf-8", errors="strict")
    except UnicodeDecodeError:
        errors.raise_for_status(4)


def validate_structure(
    structure: Structure, *, schema: str | None = None
) -> ValidationReport:
    """Validate a generic Structure without inferring a schema when omitted.

    Known schemas are identified as ``RequestPayload/<Operation>`` or
    ``Nested/<Structure>``. The special ``Unsupported`` identity asks the
    shared validator to return its standard unsupported-schema finding.
    """
    if not isinstance(structure, Structure) or (schema is not None and not isinstance(schema, str)):
        errors.raise_for_status(4)
    if schema is None:
        encoded = b""
    else:
        try:
            encoded = schema.encode("utf-8", errors="strict")
        except UnicodeEncodeError:
            errors.raise_for_status(4)
        if len(encoded) > 128:
            errors.raise_for_status(6)
    data = ffi.NULL if not encoded else ffi.new("uint8_t[]", encoded)
    with _NATIVE_CALL_LOCK:
        handle = _new_handle(
            "kmipkit_validation_report_t",
            lib.kmipkit_ttlv_structure_validation_report_v1,
            structure._pointer(),
            data,
            len(encoded),
        )
    return ValidationReport._from_native_handle(handle)


def validate_request(item: object) -> ValidationReport:
    """Validate a typed client batch item with the shared Rust request validator."""
    from .extensions import ClientBatchItem

    if not isinstance(item, ClientBatchItem):
        errors.raise_for_status(4)
    with _NATIVE_CALL_LOCK:
        handle = _new_handle(
            "kmipkit_validation_report_t",
            lib.kmipkit_client_batch_item_validation_report,
            item._pointer(),
        )
    return ValidationReport._from_native_handle(handle)


__all__ = ["ValidationIssue", "ValidationReport", "validate_request", "validate_structure"]
