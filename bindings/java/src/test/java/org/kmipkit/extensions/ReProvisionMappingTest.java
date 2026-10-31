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

/** Re-Provision binding tests derived from OASIS KMIP 2.1 §6.1.48, Tables 313–315. */
final class ReProvisionMappingTest {
    @Test
    void requestBuilderSupportsCertificateRequestCertificateAndServerCreatedForm() {
        try (ClientBatchItem serverCreated = ClientBatchItem.reprovision(null, null)) {
            assertNotNull(serverCreated);
        }
        try (var certificateRequest = TtlvStructure.create();
                ClientBatchItem csrForm = ClientBatchItem.reprovision(certificateRequest, null)) {
            assertNotNull(csrForm);
        }
        try (var certificate = TtlvStructure.create();
                ClientBatchItem certificateForm = ClientBatchItem.reprovision(null, certificate)) {
            assertNotNull(certificateForm);
        }
    }

    @Test
    void requestBuilderRejectsMutuallySuppliedCertificateObjects() {
        try (var certificateRequest = TtlvStructure.create();
                var certificate = TtlvStructure.create()) {
            assertThrows(
                    InvalidInputException.class,
                    () -> ClientBatchItem.reprovision(certificateRequest, certificate));
        }
    }

    @Test
    void successResponseKeepsAnOptionalIdentifierAndOwnedTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long withIdentifierTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        long withoutIdentifierTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        byte[] value = "reprovisioned-certificate".getBytes(StandardCharsets.UTF_8);
        Object[] identifier = {new long[] {1, 1, 0, 0}, value};

        try (var withIdentifier = ClientBatchItemResult.parseReProvisionResponse(new Object[] {
                    new long[] {1}, new long[] {1}, identifier, new long[] {withIdentifierTtlv}
                }).orElseThrow();
                var withoutIdentifier = ClientBatchItemResult.parseReProvisionResponse(new Object[] {
                    new long[] {1}, new long[] {0}, null, new long[] {withoutIdentifierTtlv}
                }).orElseThrow()) {
            assertTrue(withIdentifier.uniqueIdentifier().isPresent());
            assertEquals("reprovisioned-certificate", withIdentifier.uniqueIdentifier().orElseThrow().text());
            assertTrue(withoutIdentifier.uniqueIdentifier().isEmpty());
            assertNotNull(withoutIdentifier.responseTtlv());
            assertFalse(withIdentifier.toString().contains("reprovisioned-certificate"));
        }
    }

    @Test
    void absentTypedResponseRejectsUnexpectedIdentifierData() {
        assertTrue(ClientBatchItemResult.parseReProvisionResponse(
                new Object[] {new long[] {0}, new long[] {0}, null, new long[] {0}}).isEmpty());
        assertThrows(
                InvalidInputException.class,
                () -> ClientBatchItemResult.parseReProvisionResponse(
                        new Object[] {new long[] {0}, new long[] {1},
                            new Object(), new long[] {0}}));
    }

    @Test
    void invalidPresenceFlagsReleaseTheOwnedResponseTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseReProvisionResponse(new Object[] {
                    new long[] {2}, new long[] {0}, null, new long[] {responseTtlv}
                }));

        ResponseTtlvTestSupport.assertReleased(responseTtlv);
    }

    @Test
    void presentResponseRequiresAnOwnedTtlv() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseReProvisionResponse(new Object[] {
                    new long[] {1}, new long[] {0}, null, new long[] {0}
                }));
    }

    @Test
    void absentResponseReleasesAnUnexpectedOwnedTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseReProvisionResponse(new Object[] {
                    new long[] {0}, new long[] {0}, null, new long[] {responseTtlv}
                }));

        ResponseTtlvTestSupport.assertReleased(responseTtlv);
    }

    @Test
    void malformedPresentIdentifierReleasesTheOwnedResponseTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseReProvisionResponse(new Object[] {
                    new long[] {1}, new long[] {1}, new Object(), new long[] {responseTtlv}
                }));

        ResponseTtlvTestSupport.assertReleased(responseTtlv);
    }

    @Test
    void absentIdentifierRejectsUnexpectedDataAndReleasesTheOwnedResponseTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseReProvisionResponse(new Object[] {
                    new long[] {1}, new long[] {0}, new Object(), new long[] {responseTtlv}
                }));

        ResponseTtlvTestSupport.assertReleased(responseTtlv);
    }

    @Test
    void ttlvResponseBridgeRejectsAnInvalidNativeItemHandle() {
        NativeExtensionRegistry.ensureLoaded();
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseReprovisionTtlv(0));
    }
}
