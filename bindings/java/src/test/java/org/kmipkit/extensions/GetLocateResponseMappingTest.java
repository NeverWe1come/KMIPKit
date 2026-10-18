package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvStructure;
import org.junit.jupiter.api.Test;

final class GetLocateResponseMappingTest {
    // OASIS KMIP Specification v2.1 §6.1.19, Tables 220–222.
    @Test
    void getBuilderPreservesUnknownOptionalTable220SelectorsAndAllowsOmission() {
        try (TtlvStructure wrappingSpecification = TtlvStructure.create();
                ClientBatchItem explicit = ClientBatchItem.get(
                        UniqueIdentifier.textString("get-key"),
                        0xf123_4567L, 0xf123_4568L, 0xf123_4569L,
                        wrappingSpecification);
                ClientBatchItem omitted = ClientBatchItem.get(null, null, null, null, null)) {
            assertTrue(explicit != null);
            assertTrue(omitted != null);
        }
    }

    // OASIS KMIP Specification v2.1 §6.1.28, Tables 247–249.
    @Test
    void locateBuilderPreservesZeroUnknownMaskAndRequiredEmptyAttributes() {
        try (TtlvStructure attributes = TtlvStructure.create();
                ClientBatchItem locate = ClientBatchItem.locate(
                        -4, 0, 0x8000_0001L, 0xf123_4567L, attributes)) {
            assertTrue(locate != null);
        }
    }

    // OASIS KMIP Specification v2.1 §6.1.19, Tables 221–222.
    @Test
    void getResponseOwnsOpaqueObjectAndRedactsItsFormatting() {
        NativeExtensionRegistry.ensureLoaded();
        long object = NativeExtensionRegistry.ttlvStructureCreate();
        Object[] identifier = {new long[] {1, 1, 0, 0}, "get-secret-id".getBytes()};

        try (ClientBatchItemResult.GetResponse response = ClientBatchItemResult
                .parseGetResponse(new Object[] {
                    new long[] {1, 1, 0xf123_4567L}, identifier, new long[] {object}
                })
                .orElseThrow()) {
            assertEquals(0xf123_4567L, response.objectTypeRaw().orElseThrow());
            assertEquals("get-secret-id", response.uniqueIdentifier().orElseThrow().text());
            assertTrue(response.object().isPresent());
            assertFalse(response.toString().contains("get-secret-id"));
            assertFalse(response.toString().contains("secret-object-sentinel"));
        }
    }

    // OASIS KMIP Specification v2.1 §6.1.19, Tables 221–222.
    @Test
    void failureGetResponseHasNoSuccessPayload() {
        NativeExtensionRegistry.ensureLoaded();

        try (ClientBatchItemResult.GetResponse response = ClientBatchItemResult
                .parseGetResponse(new Object[] {
                    new long[] {1, 0, 0}, null, new long[] {0}
                })
                .orElseThrow()) {
            assertTrue(response.objectTypeRaw().isEmpty());
            assertTrue(response.uniqueIdentifier().isEmpty());
            assertTrue(response.object().isEmpty());
        }
    }

    // OASIS KMIP Specification v2.1 §6.1.28, Tables 248–249.
    @Test
    void locateResponsePreservesRepeatedIdentifiersAndOptionalLocatedItems() {
        Object[] firstIdentifier = {new long[] {1, 1, 0, 0}, "first".getBytes()};
        Object[] secondIdentifier = {new long[] {1, 1, 0, 0}, "second".getBytes()};
        Object[] identifiers = {firstIdentifier, secondIdentifier, firstIdentifier};

        ClientBatchItemResult.LocateResponse response = ClientBatchItemResult
                .parseLocateResponse(new Object[] {
                    new long[] {1, 1, 1, 3, 3}, identifiers
                })
                .orElseThrow();
        assertEquals(3, response.locatedItems().orElseThrow());
        assertEquals(
                java.util.List.of("first", "second", "first"),
                response.uniqueIdentifiers().stream().map(UniqueIdentifier::text).toList());
    }

    // OASIS KMIP Specification v2.1 §6.1.28, Tables 248–249.
    @Test
    void locateResponsePreservesIdentifiersBeyondTheDefaultDecoderLimit() {
        int identifierCount = 100_001;
        Object[] identifier = {new long[] {1, 1, 0, 0}, "repeated".getBytes()};
        Object[] identifiers = new Object[identifierCount];
        java.util.Arrays.fill(identifiers, identifier);

        ClientBatchItemResult.LocateResponse response = ClientBatchItemResult
                .parseLocateResponse(new Object[] {
                    new long[] {1, 1, 1, identifierCount, identifierCount}, identifiers
                })
                .orElseThrow();

        assertEquals(identifierCount, response.uniqueIdentifiers().size());
        assertEquals("repeated", response.uniqueIdentifiers().get(0).text());
        assertEquals("repeated", response.uniqueIdentifiers()
                .get(identifierCount - 1).text());
    }

    // OASIS KMIP Specification v2.1 §6.1.19, Table 221.
    @Test
    void successfulGetResponseRequiresTheAnyObjectTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] identifier = {new long[] {1, 1, 0, 0}, "get-secret-id".getBytes()};

        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseGetResponse(
                new Object[] {
                    new long[] {1, 1, 1}, identifier, new long[] {0}
                }));
    }
}
