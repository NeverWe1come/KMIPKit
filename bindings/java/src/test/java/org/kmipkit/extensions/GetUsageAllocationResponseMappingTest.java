package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
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
import org.kmipkit.ttlv.TtlvValueView;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.junit.jupiter.api.Test;

final class GetUsageAllocationResponseMappingTest {
    @Test
    void successfulResponseMapsIdentifierAndKeepsItRedacted() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = responseTtlv(limits);
        long responseTtlvHandle = structure.nativeHandleOwner().transfer();
        byte[] identifier = "allocation-object".getBytes(StandardCharsets.UTF_8);
        Object[] record = {new long[] {1, 1, 0, 0}, identifier};

        ClientBatchItemResult.GetUsageAllocationResponse response = ClientBatchItemResult
                .parseGetUsageAllocationResponse(new Object[] {
                    new long[] {1}, record, new long[] {responseTtlvHandle}
                })
                .orElseThrow();

        try (response) {
            assertEquals(UniqueIdentifier.Kind.TEXT_STRING, response.uniqueIdentifier().kind());
            assertEquals("allocation-object", response.uniqueIdentifier().text());
            assertFalse(response.toString().contains("allocation-object"));
        }
    }

    @Test
    void absentResponseDoesNotCreateAnIdentifier() {
        assertTrue(ClientBatchItemResult
                .parseGetUsageAllocationResponse(
                        new Object[] {new long[] {0}, null, new long[] {0}})
                .isEmpty());
    }

    @Test
    void successfulResponseOwnsCompleteOrderedTtlvAfterSourceRelease() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure source = responseTtlv(limits);
        TtlvStructure callerCopy = responseTtlv(limits);
        long copiedHandle = callerCopy.nativeHandleOwner().transfer();
        Object[] identifier = {new long[] {1, 1, 0, 0},
                "allocation-object".getBytes(StandardCharsets.UTF_8)};
        ClientBatchItemResult.GetUsageAllocationResponse response = ClientBatchItemResult
                .parseGetUsageAllocationResponse(new Object[] {
                    new long[] {1}, identifier, new long[] {copiedHandle}
                })
                .orElseThrow();

        source.close();
        try (response; TtlvStructureView view = response.responseTtlv().view()) {
            assertEquals(4, view.itemCount());
            assertEquals(0x0042_005c, tagAt(view, 0));
            assertEquals(0x0042_0093, tagAt(view, 1));
            assertEquals(0x0042_007f, tagAt(view, 2));
            assertEquals(0x0042_007c, tagAt(view, 3));
            try (TtlvItemView payload = view.itemAt(3);
                    TtlvValueView payloadValue = payload.value();
                    TtlvStructureView payloadView = payloadValue.structure()) {
                assertEquals(0x0042_0094, tagAt(payloadView, 0));
                assertEquals(0x0054_0001, tagAt(payloadView, 1));
            }
            assertFalse(response.toString().contains("allocation-object"));
        }
    }

    @Test
    void malformedResponsesReleaseOwnedTtlvAndRejectMissingTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long absentResponseTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseGetUsageAllocationResponse(new Object[] {
                    new long[] {0}, null, new long[] {absentResponseTtlv}
                }));
        ResponseTtlvTestSupport.assertReleased(absentResponseTtlv);

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseGetUsageAllocationResponse(new Object[] {
                    new long[] {1}, new Object[] {new long[] {1, 1, 0, 0}, null}, new long[] {0}
                }));

        long malformedIdentifierTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseGetUsageAllocationResponse(new Object[] {
                    new long[] {1}, new Object[] {new long[] {0, 1, 0, 0}, null},
                    new long[] {malformedIdentifierTtlv}
                }));
        ResponseTtlvTestSupport.assertReleased(malformedIdentifierTtlv);
    }

    private static TtlvStructure responseTtlv(CodecLimits limits) {
        TtlvStructure payload = TtlvStructure.create();
        payload = append(payload, 0x0042_0094,
                TtlvValue.TextString("allocation-object".getBytes(StandardCharsets.UTF_8), limits), limits);
        payload = append(payload, 0x0054_0001,
                TtlvValue.ByteString(new byte[] {(byte) 0xde, (byte) 0xad, 0, (byte) 0xff}, limits), limits);
        TtlvStructure response = TtlvStructure.create();
        response = append(response, 0x0042_005c, TtlvValue.Enumeration(0x11), limits);
        response = append(response, 0x0042_0093,
                TtlvValue.ByteString("item-0".getBytes(StandardCharsets.UTF_8), limits), limits);
        response = append(response, 0x0042_007f, TtlvValue.Enumeration(0), limits);
        return append(response, 0x0042_007c, TtlvValue.structure(payload, limits), limits);
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
