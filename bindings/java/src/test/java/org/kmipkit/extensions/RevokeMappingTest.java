package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.Optional;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvStructure;
import org.junit.jupiter.api.Test;

/** Revoke binding tests derived from OASIS KMIP 2.1 §§4.47, 6.1.44, and 11.48. */
final class RevokeMappingTest {
    @Test
    void requestRetainsUnsignedUnknownReasonAndNestedAuditMessage() {
        NativeExtensionRegistry.ensureLoaded();
        try (var reason = RevocationReason.create(0xffff_ffffL, "operator audit")) {
            assertEquals(0xffff_ffffL, reason.rawCode());
            assertEquals(Optional.of("operator audit"), reason.message());
            try (var request = RevokeRequest.create(
                    UniqueIdentifier.textString("object-to-revoke"), reason, null);
                    var item = ClientBatchItem.revoke(request)) {
                assertNotNull(item);
            }
        }
    }

    @Test
    void compromiseDateIsRejectedForReasonsOtherThanKeyOrCaCompromise() {
        NativeExtensionRegistry.ensureLoaded();
        try (var reason = RevocationReason.create(4, null)) {
            var request = RevokeRequest.create(null, reason, 1L);
            assertThrows(InvalidInputException.class, () -> ClientBatchItem.revoke(request));
        }
    }

    @Test
    void successfulResponseExposesIdentifierAndOwnedCompleteTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        byte[] identifier = "revoked-object".getBytes(StandardCharsets.UTF_8);
        Object[] identifierRecord = {new long[] {1, 1, 0, 0}, identifier};

        try (var response = ClientBatchItemResult.parseRevokeResponse(new Object[] {
                    new long[] {1}, identifierRecord, new long[] {responseTtlv}
                }).orElseThrow()) {
            assertEquals("revoked-object", response.uniqueIdentifier().text());
            assertNotNull(response.responseTtlv());
            assertFalse(response.toString().contains("revoked-object"));
        }
    }

    @Test
    void failureAndPendingHaveNoTypedSuccessResponse() {
        assertTrue(ClientBatchItemResult.parseRevokeResponse(
                new Object[] {new long[] {0}, null, new long[] {0}}).isEmpty());
        assertThrows(
                InvalidInputException.class,
                () -> ClientBatchItemResult.parseRevokeResponse(
                        new Object[] {new long[] {0}, new Object(), new long[] {0}}));
    }
}
