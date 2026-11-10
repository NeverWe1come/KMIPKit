package org.kmipkit.validator;

import org.kmipkit.extensions.ProtocolVersion;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class JavaResponseChecksTest {
    @Test
    void discoverVersionsMustIncludeTheConfiguredProtocolVersion() {
        var requested = new ProtocolVersion(2, 1);
        assertTrue(JavaResponseChecks.advertisesRequestedVersion(
                List.of(new ProtocolVersion(2, 0), requested), requested));
        assertFalse(JavaResponseChecks.advertisesRequestedVersion(
                List.of(new ProtocolVersion(2, 0)), requested));
        assertFalse(JavaResponseChecks.advertisesRequestedVersion(List.of(), requested));
    }

    @Test
    void queryOperationsFunctionMustReturnAtLeastOneOperation() {
        assertTrue(JavaResponseChecks.queryAdvertisesOperations(List.of(0x3bL)));
        assertFalse(JavaResponseChecks.queryAdvertisesOperations(List.of()));
    }
}
