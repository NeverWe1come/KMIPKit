package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.ttlv.TtlvStructure;

final class SetConstraintsJavaFacadeTest {
    @Test
    void typedRequestBuilderAcceptsAndCopiesCallerConstraints() {
        try (TtlvStructure constraints = TtlvStructure.create();
                ClientBatchItem item = ClientBatchItem.setConstraints(constraints)) {
            assertNotNull(item);
        }
    }

    @Test
    void nativeRequestBridgeRejectsAnInvalidConstraintsHandle() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemSetConstraints(0));
    }

    @Test
    void nativeResponseBridgeRejectsAnInvalidResponseHandle() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseSetConstraints(0));
    }
}
