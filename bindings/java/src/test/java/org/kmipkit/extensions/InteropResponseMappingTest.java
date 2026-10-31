package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
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

final class InteropResponseMappingTest {
    @Test
    void completeTtlvIsCallerOwnedAndItsFormattingIsRedacted() {
        TtlvStructure source = responseTtlv();
        long handle = source.nativeHandleOwner().transfer();
        ClientBatchItemResult.InteropResponse response = ClientBatchItemResult
                .parseInteropResponse(new Object[] {new long[] {1}, new long[] {handle}})
                .orElseThrow();
        source.close();

        try (response; TtlvStructureView view = response.responseTtlv().view()) {
            assertEquals(1, view.itemCount());
            assertFalse(response.toString().contains("interop-response-private-sentinel"));
        }

        assertNativeStructureHandleReleased(handle);
    }

    @Test
    void absentResponseDoesNotCreateAnOwnedTtlv() {
        assertTrue(ClientBatchItemResult
                .parseInteropResponse(new Object[] {new long[] {0}, new long[] {0}})
                .isEmpty());
    }

    @Test
    void malformedAndUnexpectedResponseHandlesAreRejectedAndReleased() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                .parseInteropResponse(new Object[] {new long[] {1}, new long[] {0}}));

        long unexpected = responseTtlvHandle();
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                .parseInteropResponse(new Object[] {new long[] {0}, new long[] {unexpected}}));
        assertNativeStructureHandleReleased(unexpected);
    }

    private static void assertNativeStructureHandleReleased(long handle) {
        assertThrows(InvalidInputException.class, () -> {
            try (TtlvStructureView view = TtlvStructureView.fromNative(
                    NativeExtensionRegistry.ttlvStructureView(handle), null)) {
                view.itemCount();
            }
        });
    }

    private static long responseTtlvHandle() {
        return responseTtlv().nativeHandleOwner().transfer();
    }

    private static TtlvStructure responseTtlv() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure response = TtlvStructure.create();
        try (Tag tag = RawTag.fromRaw(0x0054_1234).checked();
                TtlvItem item = TtlvItem.create(
                        tag,
                        TtlvValue.TextString(
                                "interop-response-private-sentinel".getBytes(StandardCharsets.UTF_8),
                                limits),
                        limits)) {
            response = TtlvStructure.withItem(response, item, limits);
        }
        return response;
    }
}
