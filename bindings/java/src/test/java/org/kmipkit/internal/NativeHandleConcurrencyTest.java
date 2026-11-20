package org.kmipkit.internal;

import static org.junit.jupiter.api.Assertions.*;
import java.util.concurrent.*;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

class NativeHandleConcurrencyTest {
    @Test
    void independentHandlesRemainUsableDuringNativeScope() throws Exception {
        ExecutorService worker = Executors.newSingleThreadExecutor();
        try (TtlvStructure first = TtlvStructure.create(); TtlvStructure second = TtlvStructure.create()) {
            first.nativeHandleOwner().withValue(raw -> {
                Future<Long> read = worker.submit(() -> {
                    try (TtlvStructureView view = second.view()) { return view.itemCount(); }
                });
                try { assertEquals(0L, read.get(2, TimeUnit.SECONDS)); }
                catch (Exception error) { throw new AssertionError(error); }
                return null;
            });
        } finally { worker.shutdownNow(); }
    }

    @Test
    void transferFailsWhileAnotherThreadBorrowsTheHandle() throws Exception {
        ExecutorService worker = Executors.newSingleThreadExecutor();
        CountDownLatch entered = new CountDownLatch(1);
        CountDownLatch resume = new CountDownLatch(1);
        try (TtlvStructure structure = TtlvStructure.create()) {
            Future<?> call = worker.submit(() -> structure.nativeHandleOwner().withValue(raw -> {
                entered.countDown();
                try { assertTrue(resume.await(2, TimeUnit.SECONDS)); }
                catch (InterruptedException error) { throw new AssertionError(error); }
                return null;
            }));
            try {
                assertTrue(entered.await(2, TimeUnit.SECONDS));
                assertThrows(InvalidInputException.class, () -> structure.nativeHandleOwner().transfer());
            } finally { resume.countDown(); }
            call.get(2, TimeUnit.SECONDS);
            try (TtlvStructureView view = structure.view()) { assertEquals(0, view.itemCount()); }
        } finally { resume.countDown(); worker.shutdownNow(); }
    }

    @Test
    void scopedTransferCanFinishAConsumingNativeCallWhileAnotherScopeRetainsTheOwner() throws Exception {
        ExecutorService worker = Executors.newSingleThreadExecutor();
        CountDownLatch borrowed = new CountDownLatch(1);
        CountDownLatch resume = new CountDownLatch(1);
        try (TtlvStructure structure = TtlvStructure.create()) {
            NativeHandle owner = structure.nativeHandleOwner();
            Future<?> other = worker.submit(() -> owner.withValue(raw -> {
                borrowed.countDown();
                try { assertTrue(resume.await(2, TimeUnit.SECONDS)); }
                catch (InterruptedException error) { throw new AssertionError(error); }
                return null;
            }));
            try {
                assertTrue(borrowed.await(2, TimeUnit.SECONDS));
                long value = owner.withValue(raw -> {
                    long converted = NativeExtensionRegistry.ttlvValueStructure(raw, new long[] {16777216, 64, 100000});
                    assertEquals(raw, owner.transfer());
                    return converted;
                });
                NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_VALUE, value);
                assertThrows(InvalidInputException.class, owner::get);
            } finally { resume.countDown(); }
            other.get(2, TimeUnit.SECONDS);
        } finally { resume.countDown(); worker.shutdownNow(); }
    }

    @Test
    void reentrantCloseDefersReleaseUntilTheNativeScopeReturns() {
        TtlvStructure structure = TtlvStructure.create();
        NativeHandle owner = structure.nativeHandleOwner();
        owner.withValue(raw -> {
            structure.close();
            long view = NativeExtensionRegistry.ttlvStructureView(raw);
            NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE_VIEW, view);
            return null;
        });
        assertThrows(InvalidInputException.class, owner::get);
    }
}
