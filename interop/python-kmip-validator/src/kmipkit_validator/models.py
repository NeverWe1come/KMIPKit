from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, List, Optional, Set


class CaseStatus(str, Enum):
    PASSED = "passed"
    FAILED = "failed"
    SERVER_UNSUPPORTED = "server_unsupported"
    POLICY_RESTRICTED = "policy_restricted"
    NOT_RUN = "not_run"
    # Compatibility aliases for the original preview-only model.
    UNSUPPORTED_SKIPPED = "not_run"
    UNSUPPORTED_VERIFIED_REJECTED = "server_unsupported"
    UNEXPECTED_SUCCESS = "failed"


@dataclass(frozen=True)
class ServerCapabilities:
    server_id: str
    supported_operations: Set[str] = field(default_factory=set)
    supported_objects: Set[str] = field(default_factory=set)
    supported_attributes: Set[str] = field(default_factory=set)


@dataclass(frozen=True)
class TestCase:
    case_id: str
    category: str
    operation: str
    object_type: str = ""
    attribute: str = ""


@dataclass(frozen=True)
class CaseResult:
    case_id: str
    operation: str
    status: CaseStatus
    detail: str = ""


@dataclass(frozen=True)
class ValidationSummary:
    server_id: str
    language: str
    total_cases: int
    passed: int
    failed: int
    unsupported: int
    results: List[CaseResult]


@dataclass(frozen=True)
class InteropCaseResult:
    case_id: str
    status: str
    library_correctness: str
    server_compatibility: str
    parity: str
    duration_ms: float = 0.0
    result_status_code: int | None = None
    result_reason_code: int | None = None
    diagnostic: str | None = None

    def as_dict(self) -> dict:
        result = {
            "case_id": self.case_id,
            "status": self.status,
            "library_correctness": self.library_correctness,
            "server_compatibility": self.server_compatibility,
            "parity": self.parity,
            "duration_ms": self.duration_ms,
        }
        for name in ("result_status_code", "result_reason_code", "diagnostic"):
            value = getattr(self, name)
            if value is not None:
                result[name] = value
        return result
