package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.ttlv.TtlvStructure;

final class SetDefaultsJavaFacadeTest {
    @Test
    void typedRequestBuilderAcceptsAndCopiesCallerDefaults() {
        try (ClientBatchItem omitted = ClientBatchItem.setDefaults();
                TtlvStructure defaultsInformation = TtlvStructure.create();
                ClientBatchItem item = ClientBatchItem.setDefaults(defaultsInformation)) {
            assertNotNull(omitted);
            assertNotNull(item);
        }
    }

    @Test
    void nativeRequestBridgeRejectsAnInvalidDefaultsHandle() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemSetDefaults(1));
    }

    @Test
    void nativeResponseBridgeRejectsAnInvalidResponseHandle() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseSetDefaults(0));
    }
}
