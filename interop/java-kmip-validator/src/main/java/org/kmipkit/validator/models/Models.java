package org.kmipkit.validator.models;

import java.util.List;
import java.util.Set;

public class Models {
    public enum CaseStatus {
        PASSED,
        FAILED,
        UNSUPPORTED_SKIPPED,
        UNSUPPORTED_VERIFIED_REJECTED,
        UNEXPECTED_SUCCESS
    }

    public record ServerCapabilities(
        String serverId,
        Set<String> supportedOperations,
        Set<String> supportedObjects,
        Set<String> supportedAttributes
    ) {}

    public record TestCase(
        String caseId,
        String category,
        String operation,
        String objectType,
        String attribute
    ) {}

    public record CaseResult(
        String caseId,
        String operation,
        CaseStatus status,
        String detail
    ) {}

    public record ValidationSummary(
        String serverId,
        String language,
        int totalCases,
        int passed,
        int failed,
        int unsupported,
        List<CaseResult> results
    ) {}
}
