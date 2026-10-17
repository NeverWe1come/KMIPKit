package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

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
}
