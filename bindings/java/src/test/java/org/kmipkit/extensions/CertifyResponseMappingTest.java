package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.junit.jupiter.api.Test;

final class CertifyResponseMappingTest {
    @Test
    void successfulResponseMapsGeneratedTextIdentifierAndKeepsItRedacted() {
        byte[] identifier = "issued-certificate".getBytes(StandardCharsets.UTF_8);
        Object[] record = {new long[] {1, 1, 0, 0}, identifier};

        ClientBatchItemResult.CertifyResponse response = ClientBatchItemResult
                .parseCertifyResponse(new Object[] {new long[] {1}, record})
                .orElseThrow();

        assertEquals(UniqueIdentifier.Kind.TEXT_STRING, response.uniqueIdentifier().kind());
        assertEquals("issued-certificate", response.uniqueIdentifier().text());
        assertFalse(response.toString().contains("issued-certificate"));
    }

    @Test
    void absentResponseDoesNotCreateAnIdentifier() {
        assertTrue(ClientBatchItemResult
                .parseCertifyResponse(new Object[] {new long[] {0}, null})
                .isEmpty());
    }
}
