package org.kmipkit.validator;

import org.kmipkit.validator.models.Models.TestCase;

public interface KmipDriver {
    record ExecutionOutcome(boolean success, String resultReason) {}

    ExecutionOutcome executeCase(TestCase testCase, AttemptContext attempt);

    static KmipDriver mockDriver() {
        return (testCase, attempt) -> new ExecutionOutcome(true, "");
    }
}
