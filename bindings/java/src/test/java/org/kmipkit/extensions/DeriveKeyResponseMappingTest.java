package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.junit.jupiter.api.Test;

final class DeriveKeyResponseMappingTest {
    @Test
    void successfulResponseMapsIdentifierAndRetainsOrderedUnknownFields() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = responseTtlv(limits);
        long responseTtlvHandle = structure.nativeHandleOwner().transfer();
        byte[] identifier = "derived-key".getBytes(StandardCharsets.UTF_8);
        Object[] record = {new long[] {1, 1, 0, 0}, identifier};

        ClientBatchItemResult.DeriveKeyResponse response = ClientBatchItemResult
                .parseDeriveKeyResponse(new Object[] {
                    new long[] {1}, record, new long[] {responseTtlvHandle}
                })
                .orElseThrow();

        try (response; TtlvStructureView view = response.responseTtlv().view()) {
            assertEquals(UniqueIdentifier.Kind.TEXT_STRING, response.uniqueIdentifier().kind());
            assertEquals("derived-key", response.uniqueIdentifier().text());
            assertEquals(2, view.itemCount());
            assertEquals(0x0042_0094, tagAt(view, 0));
            assertEquals(0x0054_0001, tagAt(view, 1));
            assertFalse(response.toString().contains("derived-key"));
        }
    }

    @Test
    void absentResponseDoesNotCreateAnIdentifierOrTtlv() {
        assertTrue(ClientBatchItemResult
                .parseDeriveKeyResponse(new Object[] {new long[] {0}, null, new long[] {0}})
                .isEmpty());
    }

    private static TtlvStructure responseTtlv(CodecLimits limits) {
        TtlvStructure response = TtlvStructure.create();
        response = append(response, 0x0042_0094,
                TtlvValue.TextString("derived-key".getBytes(StandardCharsets.UTF_8), limits), limits);
        return append(response, 0x0054_0001,
                TtlvValue.ByteString(new byte[] {(byte) 0xde, (byte) 0xad}, limits), limits);
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        try (Tag tag = RawTag.fromRaw(rawTag).checked();
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, item, limits);
        }
    }

    private static long tagAt(TtlvStructureView view, long index) {
        try (TtlvItemView item = view.itemAt(index); Tag tag = item.tag()) {
            return tag.raw();
        }
    }
}
