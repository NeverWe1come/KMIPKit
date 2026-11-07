from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, List, Optional, Set


class CaseStatus(str, Enum):
    PASSED = "passed"
    FAILED = "failed"
    UNSUPPORTED_SKIPPED = "unsupported_skipped"
    UNSUPPORTED_VERIFIED_REJECTED = "unsupported_verified_rejected"
    UNEXPECTED_SUCCESS = "unexpected_success"


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
