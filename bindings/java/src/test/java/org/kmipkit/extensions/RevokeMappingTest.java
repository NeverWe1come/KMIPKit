package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.Optional;
import java.util.concurrent.TimeUnit;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.TtlvStructure;
import org.junit.jupiter.api.Test;

/** Revoke binding tests derived from OASIS KMIP 2.1 §§4.47, 6.1.44, and 11.48. */
final class RevokeMappingTest {
    @Test
    void utf8LengthPreflightCountsAllScalarWidthsAndRejectsMalformedText() {
        assertEquals(1, UniqueIdentifier.checkedUtf8Length("A", 4, "test"));
        assertEquals(2, UniqueIdentifier.checkedUtf8Length("é", 4, "test"));
        assertEquals(3, UniqueIdentifier.checkedUtf8Length("三", 4, "test"));
        assertEquals(4, UniqueIdentifier.checkedUtf8Length("🔐", 4, "test"));
        assertThrows(ResourceLimitException.class,
                () -> UniqueIdentifier.checkedUtf8Length("é", 1, "test"));
        assertThrows(InvalidInputException.class,
                () -> UniqueIdentifier.checkedUtf8Length(String.valueOf((char) 0xd800), 4, "test"));
    }

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
    void rejectsOversizedIdentifierAndMessageBeforeAllocatingEncodedCopies() throws Exception {
        String javaExecutable = Path.of(System.getProperty("java.home"), "bin", "java").toString();
        String classpath = System.getProperty("surefire.test.class.path", System.getProperty("java.class.path"));
        for (String inputKind : new String[] {"identifier", "message"}) {
            Process process = new ProcessBuilder(javaExecutable, "-Xmx48m", "-cp", classpath,
                    RevokeTextLimitProbe.class.getName(), inputKind, "24000000")
                    .redirectErrorStream(true)
                    .start();
            if (!process.waitFor(30, TimeUnit.SECONDS)) {
                process.destroyForcibly();
                throw new AssertionError("bounded-heap Revoke text-limit probe did not finish");
            }
            String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8).trim();
            assertEquals(0, process.exitValue(), inputKind + ": " + output);
            assertEquals("resource-limit", output, inputKind);
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

    @Test
    void responseBridgeRejectsInvalidNativeItemHandle() {
        NativeExtensionRegistry.ensureLoaded();
        assertThrows(
                InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseRevoke(0));
    }
}
