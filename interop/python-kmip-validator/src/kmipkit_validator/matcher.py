from __future__ import annotations

from typing import List
from .models import ServerCapabilities, TestCase


class CapabilityMatcher:
    def __init__(self, capabilities: ServerCapabilities) -> None:
        self.capabilities = capabilities

    def get_unsupported_reasons(self, test_case: TestCase) -> List[str]:
        reasons: List[str] = []
        if test_case.operation and test_case.operation not in self.capabilities.supported_operations:
            reasons.append(f"Operation '{test_case.operation}' unsupported")
        if test_case.object_type and test_case.object_type not in self.capabilities.supported_objects:
            reasons.append(f"Managed object '{test_case.object_type}' unsupported")
        if test_case.attribute and test_case.attribute not in self.capabilities.supported_attributes:
            reasons.append(f"Attribute '{test_case.attribute}' unsupported")
        return reasons

    def is_supported(self, test_case: TestCase) -> bool:
        return len(self.get_unsupported_reasons(test_case)) == 0
