package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.kmipkit.NativeExtensionRegistry;
import org.junit.jupiter.api.Test;

final class CertifyResponseMappingTest {
    @Test
    void successfulResponseMapsGeneratedTextIdentifierAndKeepsItRedacted() {
        NativeExtensionRegistry.ensureLoaded();
        byte[] identifier = "issued-certificate".getBytes(StandardCharsets.UTF_8);
        Object[] record = {new long[] {1, 1, 0, 0}, identifier};
        long responseTtlvHandle = NativeExtensionRegistry.ttlvStructureCreate();

        try (ClientBatchItemResult.CertifyResponse response = ClientBatchItemResult
                .parseCertifyResponse(new Object[] {
                    new long[] {1}, record, new long[] {responseTtlvHandle}})
                .orElseThrow()) {

            assertEquals(UniqueIdentifier.Kind.TEXT_STRING, response.uniqueIdentifier().kind());
            assertEquals("issued-certificate", response.uniqueIdentifier().text());
            assertTrue(response.responseTtlv().handle() > 0);
            assertFalse(response.toString().contains("issued-certificate"));
        }
    }

    @Test
    void absentResponseDoesNotCreateAnIdentifier() {
        assertTrue(ClientBatchItemResult
                .parseCertifyResponse(new Object[] {new long[] {0}, null, new long[] {0}})
                .isEmpty());
    }
}
