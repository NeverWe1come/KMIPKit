package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertNotNull;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatchItem;

final class GetConstraintsJavaFacadeTest {
    @Test
    void emptyRequestBuilderCreatesTypedBatchItem() {
        try (ClientBatchItem item = ClientBatchItem.getConstraints()) {
            assertNotNull(item);
        }
    }
}
