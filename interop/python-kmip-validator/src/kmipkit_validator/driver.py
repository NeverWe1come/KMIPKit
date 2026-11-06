from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol
from .isolation import AttemptContext
from .models import TestCase


@dataclass(frozen=True)
class ExecutionOutcome:
    success: bool
    result_reason: str = ""


class KmipDriver(Protocol):
    def execute_case(self, test_case: TestCase, attempt: AttemptContext) -> ExecutionOutcome:
        ...


class MockKmipDriver:
    def execute_case(self, test_case: TestCase, attempt: AttemptContext) -> ExecutionOutcome:
        return ExecutionOutcome(success=True, result_reason="")
