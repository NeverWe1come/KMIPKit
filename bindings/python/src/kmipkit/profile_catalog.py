"""Immutable snapshots of the generated KMIP client profile metadata catalog."""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum

from . import errors
from ._client_ffi import ffi, lib

_CLIENT_PROFILES = 1
_CLIENT_REQUIREMENTS = 2
_PROFILE_TEST_CASES = 3
_DIRECT_SOURCE_CLAUSE_IDS = 1
_SOURCE_CLAUSE_IDS = 2
_REQUIREMENT_IDS = 3
_UNRESOLVED_SCOPE_OBLIGATIONS = 4
_DEPENDENCY_PROFILE_IDS = 5
_TRANSPORT_REQUIREMENTS = 6
_ENCODING_REQUIREMENTS = 7
_TEST_CASES = 8
_PROFILE_IDS = 9
_OFFICIAL_TEST_CASE_IDS = 10
_ROW_STRING = 5
_ROW_SCOPE_BLOCKER = 4
_ROW_TEST_CASE = 3


class VerificationCategory(StrEnum):
    """Category-specific verification obligation retained by the catalog."""

    REQUIRED_BEHAVIOR = "required_behavior"
    PROHIBITED_BEHAVIOR = "prohibited_behavior"
    RECOMMENDED_BEHAVIOR_OR_DEVIATION = "recommended_behavior_or_deviation"
    DISCOURAGED_ABSENCE_OR_DEVIATION = "discouraged_absence_or_deviation"
    OPTIONAL_CAPABILITY = "optional_capability"


@dataclass(frozen=True, slots=True)
class ProfileScopeBlocker:
    requirement_id: str
    source_clause_id: str
    scope_state: str


@dataclass(frozen=True, slots=True)
class ProfileTestCase:
    test_id: str
    official_case_id: str
    mandatory_status: str
    fixture_availability: str


@dataclass(frozen=True, slots=True)
class ClientProfile:
    profile_id: str
    name: str
    applicability: str
    claim_state: str
    selected_target: bool
    direct_source_clause_ids: tuple[str, ...]
    source_clause_ids: tuple[str, ...]
    requirement_ids: tuple[str, ...]
    unresolved_scope_obligations: tuple[ProfileScopeBlocker, ...]
    dependency_profile_ids: tuple[str, ...]
    transport_requirements: tuple[str, ...]
    encoding_requirements: tuple[str, ...]
    test_cases: tuple[ProfileTestCase, ...]


@dataclass(frozen=True, slots=True)
class ClientRequirement:
    requirement_id: str
    source_keyword: str | None
    source_force: str | None
    source_gap_id: str | None
    source_locator: str | None
    verification_category: VerificationCategory
    scope_state: str
    source_clause_ids: tuple[str, ...]
    profile_ids: tuple[str, ...]
    official_test_case_ids: tuple[str, ...]
    has_accepted_deviation: bool


def _decode(span: object) -> str:
    length = int(span.length)  # type: ignore[attr-defined]
    if length == 0:
        return ""
    return bytes(ffi.buffer(span.data, length)).decode("utf-8")  # type: ignore[attr-defined]


def _row(table: int, index: int) -> object:
    view = ffi.new("kmipkit_profile_catalog_row_view_v1_t *")
    view.struct_size = ffi.sizeof("kmipkit_profile_catalog_row_view_v1_t")
    view.abi_version = 1
    errors.raise_for_status(lib.kmipkit_profile_catalog_row_at_v1(table, index, view))
    return view[0]


def _table_count(table: int) -> int:
    count = ffi.new("uint64_t *")
    errors.raise_for_status(lib.kmipkit_profile_catalog_table_count_v1(table, count))
    return int(count[0])


def _relation_count(table: int, index: int, relation: int) -> int:
    count = ffi.new("uint64_t *")
    errors.raise_for_status(
        lib.kmipkit_profile_catalog_relation_count_v1(table, index, relation, count)
    )
    return int(count[0])


def _relation_row(table: int, index: int, relation: int, relation_index: int) -> object:
    view = ffi.new("kmipkit_profile_catalog_row_view_v1_t *")
    view.struct_size = ffi.sizeof("kmipkit_profile_catalog_row_view_v1_t")
    view.abi_version = 1
    errors.raise_for_status(
        lib.kmipkit_profile_catalog_relation_at_v1(table, index, relation, relation_index, view)
    )
    return view[0]


def _strings(table: int, index: int, relation: int) -> tuple[str, ...]:
    values = []
    for relation_index in range(_relation_count(table, index, relation)):
        row = _relation_row(table, index, relation, relation_index)
        if row.row_kind != _ROW_STRING:
            raise errors.InvalidInputError()
        values.append(_decode(row.text_value))
    return tuple(values)


def _test_case(row: object) -> ProfileTestCase:
    return ProfileTestCase(
        _decode(row.test_id),
        _decode(row.official_case_id),
        _decode(row.mandatory_status),
        _decode(row.fixture_availability),
    )


def client_profiles() -> tuple[ClientProfile, ...]:
    """Returns profiles and their ordered nested metadata as owned tuples."""
    result = []
    for index in range(_table_count(_CLIENT_PROFILES)):
        row = _row(_CLIENT_PROFILES, index)
        scope = []
        for nested_index in range(
            _relation_count(_CLIENT_PROFILES, index, _UNRESOLVED_SCOPE_OBLIGATIONS)
        ):
            nested = _relation_row(
                _CLIENT_PROFILES, index, _UNRESOLVED_SCOPE_OBLIGATIONS, nested_index
            )
            if nested.row_kind != _ROW_SCOPE_BLOCKER:
                raise errors.InvalidInputError()
            scope.append(
                ProfileScopeBlocker(
                    _decode(nested.requirement_id),
                    _decode(nested.source_clause_id),
                    _decode(nested.scope_state),
                )
            )
        cases = []
        for nested_index in range(_relation_count(_CLIENT_PROFILES, index, _TEST_CASES)):
            nested = _relation_row(_CLIENT_PROFILES, index, _TEST_CASES, nested_index)
            if nested.row_kind != _ROW_TEST_CASE:
                raise errors.InvalidInputError()
            cases.append(_test_case(nested))
        result.append(
            ClientProfile(
                _decode(row.profile_id),
                _decode(row.name),
                _decode(row.applicability),
                _decode(row.claim_state),
                bool(row.selected_target),
                _strings(_CLIENT_PROFILES, index, _DIRECT_SOURCE_CLAUSE_IDS),
                _strings(_CLIENT_PROFILES, index, _SOURCE_CLAUSE_IDS),
                _strings(_CLIENT_PROFILES, index, _REQUIREMENT_IDS),
                tuple(scope),
                _strings(_CLIENT_PROFILES, index, _DEPENDENCY_PROFILE_IDS),
                _strings(_CLIENT_PROFILES, index, _TRANSPORT_REQUIREMENTS),
                _strings(_CLIENT_PROFILES, index, _ENCODING_REQUIREMENTS),
                tuple(cases),
            )
        )
    return tuple(result)


def client_requirements() -> tuple[ClientRequirement, ...]:
    """Returns all requirement rows and ordered relationships from the generated catalog."""
    result = []
    for index in range(_table_count(_CLIENT_REQUIREMENTS)):
        row = _row(_CLIENT_REQUIREMENTS, index)
        result.append(
            ClientRequirement(
                _decode(row.requirement_id),
                _decode(row.source_keyword) if row.source_keyword_present else None,
                _decode(row.source_force) if row.source_force_present else None,
                _decode(row.source_gap_id) if row.source_gap_id_present else None,
                _decode(row.source_locator) if row.source_locator_present else None,
                VerificationCategory(_decode(row.verification_category)),
                _decode(row.scope_state),
                _strings(_CLIENT_REQUIREMENTS, index, _SOURCE_CLAUSE_IDS),
                _strings(_CLIENT_REQUIREMENTS, index, _PROFILE_IDS),
                _strings(_CLIENT_REQUIREMENTS, index, _OFFICIAL_TEST_CASE_IDS),
                bool(row.has_accepted_deviation),
            )
        )
    return tuple(result)


def profile_test_cases() -> tuple[ProfileTestCase, ...]:
    """Returns every generated profile test case in stable catalog order."""
    return tuple(_test_case(_row(_PROFILE_TEST_CASES, index)) for index in range(_table_count(_PROFILE_TEST_CASES)))


__all__ = [
    "ClientProfile",
    "ClientRequirement",
    "ProfileScopeBlocker",
    "ProfileTestCase",
    "VerificationCategory",
    "client_profiles",
    "client_requirements",
    "profile_test_cases",
]
