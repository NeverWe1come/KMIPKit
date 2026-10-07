package org.kmipkit.ttlv;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;

import org.junit.jupiter.api.Test;

final class TtlvValueSecretCopyTest {
    @Test
    void nativeTemporaryCopyIsClearedAfterSuccessAndFailure() throws Exception {
        Method helper = Arrays.stream(TtlvValue.class.getDeclaredMethods())
                .filter(method -> method.getName().equals("withZeroizedCopy"))
                .findFirst()
                .orElse(null);
        assertNotNull(helper, "byte factories need a scoped, zeroizing JNI copy helper");
        helper.setAccessible(true);

        byte[] original = { 0x41, 0x42, 0x43 };
        AtomicReference<byte[]> nativeInput = new AtomicReference<>();
        Object success = new Object();
        Function<byte[], Object> successfulCall = copy -> {
            nativeInput.set(copy);
            assertArrayEquals(original, copy);
            return success;
        };

        assertSame(success, helper.invoke(null, original, successfulCall));
        assertArrayEquals(new byte[original.length], nativeInput.get());
        assertArrayEquals(new byte[] { 0x41, 0x42, 0x43 }, original);

        Function<byte[], Object> failingCall = copy -> {
            nativeInput.set(copy);
            assertArrayEquals(original, copy);
            throw new IllegalStateException("simulated JNI failure");
        };
        InvocationTargetException failure = assertThrows(
                InvocationTargetException.class,
                () -> helper.invoke(null, original, failingCall));
        assertInstanceOf(IllegalStateException.class, failure.getCause());
        assertEquals("simulated JNI failure", failure.getCause().getMessage());
        assertArrayEquals(new byte[original.length], nativeInput.get());
        assertArrayEquals(new byte[] { 0x41, 0x42, 0x43 }, original);
    }
}