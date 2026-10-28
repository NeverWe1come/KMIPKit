package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
import org.junit.jupiter.api.Test;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

final class GetAttributeListJavaFacadeTest {
    @Test
    void buildsGetAttributeListWithAnOmittedIdentifier() {
        try (ClientBatchItem omitted = ClientBatchItem.getAttributeList();
                ClientBatchItem selected = ClientBatchItem.getAttributeList(
                        org.kmipkit.extensions.UniqueIdentifier.textString("object-7"))) {
            assertNotNull(omitted);
            assertNotNull(selected);
        }
    }

    @Test
    void successfulResponseOwnsTheResponsePayload() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure payload = TtlvStructure.create();
        payload = append(
                payload,
                0x0042_0094,
                TtlvValue.TextString("object-7".getBytes(StandardCharsets.UTF_8), limits),
                limits);
        payload = append(
                payload,
                0x0042_013b,
                TtlvValue.Enumeration(0x0042_002f),
                limits);
        long handle = payload.nativeHandleOwner().transfer();
        ClientBatchItemResult.GetAttributeListResponse mapped = ClientBatchItemResult
                .parseGetAttributeListResponse(new Object[] {new long[] {1}, new long[] {handle}})
                .orElseThrow();

        try (mapped; TtlvStructureView view = mapped.responsePayloadTtlv().view()) {
            assertEquals(2, view.itemCount());
            assertEquals(0x0042_0094, tagAt(view, 0));
            assertEquals(0x0042_013b, tagAt(view, 1));
        }
    }

    @Test
    void failureHasNoSuccessfulResponseTtlv() {
        assertTrue(ClientBatchItemResult
                .parseGetAttributeListResponse(new Object[] {new long[] {0}, new long[] {0}})
                .isEmpty());
    }

    @Test
    void malformedSuccessRejectsMissingOwnedTtlv() {
        org.junit.jupiter.api.Assertions.assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseGetAttributeListResponse(
                        new Object[] {new long[] {1}, new long[] {0}}));
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
