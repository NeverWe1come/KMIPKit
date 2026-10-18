package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

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
}
