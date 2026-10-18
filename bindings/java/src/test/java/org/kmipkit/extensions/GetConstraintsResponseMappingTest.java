package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;
import org.junit.jupiter.api.Test;

final class GetConstraintsResponseMappingTest {
    @Test
    void successfulResponseOwnsOrderedTtlvWithRequiredConstraintsAndUnknownMembers() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure callerCopy = responseTtlv(limits);
        long copiedHandle = callerCopy.nativeHandleOwner().transfer();
        ClientBatchItemResult.GetConstraintsResponse response = ClientBatchItemResult
                .parseGetConstraintsResponse(new Object[] {new long[] {1}, new long[] {copiedHandle}})
                .orElseThrow();

        try (response; TtlvStructureView view = response.responseTtlv().view()) {
            assertEquals(4, view.itemCount());
            assertEquals(0x0042_005c, tagAt(view, 0));
            assertEquals(0x0042_0093, tagAt(view, 1));
            assertEquals(0x0042_007f, tagAt(view, 2));
            assertEquals(0x0042_007c, tagAt(view, 3));
            try (TtlvItemView payload = view.itemAt(3);
                    TtlvValueView payloadValue = payload.value();
                    TtlvStructureView payloadView = payloadValue.structure()) {
                assertEquals(0x0042_0168, tagAt(payloadView, 0));
                assertEquals(0x0054_0002, tagAt(payloadView, 1));
                try (TtlvItemView constraints = payloadView.itemAt(0);
                        TtlvValueView constraintsValue = constraints.value();
                        TtlvStructureView constraintsView = constraintsValue.structure()) {
                    assertEquals(0x0054_0001, tagAt(constraintsView, 0));
                }
            }
            assertFalse(response.toString().contains("constraints-secret-sentinel"));
        }
    }

    @Test
    void absentResponseHasNoOwnedTtlv() {
        assertTrue(ClientBatchItemResult
                .parseGetConstraintsResponse(new Object[] {new long[] {0}, new long[] {0}})
                .isEmpty());
    }

    @Test
    void malformedResponsesRejectMissingOwnedTtlv() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseGetConstraintsResponse(
                        new Object[] {new long[] {1}, new long[] {0}}));
    }

    @Test
    void absentResponseReleasesUnexpectedOwnedTtlvBeforeRejectingIt() {
        TtlvStructure unexpected = TtlvStructure.create();
        long handle = unexpected.nativeHandleOwner().transfer();

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseGetConstraintsResponse(
                        new Object[] {new long[] {0}, new long[] {handle}}));
        ResponseTtlvTestSupport.assertReleased(handle);
    }

    private static TtlvStructure responseTtlv(CodecLimits limits) {
        TtlvStructure constraints = TtlvStructure.create();
        constraints = append(constraints, 0x0054_0001,
                TtlvValue.TextString("constraints-secret-sentinel".getBytes(StandardCharsets.UTF_8), limits), limits);
        TtlvStructure payload = TtlvStructure.create();
        payload = append(payload, 0x0042_0168, TtlvValue.structure(constraints, limits), limits);
        payload = append(payload, 0x0054_0002,
                TtlvValue.ByteString(new byte[] {(byte) 0xde, 0, (byte) 0xff}, limits), limits);
        TtlvStructure response = TtlvStructure.create();
        response = append(response, 0x0042_005c, TtlvValue.Enumeration(0x38), limits);
        response = append(response, 0x0042_0093,
                TtlvValue.ByteString("item-0".getBytes(StandardCharsets.UTF_8), limits), limits);
        response = append(response, 0x0042_007f, TtlvValue.Enumeration(0), limits);
        return append(response, 0x0042_007c, TtlvValue.structure(payload, limits), limits);
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        try (Tag tag = RawTag.fromRaw(rawTag).checked(); TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, item, limits);
        }
    }

    private static long tagAt(TtlvStructureView view, long index) {
        try (TtlvItemView item = view.itemAt(index); Tag tag = item.tag()) {
            return tag.raw();
        }
    }
}
