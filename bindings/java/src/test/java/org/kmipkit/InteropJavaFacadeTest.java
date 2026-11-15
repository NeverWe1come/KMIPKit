package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatchItem;

final class InteropJavaFacadeTest {
    @Test
    void builderPreservesOpenFunctionAndDoesNotMutateCallerIdentifier() {
        byte[] identifier = "interop-private-sentinel".getBytes(java.nio.charset.StandardCharsets.UTF_8);
        byte[] original = identifier.clone();

        try (ClientBatchItem item = ClientBatchItem.interop(0xf1234567L, identifier)) {
            assertNotNull(item);
        }

        assertArrayEquals(original, identifier);
    }

    @Test
    void builderRejectsOutOfRangeFunctionNullIdentifierAndInvalidUtf8() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.interop(-1, new byte[0]));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.interop(1L << 32, new byte[0]));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.interop(1, null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.interop(1, new byte[] {(byte) 0xff}));
    }

    @Test
    void nativeResponseBridgeRejectsNullResultHandle() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseInterop(0));
    }
}
