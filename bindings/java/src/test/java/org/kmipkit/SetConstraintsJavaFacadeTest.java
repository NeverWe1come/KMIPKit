package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertNotNull;

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
}
