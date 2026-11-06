package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
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

        try (TtlvStructure attributes = TtlvStructure.create();
                ClientBatchItem omitted = ClientBatchItem.locate(null, null, null, null, attributes)) {
            assertTrue(omitted != null);
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
            assertTrue(response.toString().contains("objectTypeRaw=OptionalLong.empty"));
        }
    }

    @Test
    void absentGetResponseIsEmptyAndMalformedIdentifierReleasesOwnedObject() {
        assertTrue(ClientBatchItemResult.parseGetResponse(new Object[] {
            new long[] {0, 0, 0}, null, new long[] {0}
        }).isEmpty());

        NativeExtensionRegistry.ensureLoaded();
        long ownedObject = NativeExtensionRegistry.ttlvStructureCreate();
        Object[] malformedIdentifier = {new long[] {1, 99, 0, 0}, new byte[] {0x41}};
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseGetResponse(
                new Object[] {
                    new long[] {1, 1, 7}, malformedIdentifier, new long[] {ownedObject}
                }));
        ResponseTtlvTestSupport.assertReleased(ownedObject);
    }

    @Test
    void getResponseRejectsMalformedNativeTuplesAndHandlesStaleOwnedHandles() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseGetResponse(
                new Object[] {new long[] {2, 0, 0}, null, new long[] {0}}));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseGetResponse(
                new Object[] {new long[] {1, 0, 7}, null, new long[] {0}}));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseGetResponse(
                new Object[] {new long[] {1, 1, 7}, null, new long[] {0}}));

        NativeExtensionRegistry.ensureLoaded();
        long releasedObject = NativeExtensionRegistry.ttlvStructureCreate();
        NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE, releasedObject);
        Object[] identifier = {new long[] {1, 99, 0, 0}, new byte[] {0x41}};
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseGetResponse(
                new Object[] {new long[] {1, 1, 7}, identifier, new long[] {releasedObject}}));
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

    @Test
    void locateResponseCoversAbsentFailureAndOmittedLocatedItems() {
        assertTrue(ClientBatchItemResult.parseLocateResponse(new Object[] {
            new long[] {0, 0, 0, 0, 0}, new Object[0]
        }).isEmpty());

        ClientBatchItemResult.LocateResponse failed = ClientBatchItemResult
                .parseLocateResponse(new Object[] {
                    new long[] {1, 0, 0, 0, 0}, new Object[0]
                })
                .orElseThrow();
        assertTrue(failed.locatedItems().isEmpty());
        assertTrue(failed.uniqueIdentifiers().isEmpty());

        Object[] identifier = {new long[] {1, 1, 0, 0}, "located".getBytes()};
        ClientBatchItemResult.LocateResponse withoutLocatedItems = ClientBatchItemResult
                .parseLocateResponse(new Object[] {
                    new long[] {1, 1, 0, 0, 1}, new Object[] {identifier}
                })
                .orElseThrow();
        assertTrue(withoutLocatedItems.locatedItems().isEmpty());
        assertEquals(java.util.List.of("located"), withoutLocatedItems.uniqueIdentifiers()
                .stream().map(UniqueIdentifier::text).toList());
    }

    @Test
    void locateResponseRejectsInconsistentPresenceAndIdentifierCount() {
        Object[] identifier = {new long[] {1, 1, 0, 0}, "located".getBytes()};
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseLocateResponse(
                new Object[] {new long[] {1, 1, 0, 4, 1}, new Object[] {identifier}}));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseLocateResponse(
                new Object[] {new long[] {1, 1, 0, 0, 0}, new Object[] {identifier}}));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseLocateResponse(
                new Object[] {new long[] {0, 0, 0, 0, 1}, new Object[] {identifier}}));
    }

    @Test
    void locateResponseRejectsMalformedTupleFieldsAndFailedPayloads() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseLocateResponse(
                new Object[] {new long[] {1, 1, 1, 0, 0}, "not-an-identifier-array"}));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseLocateResponse(
                new Object[] {new long[] {2, 0, 0, 0, 0}, new Object[0]}));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseLocateResponse(
                new Object[] {new long[] {1, 1, 1, 0x1_0000_0000L, 0}, new Object[0]}));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseLocateResponse(
                new Object[] {new long[] {1, 0, 0, 1, 0}, new Object[0]}));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseLocateResponse(
                new Object[] {new long[] {1, 1, 1, 1, 1}, new Object[] {"not-an-identifier-tuple"}}));
    }

    @Test
    void locateResponseRejectsIdentifierCountBeyondTheNativeArrayLimit() {
        int identifierCount = 100_001;
        Object[] identifiers = new Object[identifierCount];

        assertThrows(ResourceLimitException.class, () -> ClientBatchItemResult.parseLocateResponse(
                successfulLocateTuple(identifiers)));
    }

    // OASIS KMIP Specification v2.1 §6.1.28, Tables 248–249.
    @Test
    void locateResponsePreservesIdentifiersAtTheNativeArrayLimit() {
        int identifierCount = 100_000;
        Object[] identifier = {new long[] {1, 1, 0, 0}, "repeated".getBytes()};
        Object[] identifiers = new Object[identifierCount];
        java.util.Arrays.fill(identifiers, identifier);

        ClientBatchItemResult.LocateResponse response = ClientBatchItemResult
                .parseLocateResponse(successfulLocateTuple(identifiers))
                .orElseThrow();

        assertEquals(identifierCount, response.uniqueIdentifiers().size());
        assertEquals("repeated", response.uniqueIdentifiers().get(0).text());
        assertEquals("repeated", response.uniqueIdentifiers()
                .get(identifierCount - 1).text());
    }

    private static Object[] successfulLocateTuple(Object[] identifiers) {
        int identifierCount = identifiers.length;
        return new Object[] {
            new long[] {1, 1, 1, identifierCount, identifierCount}, identifiers
        };
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
