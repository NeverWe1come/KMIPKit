package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
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
        try (TtlvStructure commonAttributes = requestStructure(0x541101, "common-attributes");
                TtlvStructure privateKeyAttributes = requestStructure(0x541102, "private-attributes");
                TtlvStructure publicKeyAttributes = requestStructure(0x541103, "public-attributes");
                TtlvStructure commonMasks = requestStructure(0x541104, "common-masks");
                TtlvStructure privateMasks = requestStructure(0x541105, "private-masks");
                TtlvStructure publicMasks = requestStructure(0x541106, "public-masks");
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

    private static TtlvStructure requestStructure(int markerTag, String marker) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure empty = TtlvStructure.create();
        try (RawTag rawTag = RawTag.fromRaw(markerTag);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.ByteString(
                        marker.getBytes(StandardCharsets.US_ASCII), limits);
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(empty, item, limits);
        } finally {
            empty.close();
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

    @Test
    void malformedSuccessClosesOwnedTtlvWhenAnIdentifierIsInvalid() {
        NativeExtensionRegistry.ensureLoaded();
        long malformedResponseTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        Object[] validPrivateIdentifier = {
            new long[] {1, 1, 0, 0}, "replacement-private".getBytes(StandardCharsets.UTF_8)
        };

        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseRekeyKeyPairResponse(
                new Object[] {new long[] {1}, validPrivateIdentifier, new Object(),
                    new long[] {malformedResponseTtlv}}));

        ResponseTtlvTestSupport.assertReleased(malformedResponseTtlv);
    }
}
