"""Caller-local profile evidence evaluation and redacted readiness snapshots."""

from __future__ import annotations

from dataclasses import dataclass

from . import errors
from ._client_ffi import ffi, lib

_MAX_RECORDS = 100_000
_MAX_PROFILE_ID_BYTES = 4 * 1024
_MAX_UTF8_BYTES = 16 * 1024 * 1024


def _utf8_length_up_to(value: str, maximum: int) -> int:
    """Count UTF-8 bytes without allocating, stopping after the configured bound."""
    if len(value) > maximum:
        return maximum + 1
    if value.isascii():
        return len(value)
    total = 0
    for character in value:
        codepoint = ord(character)
        total += 1 if codepoint <= 0x7F else 2 if codepoint <= 0x7FF else 3 if codepoint <= 0xFFFF else 4
        if total > maximum:
            return maximum + 1
    return total


def _decode(span: object) -> str:
    length = int(span.length)  # type: ignore[attr-defined]
    if length == 0:
        return ""
    return bytes(ffi.buffer(span.data, length)).decode("utf-8")  # type: ignore[attr-defined]


@dataclass(frozen=True, slots=True)
class ProfileEvidenceRecord:
    """One caller-supplied pass/fail record for a stable requirement identifier."""

    requirement_id: str
    passed_value: bool

    def __post_init__(self) -> None:
        if not isinstance(self.requirement_id, str) or not isinstance(self.passed_value, bool):
            raise errors.InvalidInputError()

    @classmethod
    def passed(cls, requirement_id: str) -> ProfileEvidenceRecord:
        return cls(requirement_id, True)

    @classmethod
    def failed(cls, requirement_id: str) -> ProfileEvidenceRecord:
        return cls(requirement_id, False)

@dataclass(frozen=True, slots=True)
class ProfileReadinessBlocker:
    """One redacted blocker with stable optional catalog identifiers."""

    kind: str
    message: str
    requirement_id: str | None
    profile_id: str | None
    test_id: str | None
    source_clause_id: str | None


@dataclass(frozen=True, slots=True)
class ProfileReadiness:
    """An owned immutable snapshot returned by the canonical Rust evaluator."""

    profile_id: str
    target_selected: bool
    runtime_validation_available: bool
    effective_state: str
    requirement_records_complete: bool
    published_claim: bool
    blockers: tuple[ProfileReadinessBlocker, ...]

    @classmethod
    def evaluate(
        cls, profile_id: str, records: tuple[ProfileEvidenceRecord, ...] | list[ProfileEvidenceRecord]
    ) -> ProfileReadiness | None:
        if not isinstance(profile_id, str) or not isinstance(records, (tuple, list)):
            raise errors.InvalidInputError()
        if len(records) > _MAX_RECORDS:
            raise errors.ResourceLimitError()
        if any(not isinstance(record, ProfileEvidenceRecord) for record in records):
            raise errors.InvalidInputError()
        profile_length = _utf8_length_up_to(profile_id, _MAX_PROFILE_ID_BYTES)
        if profile_length > _MAX_PROFILE_ID_BYTES:
            raise errors.ResourceLimitError()
        total_length = profile_length
        for record in records:
            remaining = _MAX_UTF8_BYTES - total_length
            requirement_length = _utf8_length_up_to(record.requirement_id, remaining)
            if requirement_length > remaining:
                raise errors.ResourceLimitError()
            total_length += requirement_length

        try:
            profile_bytes = profile_id.encode("utf-8")
            encoded = [record.requirement_id.encode("utf-8") for record in records]
        except UnicodeEncodeError:
            raise errors.InvalidInputError() from None

        record_array = ffi.new("kmipkit_profile_evidence_record_v1_t[]", len(records))
        keepalive: list[object] = []
        for index, (record, value) in enumerate(zip(records, encoded, strict=True)):
            buffer = ffi.new("uint8_t[]", value) if value else ffi.NULL
            keepalive.append(buffer)
            record_array[index].struct_size = ffi.sizeof("kmipkit_profile_evidence_record_v1_t")
            record_array[index].abi_version = 1
            record_array[index].requirement_id.data = buffer
            record_array[index].requirement_id.length = len(value)
            record_array[index].passed = int(record.passed_value)
        profile_buffer = ffi.new("uint8_t[]", profile_bytes) if profile_bytes else ffi.NULL
        keepalive.append(profile_buffer)
        span = ffi.new("kmipkit_byte_span_v1_t *")
        span.data = profile_buffer
        span.length = len(profile_bytes)
        handle = ffi.new("kmipkit_profile_readiness_t **")
        status = lib.kmipkit_profile_readiness_evaluate_v1(
            span, record_array, len(records), handle
        )
        errors.raise_for_status(status)
        if handle[0] == ffi.NULL:
            return None
        try:
            view = ffi.new("kmipkit_profile_readiness_view_v1_t *")
            view.struct_size = ffi.sizeof("kmipkit_profile_readiness_view_v1_t")
            view.abi_version = 1
            errors.raise_for_status(lib.kmipkit_profile_readiness_view_get_v1(handle[0], view))
            blockers: list[ProfileReadinessBlocker] = []
            for index in range(int(view.blocker_count)):
                blocker = ffi.new("kmipkit_profile_readiness_blocker_view_v1_t *")
                blocker.struct_size = ffi.sizeof("kmipkit_profile_readiness_blocker_view_v1_t")
                blocker.abi_version = 1
                errors.raise_for_status(
                    lib.kmipkit_profile_readiness_blocker_at_v1(handle[0], index, blocker)
                )
                blockers.append(
                    ProfileReadinessBlocker(
                        _decode(blocker.kind),
                        _decode(blocker.message),
                        _decode(blocker.requirement_id) if blocker.requirement_id_present else None,
                        _decode(blocker.profile_id) if blocker.profile_id_present else None,
                        _decode(blocker.test_id) if blocker.test_id_present else None,
                        _decode(blocker.source_clause_id)
                        if blocker.source_clause_id_present
                        else None,
                    )
                )
            return cls(
                _decode(view.profile_id),
                bool(view.target_selected),
                bool(view.runtime_validation_available),
                _decode(view.effective_state),
                bool(view.requirement_records_complete),
                bool(view.published_claim),
                tuple(blockers),
            )
        finally:
            lib.kmipkit_profile_readiness_release(handle[0])


__all__ = ["ProfileEvidenceRecord", "ProfileReadiness", "ProfileReadinessBlocker"]
