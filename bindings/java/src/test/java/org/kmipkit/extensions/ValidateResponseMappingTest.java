package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.junit.jupiter.api.Test;

/** Validate response mapping derived from OASIS KMIP 2.1 §6.1.57, Tables 341–342. */
final class ValidateResponseMappingTest {
    @Test
    void successPreservesUnknownValidityIndicatorAndOwnsGenericResponseTtlv() {
        long responseHandle = TtlvStructure.create().nativeHandleOwner().transfer();
        ClientBatchItemResult.ValidateResponse response = ClientBatchItemResult
                .parseValidateResponse(new Object[] {
                    new long[] {1, 0xf123_4567L}, new long[] {responseHandle}
                })
                .orElseThrow();

        try (response; TtlvStructureView view = response.responseTtlv().view()) {
            assertEquals(0xf123_4567L, response.validityIndicatorRaw());
            assertEquals(0, view.itemCount());
            assertFalse(response.toString().contains("response-ttlv"));
        }

        assertNativeStructureHandleReleased(responseHandle);
    }

    @Test
    void failureAndPendingDoNotExposeASuccessPayload() {
        assertTrue(ClientBatchItemResult
                .parseValidateResponse(new Object[] {new long[] {0, 0}, new long[] {0}})
                .isEmpty());
    }

    @Test
    void malformedPresenceAndUnexpectedOwnedTtlvAreRejectedAndReleased() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                .parseValidateResponse(new Object[] {new long[] {1, 0}, new long[] {0}}));

        long unexpected = TtlvStructure.create().nativeHandleOwner().transfer();
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                .parseValidateResponse(new Object[] {new long[] {0, 0}, new long[] {unexpected}}));
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
}
