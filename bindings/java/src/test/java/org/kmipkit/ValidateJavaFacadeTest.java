package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.AbstractList;
import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;

/** Validate facade coverage derived from OASIS KMIP 2.1 §6.1.57, Table 340. */
final class ValidateJavaFacadeTest {
    @Test
    void builderCopiesNonemptyCertificateArrayAndPreservesCallerOwnership() {
        try (TtlvStructure first = certificate(new byte[] {1, 2, 3});
                TtlvStructure second = certificate(new byte[] {4, 5, 6});
                ClientBatchItem item = ClientBatchItem.validate(List.of(first, second), List.of(), null);
                TtlvStructureView firstView = first.view();
                TtlvStructureView secondView = second.view()) {
            assertNotNull(item);
            org.junit.jupiter.api.Assertions.assertEquals(2, firstView.itemCount());
            org.junit.jupiter.api.Assertions.assertEquals(2, secondView.itemCount());
        }
    }

    @Test
    void builderAcceptsIdentifierOnlyChainAndPreservesOptionalDate() {
        try (ClientBatchItem item = ClientBatchItem.validate(
                List.of(), List.of(UniqueIdentifier.textString("chain-id")), Long.MIN_VALUE)) {
            assertNotNull(item);
        }
    }

    @Test
    void builderPreservesOmittedDateAndEmptyChainWithoutSynthesizingLocalTime() {
        try (ClientBatchItem item = ClientBatchItem.validate(List.of(), List.of(), null)) {
            assertNotNull(item);
        }
    }

    @Test
    void builderRejectsMissingLists() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.validate(null, List.of(), null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.validate(List.of(), null, null));
    }

    @Test
    void builderRejectsNullChainEntriesAndAggregateIdentifierBytes() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.validate(Arrays.asList((TtlvStructure) null), List.of(), null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.validate(List.of(), Arrays.asList((UniqueIdentifier) null), null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.validate(
                        List.of(), List.of(), null, Arrays.asList((TtlvItem) null)));

        UniqueIdentifier first = UniqueIdentifier.textString("a".repeat(8 * 1024 * 1024));
        UniqueIdentifier second = UniqueIdentifier.textString("b".repeat(8 * 1024 * 1024 + 1));
        assertThrows(ResourceLimitException.class,
                () -> ClientBatchItem.validate(List.of(), List.of(first, second), null));
    }

    @Test
    void builderRejectsValidateIteratorThatExceedsItsReportedSize() {
        List<UniqueIdentifier> inconsistent = new AbstractList<>() {
            @Override
            public UniqueIdentifier get(int index) {
                throw new AssertionError("snapshot must use the provided iterator");
            }

            @Override
            public int size() {
                return 0;
            }

            @Override
            public java.util.Iterator<UniqueIdentifier> iterator() {
                return new java.util.Iterator<>() {
                    private int emitted;

                    @Override
                    public boolean hasNext() {
                        return emitted <= 100_000;
                    }

                    @Override
                    public UniqueIdentifier next() {
                        emitted++;
                        return UniqueIdentifier.integer(7);
                    }
                };
            }
        };

        assertThrows(ResourceLimitException.class,
                () -> ClientBatchItem.validate(List.of(), inconsistent, null));
    }

    @Test
    void builderCopiesUnknownPayloadExtensionsAndLeavesInputOwned() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag rawTag = RawTag.fromRaw(0x0054_1234);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.TextString(
                        "validate-extension-sentinel".getBytes(java.nio.charset.StandardCharsets.UTF_8),
                        limits);
                TtlvItem extension = TtlvItem.create(
                        tag,
                        value,
                        limits);
                ClientBatchItem item = ClientBatchItem.validate(
                        List.of(), List.of(), null, List.of(extension));
                TtlvStructure retained = TtlvStructure.withItem(
                        TtlvStructure.create(), extension, limits);
                TtlvStructureView view = retained.view()) {
            assertNotNull(item);
            org.junit.jupiter.api.Assertions.assertEquals(1, view.itemCount());
        }
    }

    @Test
    void builderRejectsOversizedExtensionListBeforeCopyingIt() {
        List<TtlvItem> oversized = new AbstractList<>() {
            @Override
            public TtlvItem get(int index) {
                throw new AssertionError("oversized list entries must not be copied");
            }

            @Override
            public int size() {
                return 100_001;
            }
        };

        assertThrows(ResourceLimitException.class,
                () -> ClientBatchItem.validate(List.of(), List.of(), null, oversized));
    }

    @Test
    void nativeBridgeRejectsMismatchedIdentifierArraysAndNoncanonicalAbsentDate() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemValidate(
                        null, new int[] {1}, new byte[0][], new long[] {0}, new long[0], false, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemValidate(
                        null, new int[0], new byte[0][], new long[0], new long[0], false, 1));
    }

    @Test
    void nativeBridgeRejectsOversizedAggregateItemCountBeforeReadingEntries() {
        int certificateCount = 40_000;
        int identifierCount = 40_000;
        int payloadExtensionCount = 20_001;

        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.clientBatchItemValidate(
                        new long[certificateCount],
                        new int[identifierCount],
                        new byte[identifierCount][],
                        new long[identifierCount],
                        new long[payloadExtensionCount],
                        false,
                        0));
    }

    @Test
    void nativeBridgeRejectsMalformedAndOverLimitIdentifierRecords() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemValidate(
                        null, new int[] {1}, new byte[][] {null}, new long[] {0}, new long[0], false, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemValidate(
                        null, new int[] {2}, new byte[][] {new byte[] {1}}, new long[] {1},
                        new long[0], false, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemValidate(
                        null, new int[] {2}, new byte[][] {null}, new long[] {0x1_0000_0000L},
                        new long[0], false, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemValidate(
                        null, new int[] {3}, new byte[][] {null}, new long[] {(long) Integer.MAX_VALUE + 1},
                        new long[0], false, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemValidate(
                        null, new int[] {99}, new byte[][] {null}, new long[] {0}, new long[0], false, 0));
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.clientBatchItemValidate(
                        null,
                        new int[] {1},
                        new byte[][] {new byte[16 * 1024 * 1024 + 1]},
                        new long[] {0},
                        new long[0],
                        false,
                        0));
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.clientBatchItemValidate(
                        null,
                        new int[] {1, 1, 1},
                        new byte[][] {
                            new byte[8 * 1024 * 1024],
                            new byte[8 * 1024 * 1024],
                            new byte[] {1}
                        },
                        new long[] {0, 0, 0},
                        new long[0],
                        false,
                        0));
    }

    @Test
    void nativeBridgeAcceptsOpenEnumerationAndSignedIntegerIdentifiers() {
        long item = NativeExtensionRegistry.clientBatchItemValidate(
                null,
                new int[] {2, 3},
                new byte[][] {null, null},
                new long[] {0xffff_ffffL, -17},
                new long[0],
                false,
                0);

        try {
            assertNotEquals(0, item);
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, item);
        }
    }

    @Test
    void nativeBridgeRejectsInvalidPayloadExtensionHandle() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemValidate(
                        null, new int[0], new byte[0][], new long[0], new long[] {0}, false, 0));
    }

    private static TtlvStructure certificate(byte[] value) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        try {
            structure = append(structure, 0x0042_001d, TtlvValue.Enumeration(1), limits);
            structure = append(
                    structure,
                    0x0042_001e,
                    TtlvValue.ByteString(value, limits),
                    limits);
            return structure;
        } catch (RuntimeException exception) {
            structure.close();
            throw exception;
        }
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTagValue, TtlvValue value, CodecLimits limits) {
        try (RawTag rawTag = RawTag.fromRaw(rawTagValue);
                Tag tag = rawTag.checked();
                value;
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, item, limits);
        }
    }
}
