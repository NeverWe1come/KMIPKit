package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.junit.jupiter.api.Test;

final class GetUsageAllocationResponseMappingTest {
    @Test
    void successfulResponseMapsIdentifierAndKeepsItRedacted() {
        byte[] identifier = "allocation-object".getBytes(StandardCharsets.UTF_8);
        Object[] record = {new long[] {1, 1, 0, 0}, identifier};

        ClientBatchItemResult.GetUsageAllocationResponse response = ClientBatchItemResult
                .parseGetUsageAllocationResponse(new Object[] {new long[] {1}, record})
                .orElseThrow();

        assertEquals(UniqueIdentifier.Kind.TEXT_STRING, response.uniqueIdentifier().kind());
        assertEquals("allocation-object", response.uniqueIdentifier().text());
        assertFalse(response.toString().contains("allocation-object"));
    }

    @Test
    void absentResponseDoesNotCreateAnIdentifier() {
        assertTrue(ClientBatchItemResult
                .parseGetUsageAllocationResponse(new Object[] {new long[] {0}, null})
                .isEmpty());
    }
}
