package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvValueView;
import org.junit.jupiter.api.Test;

/** Foreign mapping checks for KMIP v2.1 §6.1.27, Tables 244–246. */
final class JoinSplitKeyMappingTest {
    @Test
    void requestKeepsEachRepeatedIdentifierPairedWithItsOwnThreshold() {
        TtlvStructure attributes = TtlvStructure.create();
        TtlvStructure masks = TtlvStructure.create();
        try (SplitKeyReference first = new SplitKeyReference(
                        UniqueIdentifier.textString("same-split-id"), 1);
                SplitKeyReference second = new SplitKeyReference(
                        UniqueIdentifier.textString("same-split-id"), 2)) {
            List<SplitKeyReference> references = List.of(first, second);
            assertEquals("same-split-id", references.get(0).uniqueIdentifier().text());
            assertEquals("same-split-id", references.get(1).uniqueIdentifier().text());
            assertEquals(1, references.get(0).splitKeyThreshold());
            assertEquals(2, references.get(1).splitKeyThreshold());
            try (ClientBatchItem ignored = ClientBatchItem.joinSplitKey(
                    0xf1234567L, references, 0xfedcba98L, attributes, masks)) {
                assertEquals(2, references.size());
            }
        } finally {
            masks.close();
            attributes.close();
        }
    }

    @Test
    void requestRejectsMissingReferenceAndInsufficientThresholdBeforeTransport() {
        try (SplitKeyReference reference = new SplitKeyReference(
                UniqueIdentifier.textString("one"), 2)) {
            assertThrows(InvalidInputException.class, () -> ClientBatchItem.joinSplitKey(
                    1, List.of(reference)));
        }
        try (SplitKeyReference reference = new SplitKeyReference(
                UniqueIdentifier.textString("one"), 1)) {
            assertThrows(InvalidInputException.class, () -> ClientBatchItem.joinSplitKey(
                    1, java.util.Arrays.asList(reference, null)));
        }
    }

    @Test
    void requestRejectsEmptyOrNullReferenceLists() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.joinSplitKey(1, List.of()));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.joinSplitKey(1, null));
    }

    @Test
    void splitKeyReferenceRejectsInvalidConstructorArguments() {
        assertThrows(InvalidInputException.class, () -> new SplitKeyReference(null, 1));
        assertThrows(InvalidInputException.class, () -> new SplitKeyReference(
                UniqueIdentifier.textString("one"), -1));
        assertThrows(InvalidInputException.class, () -> new SplitKeyReference(
                UniqueIdentifier.textString("one"), 0x1_0000_0000L));
    }

    @Test
    void splitKeyReferenceRedactsIdentifierInToString() {
        try (SplitKeyReference reference = new SplitKeyReference(
                UniqueIdentifier.textString("private-split-key-id"), 3)) {
            assertEquals(
                    "SplitKeyReference[uniqueIdentifier=redacted, splitKeyThreshold=3]",
                    reference.toString());
            assertFalse(reference.toString().contains("private-split-key-id"));
        }
    }

    @Test
    void requestRejectsAggregateRepeatedIdentifierBytesAtTheNativeBoundary() {
        String identifier = "x".repeat((16 * 1024 * 1024) / 2 + 1);
        try (SplitKeyReference reference = new SplitKeyReference(
                UniqueIdentifier.textString(identifier), 1)) {
            assertThrows(ResourceLimitException.class, () -> ClientBatchItem.joinSplitKey(
                    1, List.of(reference, reference)));
        }
    }

    @Test
    void successfulResponseOwnsOrderedTtlvAndRedactsIdentifier() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure source = responseTtlv(limits);
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
            assertEquals(4, view.itemCount());
            assertEquals(0x0042_005c, tagAt(view, 0));
            assertEquals(0x0042_0093, tagAt(view, 1));
            assertEquals(0x0042_007f, tagAt(view, 2));
            assertEquals(0x0042_007c, tagAt(view, 3));
            try (TtlvItemView payload = view.itemAt(3);
                    TtlvValueView payloadValue = payload.value();
                    TtlvStructureView payloadView = payloadValue.structure()) {
                assertEquals(2, payloadView.itemCount());
                assertEquals(0x0042_0094, tagAt(payloadView, 0));
                assertEquals(0x0054_0001, tagAt(payloadView, 1));
            }
            assertFalse(response.toString().contains("private-split-key-id"));
        }
    }

    @Test
    void failureRetainsResultMetadataWithoutASuccessIdentifier() {
        ClientBatchItemMetadataV2 failed = ClientBatchItemResult.parseMetadataV2(new Object[] {
            new long[] {0x29, 1, 1, 1, 0xf1234567L, 1},
            "join-split-key-failed".getBytes(StandardCharsets.UTF_8)
        });
        assertEquals(0x29, failed.operation());
        assertEquals(1, failed.resultStatus());
        assertEquals(0xf1234567L, failed.resultReason().orElseThrow());
        assertEquals("join-split-key-failed", failed.resultMessage().orElseThrow());
        assertTrue(ClientBatchItemResult.parseJoinSplitKeyResponse(
                new Object[] {new long[] {0}, null, new long[] {0}}).isEmpty());
    }

    @Test
    void pendingRetainsOutcomeAndCorrelationMetadataWithoutASuccessIdentifier() {
        ClientBatchItemMetadataV2 pending = ClientBatchItemResult.parseMetadataV2(new Object[] {
            new long[] {0x29, 2, 0, 0, 0, 0}, null
        });
        assertTrue(pending.outcomeKind().isPending());
        assertEquals(0x29, pending.operation());
        assertTrue(ClientBatchItemResult.parseJoinSplitKeyResponse(
                new Object[] {new long[] {0}, null, new long[] {0}}).isEmpty());
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                .parseJoinSplitKeyResponse(new Object[] {new long[] {1}, null, new long[] {0}}));
    }

    @Test
    void absentResponseWithUnexpectedFieldsReleasesTransferredTtlv() {
        try (TtlvStructure source = responseTtlv(CodecLimits.defaults())) {
            long handle = source.nativeHandleOwner().transfer();
            Object[] identifier = new Object[] {
                new long[] {1, 1, 0, 0}, "unexpected-id".getBytes(StandardCharsets.UTF_8)
            };

            assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                    .parseJoinSplitKeyResponse(new Object[] {new long[] {0}, identifier, new long[] {handle}}));
            assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.ttlvStructureView(handle));
        }
    }

    @Test
    void successfulResponseWithoutIdentifierReleasesTransferredTtlv() {
        try (TtlvStructure source = responseTtlv(CodecLimits.defaults())) {
            long handle = source.nativeHandleOwner().transfer();

            assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                    .parseJoinSplitKeyResponse(new Object[] {new long[] {1}, null, new long[] {handle}}));
            assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.ttlvStructureView(handle));
        }
    }

    @Test
    void malformedIdentifierAfterTtlvMaterializationReleasesTransferredTtlv() {
        try (TtlvStructure source = responseTtlv(CodecLimits.defaults())) {
            long handle = source.nativeHandleOwner().transfer();
            Object[] malformedIdentifier = new Object[] {
                new long[] {1, 1, 0, 0}, new byte[] {(byte) 0xc3, 0x28}
            };

            assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                    .parseJoinSplitKeyResponse(new Object[] {
                        new long[] {1}, malformedIdentifier, new long[] {handle}
                    }));
            assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.ttlvStructureView(handle));
        }
    }

    private static TtlvStructure responseTtlv(CodecLimits limits) {
        TtlvStructure payload = TtlvStructure.create();
        payload = append(payload, 0x0042_0094,
                TtlvValue.TextString("private-split-key-id".getBytes(StandardCharsets.UTF_8), limits), limits);
        payload = append(payload, 0x0054_0001,
                TtlvValue.ByteString(new byte[] {(byte) 0xde, (byte) 0xad, 0, (byte) 0xff}, limits), limits);
        TtlvStructure response = TtlvStructure.create();
        response = append(response, 0x0042_005c, TtlvValue.Enumeration(0x29), limits);
        response = append(response, 0x0042_0093,
                TtlvValue.ByteString("batch-item".getBytes(StandardCharsets.UTF_8), limits), limits);
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
