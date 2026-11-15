package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertTrue;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

final class ResponseTtlvTestSupport {
    private ResponseTtlvTestSupport() {}

    static void assertReleased(long structureHandle) {
        long viewHandle = 0;
        boolean released = false;
        try {
            viewHandle = NativeExtensionRegistry.ttlvStructureView(structureHandle);
        } catch (InvalidInputException expected) {
            released = true;
        } finally {
            if (viewHandle != 0) {
                NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE_VIEW, viewHandle);
                NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE, structureHandle);
            }
        }
        assertTrue(released, "the rejected response must release its owned TTLV structure");
    }
}
