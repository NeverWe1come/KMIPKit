package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;

final class NativeAttributeRequestsCoverageTest {
    @Test
    void nativeAttributeRequestFactoryRejectsMalformedArraysAndReferences() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeAttributeRequests.create(0, 0, null, 0,
                        new int[0], new long[0], new byte[0][], new byte[0][]));
        assertThrows(InvalidInputException.class,
                () -> NativeAttributeRequests.create(5, 0, null, 0,
                        null, new long[0], new byte[0][], new byte[0][]));
        assertThrows(InvalidInputException.class,
                () -> NativeAttributeRequests.create(5, 0, null, 0,
                        new int[] {1}, new long[0], new byte[0][], new byte[0][]));
        assertThrows(InvalidInputException.class,
                () -> NativeAttributeRequests.create(6, 0, null, 0,
                        new int[] {1}, new long[] {1}, new byte[][] {null}, new byte[][] {null}));
        assertThrows(InvalidInputException.class,
                () -> NativeAttributeRequests.create(5, 0, null, 0,
                        new int[] {1}, new long[] {-1}, new byte[][] {null}, new byte[][] {null}));
        assertThrows(InvalidInputException.class,
                () -> NativeAttributeRequests.create(5, 0, null, 0,
                        new int[] {2}, new long[] {1},
                        new byte[][] {"KMIPKit".getBytes(java.nio.charset.StandardCharsets.UTF_8)},
                        new byte[][] {"Comment".getBytes(java.nio.charset.StandardCharsets.UTF_8)}));
        assertThrows(InvalidInputException.class,
                () -> NativeAttributeRequests.create(5, 0, null, 0,
                        new int[] {2}, new long[] {0}, new byte[][] {null},
                        new byte[][] {"Comment".getBytes(java.nio.charset.StandardCharsets.UTF_8)}));
        assertThrows(InvalidInputException.class,
                () -> NativeAttributeRequests.create(5, 0, null, 0,
                        new int[] {2}, new long[] {0}, new byte[][] {new byte[] {(byte) 0xff}},
                        new byte[][] {"Comment".getBytes(java.nio.charset.StandardCharsets.UTF_8)}));
        assertThrows(InvalidInputException.class,
                () -> NativeAttributeRequests.create(5, 0, null, 0,
                        new int[] {99}, new long[] {0}, new byte[][] {null}, new byte[][] {null}));
        assertThrows(InvalidInputException.class,
                () -> NativeAttributeRequests.create(5, 99, null, 0,
                        new int[0], new long[0], new byte[0][], new byte[0][]));
    }

    @Test
    void nativeAttributeRequestFactoryEnforcesTheReferenceCountLimitBeforeAllocation() {
        NativeExtensionRegistry.ensureLoaded();
        int[] kinds = new int[100_001];
        long[] tags = new long[kinds.length];
        byte[][] vendors = new byte[kinds.length][];
        byte[][] names = new byte[kinds.length][];

        assertThrows(ResourceLimitException.class,
                () -> NativeAttributeRequests.create(5, 0, null, 0,
                        kinds, tags, vendors, names));
    }
}
