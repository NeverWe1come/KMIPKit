package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvStructure;

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

    @Test
    void createFactoriesRejectMissingRequiredAttributesAndOutOfRangeUnsignedValues() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.create(7, null, null));

        try (TtlvStructure attributes = TtlvStructure.create()) {
            assertThrows(InvalidInputException.class,
                    () -> ClientBatchItem.create(0x1_0000_0000L, attributes, null));
            assertThrows(InvalidInputException.class,
                    () -> ClientBatchItem.createSplitKey(7, null, 2, 2,
                            0x1_0000_0000L, null, attributes, null));
            assertThrows(InvalidInputException.class,
                    () -> ClientBatchItem.createSplitKey(7, null, 2, 2,
                            1, null, null, null));
        }
    }

    @Test
    void splitKeyFactoryMarshalsTextAndEnumerationIdentifiers() {
        try (TtlvStructure attributes = TtlvStructure.create();
                ClientBatchItem textIdentifier = ClientBatchItem.createSplitKey(
                        7, UniqueIdentifier.textString("request-key-id"), 2, 2, 1,
                        null, attributes, null);
                ClientBatchItem enumerationIdentifier = ClientBatchItem.createSplitKey(
                        7, UniqueIdentifier.enumeration(0xffff_ffffL), 2, 2, 1,
                        null, attributes, null)) {
            assertEquals(0, textIdentifier.extensionCount());
            assertEquals(0, enumerationIdentifier.extensionCount());
        }
    }
}
