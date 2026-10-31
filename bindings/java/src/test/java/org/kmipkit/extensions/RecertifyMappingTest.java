package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.junit.jupiter.api.Test;

/** Re-certify mappings for OASIS KMIP 2.1 §6.1.45, Tables 300–302. */
final class RecertifyMappingTest {
    @Test
    void requestBuilderPreservesOptionalFieldsAndDoesNotChangeCallerBytes() {
        byte[] requestValue = {0x30, 0x00, 0x41, 0x42};
        byte[] original = requestValue.clone();

        try (ClientBatchItem item = ClientBatchItem.recertify(
                UniqueIdentifier.textString("certificate-7"),
                UniqueIdentifier.textString("request-7"),
                0xfedc_ba98L,
                requestValue,
                0xffff_ffffL,
                null,
                null)) {
            assertNotNull(item);
        }

        assertArrayEquals(original, requestValue);
    }

    @Test
    void requestBuilderAllowsAllOptionalFieldsToBeOmitted() {
        try (ClientBatchItem item = ClientBatchItem.recertify(
                null, null, null, null, null, null, null)) {
            assertNotNull(item);
        }
    }

    @Test
    void successfulResponseMapsReplacementIdentifierAndRedactsIt() {
        NativeExtensionRegistry.ensureLoaded();
        byte[] identifier = "replacement-certificate".getBytes(StandardCharsets.UTF_8);
        Object[] identifierRecord = {new long[] {1, 1, 0, 0}, identifier};
        long responseTtlvHandle = NativeExtensionRegistry.ttlvStructureCreate();

        try (ClientBatchItemResult.RecertifyResponse response = ClientBatchItemResult
                .parseRecertifyResponse(new Object[] {
                    new long[] {1}, identifierRecord, new long[] {responseTtlvHandle}
                })
                .orElseThrow()) {
            assertEquals(UniqueIdentifier.Kind.TEXT_STRING, response.uniqueIdentifier().kind());
            assertEquals("replacement-certificate", response.uniqueIdentifier().text());
            assertNotNull(response.responseTtlv());
            assertFalse(response.toString().contains("replacement-certificate"));
        }
    }

    @Test
    void absentResponseIsEmptyAndRejectsUnexpectedOwnedTtlv() {
        assertTrue(ClientBatchItemResult
                .parseRecertifyResponse(new Object[] {new long[] {0}, null, new long[] {0}})
                .isEmpty());

        NativeExtensionRegistry.ensureLoaded();
        long unexpectedTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseRecertifyResponse(new Object[] {
                    new long[] {0}, null, new long[] {unexpectedTtlv}}
                ));
        ResponseTtlvTestSupport.assertReleased(unexpectedTtlv);
    }

    @Test
    void malformedSuccessRejectsMissingTtlvAndReleasesTtlvOnInvalidIdentifier() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseRecertifyResponse(new Object[] {
                    new long[] {1}, new Object[] {new long[] {1, 1, 0, 0},
                        "replacement-certificate".getBytes(StandardCharsets.UTF_8)}, new long[] {0}}
                ));

        NativeExtensionRegistry.ensureLoaded();
        long malformedIdentifierTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseRecertifyResponse(new Object[] {
                    new long[] {1}, new Object[] {new long[] {0, 1, 0, 0}, null},
                    new long[] {malformedIdentifierTtlv}}
                ));
        ResponseTtlvTestSupport.assertReleased(malformedIdentifierTtlv);
    }
}
