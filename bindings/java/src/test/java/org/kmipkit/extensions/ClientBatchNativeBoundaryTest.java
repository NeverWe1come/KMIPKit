package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Exercises fixed-width validation at the Java-to-JNI boundary. */
final class ClientBatchNativeBoundaryTest {
    @Test
    void asynchronousIndicatorRejectsNullHandlesAndValuesOutsideUint32() {
        try (ClientBatch batch = ClientBatch.create()) {
            long nativeBatch = batch.nativeHandleOwner().get();

            assertThrows(
                    InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetAsynchronousIndicator(0, 1));
            assertThrows(
                    InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetAsynchronousIndicator(nativeBatch, -1));
            assertThrows(
                    InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetAsynchronousIndicator(
                            nativeBatch, 0x1_0000_0000L));
        }
    }
}
