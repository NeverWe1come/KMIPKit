package org.kmipkit.message;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.KmipException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;

final class MessageTtlvAccessCoverageTest {
    private static final CodecLimits LIMITS = CodecLimits.defaults();
    private static final int ENUM_TAG = 0x0042_0001;
    private static final int BYTES_TAG = 0x0042_0002;
    private static final int TEXT_TAG = 0x0042_0003;
    private static final int STRUCTURE_TAG = 0x0042_0004;
    private static final int DATE_TIME_TAG = 0x0042_0005;

    @Test
    void messageNativeValidatesNullAndMalformedAccessResults() {
        assertThrows(InvalidInputException.class, () -> MessageNative.parse(null, true));
        assertThrows(KmipException.class, () -> MessageNative.optionalRaw(null));
        assertThrows(KmipException.class, () -> MessageNative.optionalRaw(new long[] {0}));
        assertTrue(MessageNative.optionalRaw(new long[] {0, 7}).isEmpty());
        assertEquals(7, MessageNative.optionalRaw(new long[] {1, 7}).orElseThrow());
    }

    @Test
    void typedAccessorsFilterValuesAndScopedCallbacksCloseAndClearTheirCopies() {
        TtlvStructure nested = TtlvStructure.withItem(TtlvStructure.create(),
                item(ENUM_TAG, TtlvValue.Enumeration(9)), LIMITS);
        TtlvStructure structure = TtlvStructure.create();
        structure = TtlvStructure.withItem(structure, item(ENUM_TAG, TtlvValue.Enumeration(9)), LIMITS);
        structure = TtlvStructure.withItem(structure, item(BYTES_TAG,
                TtlvValue.ByteString(new byte[] {1, 2, 3}, LIMITS)), LIMITS);
        structure = TtlvStructure.withItem(structure, item(TEXT_TAG,
                TtlvValue.TextString("text".getBytes(StandardCharsets.UTF_8), LIMITS)), LIMITS);
        structure = TtlvStructure.withItem(structure, item(STRUCTURE_TAG,
                TtlvValue.structure(nested, LIMITS)), LIMITS);
        structure = TtlvStructure.withItem(structure, item(DATE_TIME_TAG,
                TtlvValue.DateTime(1_699_999_999)), LIMITS);
        final TtlvStructure root = structure;
        try (root) {
            assertEquals(9, MessageTtlvAccess.enumeration(root, ENUM_TAG).orElseThrow());
            assertEquals(3, MessageTtlvAccess.byteString(root, BYTES_TAG).orElseThrow().length);
            assertEquals("text", MessageTtlvAccess.textString(root, TEXT_TAG).orElseThrow());
            assertEquals(1_699_999_999L,
                    MessageTtlvAccess.dateTime(root, DATE_TIME_TAG).orElseThrow());
            assertFalse(MessageTtlvAccess.booleanValue(root, ENUM_TAG).isPresent());
            assertTrue(MessageTtlvAccess.enumeration(root, 0x0042_00ff).isEmpty());
            assertEquals(java.util.List.of(9L),
                    MessageTtlvAccess.repeatedEnumerations(root, ENUM_TAG));

            AtomicReference<TtlvStructureView> borrowedStructure = new AtomicReference<>();
            assertEquals(1, MessageTtlvAccess.withStructure(root, STRUCTURE_TAG, view -> {
                borrowedStructure.set(view);
                return view.itemCount();
            }).orElseThrow());
            assertThrows(InvalidInputException.class, borrowedStructure.get()::itemCount);

            AtomicReference<byte[]> borrowedBytes = new AtomicReference<>();
            assertEquals(3, MessageTtlvAccess.withByteString(root, BYTES_TAG, bytes -> {
                borrowedBytes.set(bytes);
                return bytes.length;
            }).orElseThrow());
            assertArrayEquals(new byte[3], borrowedBytes.get());
            assertTrue(MessageTtlvAccess.withStructure(root, ENUM_TAG, view -> view.itemCount()).isEmpty());
            assertTrue(MessageTtlvAccess.withByteString(root, ENUM_TAG, bytes -> bytes.length).isEmpty());
            assertThrows(InvalidInputException.class,
                    () -> MessageTtlvAccess.withStructure(root, ENUM_TAG, null));
            assertThrows(InvalidInputException.class,
                    () -> MessageTtlvAccess.withByteString(root, ENUM_TAG, null));
        }
    }

    private static TtlvItem item(int rawTag, TtlvValue value) {
        Tag tag = RawTag.fromRaw(rawTag).checked();
        return TtlvItem.create(tag, value, LIMITS);
    }
}
