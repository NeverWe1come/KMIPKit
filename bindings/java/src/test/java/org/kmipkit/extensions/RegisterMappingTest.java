package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvStructure;
import org.junit.jupiter.api.Test;

/** Register binding tests derived from OASIS KMIP 2.1 §6.1.43, Tables 291–294. */
final class RegisterMappingTest {
    @Test
    void successResponseKeepsRequiredIdentifierAndOwnedOrderedTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        byte[] identifier = "registered-object".getBytes(StandardCharsets.UTF_8);
        Object[] identifierRecord = {new long[] {1, 1, 0, 0}, identifier};

        try (var response = ClientBatchItemResult.parseRegisterResponse(new Object[] {
                    new long[] {1}, identifierRecord, new long[] {responseTtlv}
                }).orElseThrow()) {
            assertEquals("registered-object", response.uniqueIdentifier().text());
            assertNotNull(response.responseTtlv());
            assertFalse(response.toString().contains("registered-object"));
        }
    }

    @Test
    void invalidPresenceFlagReleasesTheOwnedResponseTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();

        assertThrows(
                InvalidInputException.class,
                () -> ClientBatchItemResult.parseRegisterResponse(new Object[] {
                    new long[] {2}, null, new long[] {responseTtlv}
                }));

        ResponseTtlvTestSupport.assertReleased(responseTtlv);
    }

    @Test
    void absentTypedResponseRejectsUnexpectedIdentifierAndTtlv() {
        assertTrue(ClientBatchItemResult.parseRegisterResponse(
                new Object[] {new long[] {0}, null, new long[] {0}}).isEmpty());
        assertThrows(
                InvalidInputException.class,
                () -> ClientBatchItemResult.parseRegisterResponse(
                        new Object[] {new long[] {0}, new Object(), new long[] {0}}));
    }

    @Test
    void successfulResponseRequiresIdentifierAndReleasesTtlvOnMalformedData() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();

        assertThrows(
                InvalidInputException.class,
                () -> ClientBatchItemResult.parseRegisterResponse(new Object[] {
                    new long[] {1}, null, new long[] {responseTtlv}
                }));

        ResponseTtlvTestSupport.assertReleased(responseTtlv);
    }

    @Test
    void responseBridgeRejectsInvalidNativeItemHandle() {
        NativeExtensionRegistry.ensureLoaded();
        assertThrows(
                InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseRegister(0));
    }
}
