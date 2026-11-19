package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.junit.jupiter.api.Test;

/** Binding checks derived from KMIP 2.1 §§6.1.1, 6.1.4, 6.1.15, and 6.1.42. */
final class LifecycleOperationMappingTest {
    @Test
    void allLifecycleFactoriesCreateNativeBatchItemsWithOptionalIdentifiers() {
        NativeExtensionRegistry.ensureLoaded();
        UniqueIdentifier identifier = UniqueIdentifier.enumeration(0x8000_0001L);

        try (ClientBatchItem activate = ClientBatchItem.activate(identifier);
                ClientBatchItem archive = ClientBatchItem.archive(null);
                ClientBatchItem destroy = ClientBatchItem.destroy(identifier);
                ClientBatchItem recover = ClientBatchItem.recover(null)) {
            assertNotNull(activate);
            assertNotNull(archive);
            assertNotNull(destroy);
            assertNotNull(recover);
        }
    }

    @Test
    void successfulResponseRequiresIdentifierAndAbsentResultStaysEmpty() {
        byte[] identifier = "lifecycle-response".getBytes(java.nio.charset.StandardCharsets.UTF_8);

        assertEquals(
                "lifecycle-response",
                ClientBatchItemResult.parseLifecycleIdentifierResponse(
                        new Object[] {new long[] {1}, new Object[] {
                            new long[] {1, 1, 0, 0}, identifier
                        }}, "Activate").orElseThrow().text());
        assertTrue(ClientBatchItemResult.parseLifecycleIdentifierResponse(
                new Object[] {new long[] {0}, null}, "Archive").isEmpty());
        assertThrows(
                InvalidInputException.class,
                () -> ClientBatchItemResult.parseLifecycleIdentifierResponse(
                        new Object[] {new long[] {1}, null}, "Recover"));
        assertThrows(
                InvalidInputException.class,
                () -> ClientBatchItemResult.parseLifecycleIdentifierResponse(
                        new Object[] {new long[] {2}, null}, "Destroy"));
    }
}
