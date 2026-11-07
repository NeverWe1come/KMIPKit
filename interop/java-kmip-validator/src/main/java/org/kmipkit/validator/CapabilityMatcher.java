package org.kmipkit.validator;

import org.kmipkit.validator.models.Models.ServerCapabilities;
import org.kmipkit.validator.models.Models.TestCase;

import java.util.ArrayList;
import java.util.List;

public class CapabilityMatcher {
    private final ServerCapabilities capabilities;

    public CapabilityMatcher(ServerCapabilities capabilities) {
        this.capabilities = capabilities;
    }

    public List<String> getUnsupportedReasons(TestCase testCase) {
        List<String> reasons = new ArrayList<>();
        if (testCase.operation() != null && !testCase.operation().isEmpty()) {
            if (!capabilities.supportedOperations().contains(testCase.operation())) {
                reasons.add("Operation '" + testCase.operation() + "' unsupported");
            }
        }
        if (testCase.objectType() != null && !testCase.objectType().isEmpty()) {
            if (!capabilities.supportedObjects().contains(testCase.objectType())) {
                reasons.add("Managed object '" + testCase.objectType() + "' unsupported");
            }
        }
        if (testCase.attribute() != null && !testCase.attribute().isEmpty()) {
            if (!capabilities.supportedAttributes().contains(testCase.attribute())) {
                reasons.add("Attribute '" + testCase.attribute() + "' unsupported");
            }
        }
        return reasons;
    }

    public boolean isSupported(TestCase testCase) {
        return getUnsupportedReasons(testCase).isEmpty();
    }
}
