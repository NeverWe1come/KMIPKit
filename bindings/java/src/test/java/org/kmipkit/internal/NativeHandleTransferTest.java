package org.kmipkit.internal;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

final class NativeHandleTransferTest {
    @Test
    void transferDisarmsCleanerAndMakesOwnerUnavailable() {
        NativeHandle owner = NativeHandle.owned(42, NativeExtensionRegistry.TTLV_VALUE);

        assertEquals(42, owner.transfer());
        assertThrows(InvalidInputException.class, owner::get);
        assertDoesNotThrow(owner::close);
    }

    @Test
    void secondTransferFailsInsteadOfReleasingTransferredHandle() {
        NativeHandle owner = NativeHandle.owned(42, NativeExtensionRegistry.TTLV_VALUE);
        owner.transfer();

        assertThrows(InvalidInputException.class, owner::transfer);
        assertDoesNotThrow(owner::close);
    }

    @Test
    void multiHandleTransferDoesNotDetachAnyHandleWhenOneIsAlreadyClosed() {
        NativeHandle first = NativeHandle.owned(42, NativeExtensionRegistry.TTLV_VALUE);
        NativeHandle closed = NativeHandle.owned(84, NativeExtensionRegistry.TTLV_ITEM);
        closed.transfer();

        assertThrows(InvalidInputException.class, () -> NativeHandle.transferAll(first, closed));
        assertEquals(42, first.get());
        assertEquals(42, first.transfer());
        assertDoesNotThrow(first::close);
        assertDoesNotThrow(closed::close);
    }

    @Test
    void closeWaitsUntilScopedNativeUseReturns() throws InterruptedException {
        NativeHandle owner = NativeHandle.owned(42, 0);
        CountDownLatch nativeUseEntered = new CountDownLatch(1);
        CountDownLatch resumeNativeUse = new CountDownLatch(1);
        CountDownLatch closeStarted = new CountDownLatch(1);
        CountDownLatch closeCompleted = new CountDownLatch(1);
        AtomicReference<Long> usedHandle = new AtomicReference<>();
        AtomicReference<Throwable> closeFailure = new AtomicReference<>();

        Thread nativeCall = new Thread(() -> usedHandle.set(owner.withValue(value -> {
            nativeUseEntered.countDown();
            try {
                if (!resumeNativeUse.await(2, TimeUnit.SECONDS)) {
                    throw new AssertionError("native use was not resumed");
                }
            } catch (InterruptedException error) {
                Thread.currentThread().interrupt();
                throw new AssertionError("native use was interrupted", error);
            }
            return value;
        })));
        Thread closer = new Thread(() -> {
            closeStarted.countDown();
            try {
                owner.close();
            } catch (Throwable error) { // Invalid handle kind is safe and prevents a fake pointer release.
                closeFailure.set(error);
            } finally {
                closeCompleted.countDown();
            }
        });

        nativeCall.start();
        assertTrue(nativeUseEntered.await(2, TimeUnit.SECONDS));
        closer.start();
        assertTrue(closeStarted.await(2, TimeUnit.SECONDS));
        assertFalse(closeCompleted.await(50, TimeUnit.MILLISECONDS));
        resumeNativeUse.countDown();
        nativeCall.join(2_000);
        closer.join(2_000);

        assertFalse(nativeCall.isAlive());
        assertFalse(closer.isAlive());
        assertEquals(42L, usedHandle.get());
        assertTrue(closeFailure.get() instanceof InvalidInputException);
        assertThrows(InvalidInputException.class, owner::get);
    }
}
