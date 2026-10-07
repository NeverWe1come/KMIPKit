package org.kmipkit.ttlv;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;

import org.junit.jupiter.api.Test;

final class TtlvValueSecretCopyTest {
    @Test
    void nativeTemporaryCopyIsClearedAfterSuccessAndFailure() throws Exception {
        byte[] original = { 0x41, 0x42, 0x43 };
        AtomicReference<byte[]> nativeInput = new AtomicReference<>();
        Object success = new Object();
        Function<byte[], Object> successfulCall = copy -> {
            nativeInput.set(copy);
            assertArrayEquals(original, copy);
            return success;
        };

        assertSame(success, TtlvValue.withZeroizedCopy(original, successfulCall));
        assertArrayEquals(new byte[original.length], nativeInput.get());
        assertArrayEquals(new byte[] { 0x41, 0x42, 0x43 }, original);

        Function<byte[], Object> failingCall = copy -> {
            nativeInput.set(copy);
            assertArrayEquals(original, copy);
            throw new IllegalStateException("simulated JNI failure");
        };
        IllegalStateException failure = assertThrows(
                IllegalStateException.class,
                () -> TtlvValue.withZeroizedCopy(original, failingCall));
        assertEquals("simulated JNI failure", failure.getMessage());
        assertArrayEquals(new byte[original.length], nativeInput.get());
        assertArrayEquals(new byte[] { 0x41, 0x42, 0x43 }, original);
    }
}