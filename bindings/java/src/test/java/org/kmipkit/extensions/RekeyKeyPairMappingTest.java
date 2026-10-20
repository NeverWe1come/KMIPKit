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
import org.kmipkit.ttlv.TtlvStructureView;
import org.junit.jupiter.api.Test;

/** Re-key Key Pair mapping tests for OASIS KMIP 2.1 §6.1.47, Tables 308–312. */
final class RekeyKeyPairMappingTest {
    @Test
    void requestBuilderAllowsServerPlaceholderAndOptionalOffset() {
        try (ClientBatchItem omitted = ClientBatchItem.rekeyKeyPair(null, null, null, null, null, null, null, null);
                ClientBatchItem selected = ClientBatchItem.rekeyKeyPair(
                        UniqueIdentifier.textString("existing-private-key"), 0xffff_ffffL,
                        null, null, null, null, null, null)) {
            assertNotNull(omitted);
            assertNotNull(selected);
        }
    }

    @Test
    void requestBuilderRejectsOffsetOutsideUnsignedInterval() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.rekeyKeyPair(
                null, 0x1_0000_0000L, null, null, null, null, null, null));
    }

    @Test
    void requestBuilderCopiesAllSixOptionalStructures() {
        try (TtlvStructure commonAttributes = TtlvStructure.create();
                TtlvStructure privateKeyAttributes = TtlvStructure.create();
                TtlvStructure publicKeyAttributes = TtlvStructure.create();
                TtlvStructure commonMasks = TtlvStructure.create();
                TtlvStructure privateMasks = TtlvStructure.create();
                TtlvStructure publicMasks = TtlvStructure.create();
                ClientBatchItem item = ClientBatchItem.rekeyKeyPair(
                        null, 86_400L, commonAttributes, privateKeyAttributes, publicKeyAttributes,
                        commonMasks, privateMasks, publicMasks)) {
            for (TtlvStructure structure : new TtlvStructure[] {
                    commonAttributes, privateKeyAttributes, publicKeyAttributes,
                    commonMasks, privateMasks, publicMasks}) {
                try (TtlvStructureView view = structure.view()) {
                    assertEquals(1, view.itemCount());
                }
            }
            assertNotNull(item);
        }
    }

    @Test
    void successfulResponseRetainsBothIdentifiersAndRedactsThem() {
        NativeExtensionRegistry.ensureLoaded();
        byte[] privateId = "replacement-private".getBytes(StandardCharsets.UTF_8);
        byte[] publicId = "replacement-public".getBytes(StandardCharsets.UTF_8);
        Object[] privateRecord = {new long[] {1, 1, 0, 0}, privateId};
        Object[] publicRecord = {new long[] {1, 1, 0, 0}, publicId};
        long ttlvHandle = NativeExtensionRegistry.ttlvStructureCreate();
        try (ClientBatchItemResult.RekeyKeyPairResponse response = ClientBatchItemResult
                .parseRekeyKeyPairResponse(new Object[] {
                    new long[] {1}, privateRecord, publicRecord, new long[] {ttlvHandle}
                }).orElseThrow()) {
            assertEquals("replacement-private", response.privateKeyUniqueIdentifier().text());
            assertEquals("replacement-public", response.publicKeyUniqueIdentifier().text());
            assertNotNull(response.responseTtlv());
            assertFalse(response.toString().contains("replacement-private"));
            assertFalse(response.toString().contains("replacement-public"));
        }
    }

    @Test
    void absentPairResponseDoesNotAcceptIdentifiersOrOwnedTtlv() {
        assertTrue(ClientBatchItemResult.parseRekeyKeyPairResponse(
                new Object[] {new long[] {0}, null, null, new long[] {0}}).isEmpty());
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                .parseRekeyKeyPairResponse(new Object[] {new long[] {0}, new Object(), null, new long[] {0}}));
    }
}
