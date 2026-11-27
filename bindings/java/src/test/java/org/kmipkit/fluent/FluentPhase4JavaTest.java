package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;

final class FluentPhase4JavaTest {
    @Test
    void fluentBatchBuilderAssemblesOrderedBatchWithoutClient() {
        try (FluentBatchBuilder batch = new FluentBatchBuilder(null).ordered()) {
            batch.add(new PingBuilder(null).build());
            assertEquals(1L, batch.build().itemCount());
        }
    }

    @Test
    void pendingOperationPreservesCorrelationValue() {
        PendingOperation pending = new PendingOperation(new byte[] {1, 2, 3, 4});
        assertArrayEquals(new byte[] {1, 2, 3, 4}, pending.correlationValue());
        assertThrows(InvalidInputException.class, () -> new PendingOperation(new byte[0]));
    }
}
