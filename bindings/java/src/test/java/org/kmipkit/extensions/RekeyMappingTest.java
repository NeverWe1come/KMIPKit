package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.junit.jupiter.api.Test;

/** Re-key mapping tests for OASIS KMIP 2.1 §6.1.46, Tables 303–307. */
final class RekeyMappingTest {
    @Test
    void requestBuilderAllowsOmittedFieldsAndRawOffset() {
        try (ClientBatchItem empty = ClientBatchItem.rekey(null, null, null, null);
                ClientBatchItem full = ClientBatchItem.rekey(
                        UniqueIdentifier.textString("existing-key"), 0xffff_ffffL, null, null)) {
            assertNotNull(empty);
            assertNotNull(full);
        }
    }

    @Test
    void requestBuilderRejectsOffsetOutsideUnsignedInterval() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.rekey(null, 0x1_0000_0000L, null, null));
    }

    @Test
    void successfulResponseMapsReplacementIdentifierAndRedactsIt() {
        NativeExtensionRegistry.ensureLoaded();
        byte[] identifier = "replacement-key".getBytes(StandardCharsets.UTF_8);
        Object[] identifierRecord = {new long[] {1, 1, 0, 0}, identifier};
        long responseTtlvHandle = NativeExtensionRegistry.ttlvStructureCreate();

        try (ClientBatchItemResult.RekeyResponse response = ClientBatchItemResult
                .parseRekeyResponse(new Object[] {
                    new long[] {1}, identifierRecord, new long[] {responseTtlvHandle}
                })
                .orElseThrow()) {
            assertEquals(UniqueIdentifier.Kind.TEXT_STRING, response.uniqueIdentifier().kind());
            assertEquals("replacement-key", response.uniqueIdentifier().text());
            assertNotNull(response.responseTtlv());
            assertFalse(response.toString().contains("replacement-key"));
        }
    }

    @Test
    void absentResponseIsEmptyAndRejectsUnexpectedOwnedTtlv() {
        assertTrue(ClientBatchItemResult
                .parseRekeyResponse(new Object[] {new long[] {0}, null, new long[] {0}})
                .isEmpty());

        NativeExtensionRegistry.ensureLoaded();
        long unexpectedTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseRekeyResponse(new Object[] {
                    new long[] {0}, null, new long[] {unexpectedTtlv}}
                ));
        ResponseTtlvTestSupport.assertReleased(unexpectedTtlv);
    }

    @Test
    void presentResponseRequiresAnOwnedTtlv() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseRekeyResponse(
                        new Object[] {new long[] {1}, null, new long[] {0}}));
    }

    @Test
    void malformedPresentIdentifierReleasesTheOwnedResponseTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseRekeyResponse(new Object[] {
                    new long[] {1}, new Object(), new long[] {responseTtlv}
                }));

        ResponseTtlvTestSupport.assertReleased(responseTtlv);
    }
}
