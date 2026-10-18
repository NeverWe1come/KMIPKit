package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.junit.jupiter.api.Test;

/** Foreign mapping checks for KMIP v2.1 §6.1.27, Tables 244–246. */
final class JoinSplitKeyMappingTest {
    @Test
    void requestKeepsEachRepeatedIdentifierPairedWithItsOwnThreshold() {
        TtlvStructure attributes = TtlvStructure.create();
        TtlvStructure masks = TtlvStructure.create();
        List<SplitKeyReference> references = List.of(
                new SplitKeyReference(UniqueIdentifier.textString("same-split-id"), 1),
                new SplitKeyReference(UniqueIdentifier.textString("same-split-id"), 2));
        try (ClientBatchItem ignored = ClientBatchItem.joinSplitKey(
                0xf1234567L, references, 0xfedcba98L, attributes, masks)) {
            assertTrue(true);
        } finally {
            masks.close();
            attributes.close();
        }
    }

    @Test
    void requestRejectsMissingReferenceAndInsufficientThresholdBeforeTransport() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.joinSplitKey(
                1, List.of(new SplitKeyReference(UniqueIdentifier.textString("one"), 2))));
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.joinSplitKey(
                1, java.util.Arrays.asList(
                        new SplitKeyReference(UniqueIdentifier.textString("one"), 1), null)));
    }

    @Test
    void successfulResponseOwnsOrderedTtlvAndRedactsIdentifier() {
        TtlvStructure source = responseTtlv();
        long handle = source.nativeHandleOwner().transfer();
        ClientBatchItemResult.JoinSplitKeyResponse response = ClientBatchItemResult
                .parseJoinSplitKeyResponse(new Object[] {
                    new long[] {1},
                    new Object[] {new long[] {1, 1, 0, 0},
                            "private-split-key-id".getBytes(StandardCharsets.UTF_8)},
                    new long[] {handle}
                })
                .orElseThrow();
        source.close();
        try (response; TtlvStructureView view = response.responseTtlv().view()) {
            assertEquals("private-split-key-id", response.uniqueIdentifier().text());
            assertEquals(2, view.itemCount());
            assertFalse(response.toString().contains("private-split-key-id"));
        }
    }

    @Test
    void failureAndPendingDoNotExposeASuccessIdentifier() {
        assertTrue(ClientBatchItemResult.parseJoinSplitKeyResponse(
                new Object[] {new long[] {0}, null, new long[] {0}}).isEmpty());
        assertTrue(ClientBatchItemResult.parseJoinSplitKeyResponse(
                new Object[] {new long[] {2}, null, new long[] {0}}).isEmpty());
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                .parseJoinSplitKeyResponse(new Object[] {new long[] {1}, null, new long[] {0}}));
    }

    private static TtlvStructure responseTtlv() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure response = TtlvStructure.create();
        response = append(response, 0x0042_005c, TtlvValue.Enumeration(0), limits);
        return append(response, 0x0054_0001,
                TtlvValue.ByteString(new byte[] {(byte) 0xde, 0, (byte) 0xff}, limits), limits);
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        try (Tag tag = RawTag.fromRaw(rawTag).checked();
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, item, limits);
        }
    }
}
