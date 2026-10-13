package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;

/** Exercises transport-material defenses at the JNI boundary rather than the Java builder. */
final class NativeTransportConfigurationBoundaryTest {
    private static final int MAX_TRANSPORT_INPUT_BYTES = 16 * 1024 * 1024;

    @Test
    void rejectsNullAndMismatchedParallelArraysAtNativeBoundary() {
        assertThrows(InvalidInputException.class, () -> create(
                null, new int[0], new byte[0][], new long[12]));
        assertThrows(InvalidInputException.class, () -> create(
                new int[] {1}, new int[0], new byte[0][], new long[12]));
        assertThrows(InvalidInputException.class, () -> create(
                new int[] {1}, new int[] {1}, new byte[0][], new long[12]));
    }

    @Test
    void rejectsUnsupportedMaterialKindsAndNullElementsAtNativeBoundary() {
        assertThrows(InvalidInputException.class, () -> create(
                new int[] {99}, new int[] {1}, new byte[][] {{1}}, new long[12]));
        assertThrows(InvalidInputException.class, () -> create(
                new int[] {1}, new int[] {99}, new byte[][] {{1}}, new long[12]));
        assertThrows(InvalidInputException.class, () -> create(
                new int[] {1}, new int[] {1}, new byte[][] {null}, new long[12]));
        assertThrows(InvalidInputException.class, () -> create(
                new int[] {1}, new int[] {1}, new byte[][] {new byte[0]}, new long[12]));
    }

    @Test
    void rejectsOversizedIndividualTransportInputBeforeCopyingIt() {
        byte[][] oversized = {new byte[MAX_TRANSPORT_INPUT_BYTES + 1]};

        assertThrows(ResourceLimitException.class, () -> create(
                new int[] {1}, new int[] {1}, oversized, new long[12]));
    }

    @Test
    void rejectsAggregateTransportInputOverTheNativeBridgeBudget() {
        byte[][] inputs = {
            new byte[MAX_TRANSPORT_INPUT_BYTES / 2],
            new byte[MAX_TRANSPORT_INPUT_BYTES / 2 + 1]
        };

        assertThrows(ResourceLimitException.class, () -> create(
                new int[] {1, 1}, new int[] {1, 1}, inputs, new long[12]));
    }

    @Test
    void rejectsInvalidTimeoutValuesAtNativeBoundary() {
        long[] invalidKind = new long[12];
        invalidKind[0] = 3;
        assertThrows(InvalidInputException.class, () -> create(
                new int[] {1}, new int[] {1}, new byte[][] {{1}}, invalidKind));

        long[] invalidNanoseconds = new long[12];
        invalidNanoseconds[2] = 1_000_000_000L;
        assertThrows(InvalidInputException.class, () -> create(
                new int[] {1}, new int[] {1}, new byte[][] {{1}}, invalidNanoseconds));
    }

    @Test
    void rejectsInvalidEndpointBeforeCallingTheRustConstructor() {
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.transportConfigCreate(
                99, "kmip.example.test", 5696, 1,
                new int[] {1}, new int[] {1}, new byte[][] {{1}},
                1, 1, new byte[] {1},
                new int[0], new int[0], new byte[0][],
                new int[0], new int[0], new byte[0][],
                null, null, new long[12], 16 * 1024 * 1024L));
    }

    @Test
    void rejectsNullEndpointAtNativeBoundary() {
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.transportConfigCreate(
                1, null, 5696, 1,
                new int[] {1}, new int[] {1}, new byte[][] {{1}},
                1, 1, new byte[] {1},
                new int[0], new int[0], new byte[0][],
                new int[0], new int[0], new byte[0][],
                null, null, new long[12], 16 * 1024 * 1024L));
    }

    @Test
    void rejectsNullPrivateKeyAtNativeBoundary() {
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.transportConfigCreate(
                1, "kmip.example.test", 5696, 1,
                new int[] {1}, new int[] {1}, new byte[][] {{1}},
                1, 1, null,
                new int[0], new int[0], new byte[0][],
                new int[0], new int[0], new byte[0][],
                null, null, new long[12], 16 * 1024 * 1024L));
    }

    private static long create(int[] sources, int[] encodings, byte[][] certificates, long[] timeouts) {
        return NativeExtensionRegistry.transportConfigCreate(
                1, "kmip.example.test", 5696, 1,
                sources, encodings, certificates,
                1, 1, new byte[] {1},
                new int[0], new int[0], new byte[0][],
                new int[0], new int[0], new byte[0][],
                null, null, timeouts, 16 * 1024 * 1024L);
    }
}
