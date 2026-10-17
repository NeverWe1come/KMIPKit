package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
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

    @Test
    void malformedResponsesReleaseOwnedTtlvAndRejectMissingTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long absentResponseTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCertifyResponse(new Object[] {
                    new long[] {0}, null, new long[] {absentResponseTtlv}
                }));
        ResponseTtlvTestSupport.assertReleased(absentResponseTtlv);

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCertifyResponse(new Object[] {
                    new long[] {1}, new Object[] {new long[] {1, 1, 0, 0}, null}, new long[] {0}
                }));

        long malformedIdentifierTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCertifyResponse(new Object[] {
                    new long[] {1}, new Object[] {new long[] {0, 1, 0, 0}, null},
                    new long[] {malformedIdentifierTtlv}
                }));
        ResponseTtlvTestSupport.assertReleased(malformedIdentifierTtlv);
    }
}
