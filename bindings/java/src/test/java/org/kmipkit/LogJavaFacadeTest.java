package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatchItem;

final class LogJavaFacadeTest {
    @Test
    void builderPreservesCallerMessageAndReleasesBatchItem() {
        byte[] message = "log-message-private-sentinel".getBytes(java.nio.charset.StandardCharsets.UTF_8);
        byte[] original = message.clone();

        try (ClientBatchItem item = ClientBatchItem.log(message)) {
            assertNotNull(item);
        }

        assertArrayEquals(original, message);
    }

    @Test
    void builderRejectsNullMessage() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.log(null));
    }

    @Test
    void nativeResponseBridgeRejectsNullResultHandle() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseLog(0));
    }
}
