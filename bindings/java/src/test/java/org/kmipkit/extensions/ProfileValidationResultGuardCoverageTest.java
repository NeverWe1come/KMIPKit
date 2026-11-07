package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.TtlvStructure;

final class ProfileValidationResultGuardCoverageTest {
    @Test
    void rejectsInvalidHandlesAndReleasesTheHandleWhenEitherSnapshotIsInvalid() {
        FakeBridge bridge = new FakeBridge();
        assertThrows(InvalidInputException.class, () -> new ProfileValidationResult(0, bridge));
        assertThrows(InvalidInputException.class, () -> new ProfileValidationResult(71, null));

        bridge.profileCount = -1;
        assertThrows(InvalidInputException.class, () -> new ProfileValidationResult(72, bridge));
        assertEquals(1, bridge.releaseCount);

        bridge.profileCount = 4_097;
        assertThrows(ResourceLimitException.class, () -> new ProfileValidationResult(73, bridge));
        assertEquals(2, bridge.releaseCount);

        bridge.profileCount = 1;
        bridge.profileId = null;
        assertThrows(InvalidInputException.class, () -> new ProfileValidationResult(74, bridge));
        assertEquals(3, bridge.releaseCount);

        bridge.profileId = "profile";
        bridge.requirementCount = 100_001;
        assertThrows(ResourceLimitException.class, () -> new ProfileValidationResult(75, bridge));
        assertEquals(4, bridge.releaseCount);

        bridge.requirementCount = 1;
        bridge.requirementId = null;
        assertThrows(InvalidInputException.class, () -> new ProfileValidationResult(76, bridge));
        assertEquals(5, bridge.releaseCount);
    }

    @Test
    void nullValidationArgumentsAreRejectedAndCloseReleasesOnlyOnce() {
        FakeBridge bridge = new FakeBridge();
        ProfileValidationResult result = new ProfileValidationResult(77, bridge);

        assertThrows(InvalidInputException.class, () -> result.validateTransport(null));
        assertThrows(InvalidInputException.class, () -> result.validateRequest(null));
        assertThrows(InvalidInputException.class, () -> result.validateRequestWithContext(null, null));
        assertThrows(InvalidInputException.class,
                () -> result.validateRequestWithContext(null, new ProfileValidationContext()));
        assertThrows(InvalidInputException.class, () -> result.validateResponse(null, null));
        assertThrows(InvalidInputException.class,
                () -> result.validateResponseWithContext(null, null, null));

        assertEquals(java.util.List.of(), result.profileIds());
        assertEquals(java.util.List.of(), result.requirementIds());
        assertEquals("ProfileValidationResult[profileCount=0, requirementCount=0]", result.toString());
        result.close();
        result.close();
        assertEquals(1, bridge.releaseCount);
        assertThrows(InvalidInputException.class,
                () -> result.validateTransport(ProfileTransport.HTTPS));
    }

    @Test
    void forwardsValidValidationCallsWithAndWithoutCallerContext() {
        FakeBridge bridge = new FakeBridge();
        ProfileValidationContext context = new ProfileValidationContext()
                .withObjectState(2, new ProfileObjectState("object-2"));
        try (ProfileValidationResult result = new ProfileValidationResult(78, bridge);
                TtlvStructure request = TtlvStructure.create();
                TtlvStructure response = TtlvStructure.create()) {
            result.validateTransport(ProfileTransport.RAW_TTLV_TLS);
            result.validateRequest(request);
            result.validateRequestWithContext(request, context);
            result.validateResponse(request, response);
            result.validateResponseWithContext(request, response, context);
        }
        assertEquals(1, bridge.transportCalls);
        assertEquals(1, bridge.requestCalls);
        assertEquals(1, bridge.requestContextCalls);
        assertEquals(1, bridge.responseCalls);
        assertEquals(1, bridge.responseContextCalls);
        assertEquals(1, bridge.releaseCount);
    }

    @Test
    void contextBridgeArraysPreserveKnownFieldsAndOptionalBooleanValues() throws Exception {
        ProfileObjectState complete = new ProfileObjectState("complete")
                .withOriginalCreationDatePresent(true)
                .withUsageLimitsPresent(false)
                .withProtectionUseStarted(true)
                .withGetUsageAllocationPerformed(false)
                .withGetUsageAllocationChangedCount(true)
                .withUsageLimitsTotal(12L)
                .withUsageLimitsCount(4L);
        ProfileObjectState partial = new ProfileObjectState("partial")
                .withUsageLimitsTotal(0L);
        ProfileValidationContext context = new ProfileValidationContext()
                .withObjectState(3, complete)
                .withObjectState(8, partial);

        Class<?> arraysType = Class.forName(
                "org.kmipkit.extensions.ProfileValidationResult$ProfileContextArrays");
        Method from = arraysType.getDeclaredMethod("from", ProfileValidationContext.class);
        from.setAccessible(true);
        Object arrays = from.invoke(null, context);

        assertArrayEquals(new long[] {3, 8}, (long[]) field(arraysType, arrays, "indexes"));
        assertArrayEquals(new String[] {"complete", "partial"},
                (String[]) field(arraysType, arrays, "identifiers"));
        assertArrayEquals(new int[] {127, 32}, (int[]) field(arraysType, arrays, "knownFields"));
        assertArrayEquals(new int[] {21, 0}, (int[]) field(arraysType, arrays, "booleanValues"));
        assertArrayEquals(new long[] {12, 0}, (long[]) field(arraysType, arrays, "usageLimitsTotals"));
        assertArrayEquals(new long[] {4, 0}, (long[]) field(arraysType, arrays, "usageLimitsCounts"));

        InvocationTargetException invalidContext = org.junit.jupiter.api.Assertions.assertThrows(
                InvocationTargetException.class, () -> from.invoke(null, new Object[] {null}));
        assertEquals(InvalidInputException.class, invalidContext.getCause().getClass());
    }

    private static Object field(Class<?> type, Object instance, String name)
            throws ReflectiveOperationException {
        Field field = type.getDeclaredField(name);
        field.setAccessible(true);
        return field.get(instance);
    }

    private static final class FakeBridge implements ProfileValidationResult.Bridge {
        private long profileCount;
        private long requirementCount;
        private String profileId = "profile";
        private String requirementId = "requirement";
        private int releaseCount;
        private int transportCalls;
        private int requestCalls;
        private int requestContextCalls;
        private int responseCalls;
        private int responseContextCalls;

        @Override public long profileIdCount(long handle) { return profileCount; }
        @Override public String profileIdAt(long handle, long index) { return profileId; }
        @Override public long requirementIdCount(long handle) { return requirementCount; }
        @Override public String requirementIdAt(long handle, long index) { return requirementId; }
        @Override public void validateTransport(long handle, int transport) { transportCalls++; }
        @Override public void validateRequest(long handle, long request) { requestCalls++; }
        @Override public void validateRequestWithContext(
                long handle, long request, ProfileValidationContext context) { requestContextCalls++; }
        @Override public void validateResponse(long handle, long request, long response) { responseCalls++; }
        @Override public void validateResponseWithContext(
                long handle, long request, long response, ProfileValidationContext context) {
            responseContextCalls++;
        }
        @Override public void release(long handle) { releaseCount++; }
    }
}
