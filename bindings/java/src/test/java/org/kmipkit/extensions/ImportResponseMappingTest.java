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

final class ImportResponseMappingTest {
    @Test
    void importRequestUsesTheJniBuilderAndPreservesExplicitFalse() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure attributes = TtlvStructure.create();
        TtlvStructure nestedObject = TtlvStructure.create();
        TtlvStructure anyObject = TtlvStructure.create();
        try {
            try (Tag tag = RawTag.fromRaw(0x0042_008f).checked();
                    TtlvItem object = TtlvItem.create(
                            tag, TtlvValue.structure(nestedObject, limits), limits)) {
                anyObject = TtlvStructure.withItem(anyObject, object, limits);
            }
            try (ClientBatchItem ignored = ClientBatchItem.importObject(
                    UniqueIdentifier.textString("import-id"), 0xf1234567L,
                    Boolean.FALSE, null, attributes, anyObject)) {
                assertTrue(true);
            }
        } finally {
            anyObject.close();
            nestedObject.close();
            attributes.close();
        }
    }

    @Test
    void requestRejectsMissingRequiredInputsAndOutOfRangeRawSelectors() {
        TtlvStructure attributes = TtlvStructure.create();
        TtlvStructure anyObject = TtlvStructure.create();
        try {
            assertThrows(InvalidInputException.class, () -> ClientBatchItem.importObject(
                    null, 1, null, null, attributes, anyObject));
            assertThrows(InvalidInputException.class, () -> ClientBatchItem.importObject(
                    UniqueIdentifier.textString("id"), 1L << 32, null, null,
                    attributes, anyObject));
            assertThrows(InvalidInputException.class, () -> ClientBatchItem.importObject(
                    UniqueIdentifier.textString("id"), 1, null, 1L << 32,
                    attributes, anyObject));
        } finally {
            anyObject.close();
            attributes.close();
        }
    }

    @Test
    void successfulResponseOwnsOrderedTtlvAndRedactsIdentifier() {
        TtlvStructure source = responseTtlv();
        long handle = source.nativeHandleOwner().transfer();
        ClientBatchItemResult.ImportResponse response = ClientBatchItemResult
                .parseImportResponse(new Object[] {
                    new long[] {1},
                    new Object[] {new long[] {1, 1, 0, 0},
                            "imported-secret-id".getBytes(StandardCharsets.UTF_8)},
                    new long[] {handle}
                })
                .orElseThrow();
        source.close();
        try (response; TtlvStructureView view = response.responseTtlv().view()) {
            assertEquals("imported-secret-id", response.uniqueIdentifier().text());
            assertEquals(2, view.itemCount());
            assertFalse(response.toString().contains("imported-secret-id"));
        }
    }

    @Test
    void pendingOrFailureHasNoTypedSuccessResponseAndMalformedTupleIsRejected() {
        assertTrue(ClientBatchItemResult.parseImportResponse(
                new Object[] {new long[] {0}, null, new long[] {0}}).isEmpty());
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                .parseImportResponse(new Object[] {
                    new long[] {1}, null, new long[] {0}
                }));
        NativeExtensionRegistry.ensureLoaded();
    }

    @Test
    void malformedNativeTuplesReleaseOwnedImportTtlvHandles() {
        long absentWithTtlv = responseTtlvHandle();
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseImportResponse(
                new Object[] {new long[] {0}, null, new long[] {absentWithTtlv}}));
        assertNativeStructureHandleReleased(absentWithTtlv);

        long incompleteSuccess = responseTtlvHandle();
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseImportResponse(
                new Object[] {new long[] {1}, null, new long[] {incompleteSuccess}}));
        assertNativeStructureHandleReleased(incompleteSuccess);

        long invalidIdentifier = responseTtlvHandle();
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseImportResponse(
                new Object[] {new long[] {1}, new Object[] {new Object(), null},
                        new long[] {invalidIdentifier}}));
        assertNativeStructureHandleReleased(invalidIdentifier);
    }

    private static void assertNativeStructureHandleReleased(long handle) {
        assertThrows(InvalidInputException.class, () -> {
            try (TtlvStructureView view = TtlvStructureView.fromNative(
                    NativeExtensionRegistry.ttlvStructureView(handle), null)) {
                view.itemCount();
            }
        });
    }

    private static TtlvStructure responseTtlv() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure response = TtlvStructure.create();
        response = append(response, 0x0042_005c, TtlvValue.Enumeration(0), limits);
        return append(response, 0x0054_0001,
                TtlvValue.ByteString(new byte[] {(byte) 0xde, 0, (byte) 0xff}, limits), limits);
    }

    private static long responseTtlvHandle() {
        TtlvStructure source = responseTtlv();
        long handle = source.nativeHandleOwner().transfer();
        source.close();
        return handle;
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        try (Tag tag = RawTag.fromRaw(rawTag).checked();
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, item, limits);
        }
    }
}
