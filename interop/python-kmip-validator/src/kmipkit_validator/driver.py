from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable, Protocol
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


@dataclass(frozen=True)
class NormalizedResponse:
    status: str
    result_reason: str = ""
    object_type: str = ""
    sanitized_payload_hex: str = ""


@dataclass(frozen=True)
class EquivalenceResult:
    is_equivalent: bool
    discrepancies: list[str]


class EquivalenceVerifier:
    @staticmethod
    def verify_equivalence(responses: dict[str, NormalizedResponse]) -> EquivalenceResult:
        discrepancies: list[str] = []
        if not responses or len(responses) <= 1:
            return EquivalenceResult(is_equivalent=True, discrepancies=discrepancies)

        items = list(responses.items())
        base_label, base_resp = items[0]
        for label, resp in items[1:]:
            if resp.status != base_resp.status:
                discrepancies.append(
                    f"[{label}] status '{resp.status}' differs from [{base_label}] '{base_resp.status}'"
                )
            if resp.result_reason != base_resp.result_reason:
                discrepancies.append(
                    f"[{label}] result_reason '{resp.result_reason}' differs from [{base_label}] '{base_resp.result_reason}'"
                )
            if resp.object_type != base_resp.object_type:
                discrepancies.append(
                    f"[{label}] object_type '{resp.object_type}' differs from [{base_label}] '{base_resp.object_type}'"
                )
            if resp.sanitized_payload_hex != base_resp.sanitized_payload_hex:
                discrepancies.append(f"[{label}] payload differs from [{base_label}]")

        return EquivalenceResult(is_equivalent=len(discrepancies) == 0, discrepancies=discrepancies)


class RecipeRunner:
    @staticmethod
    def execute_with_guaranteed_teardown(
        setup_fn: Callable[[], bool] | None,
        target_fn: Callable[[], Any] | None,
        teardown_fn: Callable[[], bool] | None,
    ) -> Any:
        if setup_fn and not setup_fn():
            raise RuntimeError("Setup phase failed")
        try:
            return target_fn() if target_fn else None
        finally:
            if teardown_fn:
                teardown_fn()
