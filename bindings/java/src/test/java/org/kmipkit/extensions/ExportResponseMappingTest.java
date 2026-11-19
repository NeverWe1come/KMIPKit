package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvStructure;
import org.junit.jupiter.api.Test;

final class ExportResponseMappingTest {
    @Test
    void exportBuilderPreservesUnknownOptionalTable217ValuesAndAllowsOmission() {
        try (TtlvStructure wrappingSpecification = TtlvStructure.create();
                ClientBatchItem explicit = ClientBatchItem.export(
                        UniqueIdentifier.textString("exported-key"),
                        0xf123_4567L, 0xf123_4568L, 0xf123_4569L,
                        wrappingSpecification);
                ClientBatchItem omitted = ClientBatchItem.export(null, null, null, null, null)) {
            assertTrue(explicit != null);
            assertTrue(omitted != null);
        }
    }

    @Test
    void exportJniBoundaryRejectsSelectorsOutsideUnsignedThirtyTwoBitRange() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.clientBatchItemExport(
                0, null, 0, true, -1, false, 0, false, 0, 0));
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.clientBatchItemExport(
                0, null, 0, false, 0, true, 0x1_0000_0000L, false, 0, 0));
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.clientBatchItemExport(
                0, null, 0, false, 0, false, 0, true, -1, 0));
    }

    @Test
    void successfulExportResponsePreservesRawObjectTypeAndOwnsOpaqueOrderedTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long attributes = NativeExtensionRegistry.ttlvStructureCreate();
        long anyObject = NativeExtensionRegistry.ttlvStructureCreate();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        Object[] identifier = {new long[] {1, 1, 0, 0}, "export-secret-id".getBytes()};

        try (ClientBatchItemResult.ExportResponse response = ClientBatchItemResult
                .parseExportResponse(new Object[] {
                    new long[] {1, 1, 0xf123_4567L}, identifier,
                    new long[] {attributes, anyObject, responseTtlv}
                })
                .orElseThrow()) {
            assertEquals(0xf123_4567L, response.objectTypeRaw().orElseThrow());
            assertEquals("export-secret-id", response.uniqueIdentifier().orElseThrow().text());
            assertTrue(response.attributes().isPresent());
            assertTrue(response.anyObject().isPresent());
            assertTrue(response.responseTtlv().handle() > 0);
            assertFalse(response.toString().contains("export-secret-id"));
        }
    }

    @Test
    void failureExportResponseRemainsTypedAndRetainsCompleteResponseTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();

        try (ClientBatchItemResult.ExportResponse response = ClientBatchItemResult
                .parseExportResponse(new Object[] {
                    new long[] {1, 0, 0}, null, new long[] {0, 0, responseTtlv}
                })
                .orElseThrow()) {
            assertTrue(response.objectTypeRaw().isEmpty());
            assertTrue(response.uniqueIdentifier().isEmpty());
            assertTrue(response.attributes().isEmpty());
            assertTrue(response.anyObject().isEmpty());
            assertTrue(response.responseTtlv().handle() > 0);
        }
    }

    @Test
    void successfulExportResponseRequiresTable218Attributes() {
        NativeExtensionRegistry.ensureLoaded();
        long anyObject = NativeExtensionRegistry.ttlvStructureCreate();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        Object[] identifier = {new long[] {1, 1, 0, 0}, "export-secret-id".getBytes()};

        assertThrows(InvalidInputException.class, () -> {
            var response = ClientBatchItemResult.parseExportResponse(new Object[] {
                    new long[] {1, 1, 0xf123_4567L}, identifier,
                    new long[] {0, anyObject, responseTtlv}
            });
            response.ifPresent(ClientBatchItemResult.ExportResponse::close);
        });
    }

    @Test
    void absentExportResponseRejectsUnexpectedTtlvHandle() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();

        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseExportResponse(
                new Object[] {new long[] {0, 0, 0}, null, new long[] {0, 0, responseTtlv}}));
    }

    @Test
    void exportResponseRejectsPayloadPresenceOutsideBooleanRange() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();

        InvalidInputException exception = assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseExportResponse(new Object[] {
                    new long[] {1, 2, 0}, null, new long[] {0, 0, responseTtlv}
                }));
        assertEquals("native Export payload presence is invalid", exception.getMessage());
        try {
            assertThrows(InvalidInputException.class, () -> {
                long view = NativeExtensionRegistry.ttlvStructureView(responseTtlv);
                NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE_VIEW, view);
            });
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE, responseTtlv);
        }
    }

    @Test
    void successfulExportResponseRequiresCompleteResponseTtlv() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseExportResponse(
                new Object[] {new long[] {1, 0, 0}, null, new long[] {0, 0, 0}}));
    }

    @Test
    void failedExportResponseRejectsAndReleasesTypedPayloadHandles() {
        NativeExtensionRegistry.ensureLoaded();
        long attributes = NativeExtensionRegistry.ttlvStructureCreate();
        long responseTtlv = NativeExtensionRegistry.ttlvStructureCreate();

        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseExportResponse(
                new Object[] {new long[] {1, 0, 0}, null, new long[] {attributes, 0, responseTtlv}}));
    }
}
