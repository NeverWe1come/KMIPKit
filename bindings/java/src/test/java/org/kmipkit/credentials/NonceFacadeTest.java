package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;
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

final class NonceFacadeTest {
    private static final int NONCE_ID_TAG = 0x0042_00C9;
    private static final int NONCE_VALUE_TAG = 0x0042_00CA;
    private static final byte[] NONCE_ID = {(byte) 0x10, 0x00, (byte) 0xff};
    private static final byte[] NONCE_VALUE = {(byte) 0x80, 0x01, 0x7f, 0x00};

    @Test
    void nonceCallbackPreservesBytesAndTtlvOwnershipMovesWithoutCopy() throws Exception {
        Class<?> facade = assertDoesNotThrow(
                () -> Class.forName("org.kmipkit.credentials.Nonce"),
                "the Java facade must expose the canonical server-sourced Nonce model");
        Method fromTtlv = facade.getMethod("fromTtlv", TtlvStructure.class);
        Method withTtlv = facade.getMethod("withTtlv", Function.class);
        Method intoTtlv = facade.getMethod("intoTtlv");

        TtlvStructure structure = nonceStructure();
        Object nonce = fromTtlv.invoke(null, structure);
        assertEquals("Nonce[redacted]", nonce.toString());
        assertTrue(!nonce.toString().contains(hex(NONCE_VALUE)));
        assertTrue(
                throwsInvalidInput(structure::view),
                "fromTtlv must consume the original Structure handle");
        AtomicReference<TtlvStructureView> borrowed = new AtomicReference<>();
        Object itemCount = withTtlv.invoke(nonce, (Function<TtlvStructureView, Long>) view -> {
            borrowed.set(view);
            return readNonce(view);
        });
        assertEquals(2L, itemCount);
        assertThrows(InvalidInputException.class, () -> borrowed.get().itemCount());

        TtlvStructure restored = (TtlvStructure) intoTtlv.invoke(nonce);
        try (restored) {
            try (TtlvStructureView view = restored.view()) {
                assertEquals(2L, readNonce(view));
            }
        }
        ((AutoCloseable) nonce).close();
    }

    @Test
    void nonceRejectsMissingRequiredValueWithoutDisclosingNonceBytes() throws Exception {
        Class<?> facade = assertDoesNotThrow(
                () -> Class.forName("org.kmipkit.credentials.Nonce"));
        Method fromTtlv = facade.getMethod("fromTtlv", TtlvStructure.class);
        TtlvStructure malformed = TtlvStructure.create();
        InvocationTargetException error = assertThrows(
                InvocationTargetException.class, () -> fromTtlv.invoke(null, malformed));
        assertInstanceOf(InvalidInputException.class, error.getCause());
        assertTrue(!error.getCause().getMessage().contains(hex(NONCE_VALUE)));
    }

    @Test
    void failedNonceConversionRetainsSourceWhileItsViewIsLive() {
        TtlvStructure source = nonceStructure();
        try (TtlvStructureView view = source.view()) {
            assertThrows(InvalidInputException.class, () -> Nonce.fromTtlv(source));
            assertEquals(2L, readNonce(view));
        }

        try (Nonce nonce = Nonce.fromTtlv(source)) {
            assertEquals(2L, nonce.withTtlv(NonceFacadeTest::readNonce));
        }
    }

    @Test
    void nonceRemainsUsableWhenConsumptionIsAttemptedInsideItsScopedView() {
        TtlvStructure structure = nonceStructure();
        Nonce nonce = Nonce.fromTtlv(structure);

        nonce.withTtlv(view -> {
            assertThrows(InvalidInputException.class, nonce::intoTtlv);
            assertEquals(2L, readNonce(view));
            return null;
        });

        assertEquals(2L, nonce.withTtlv(NonceFacadeTest::readNonce));
        try (TtlvStructure restored = nonce.intoTtlv()) {
            try (TtlvStructureView view = restored.view()) {
                assertEquals(2L, readNonce(view));
            }
        }
        nonce.close();
    }

    private static boolean throwsInvalidInput(ThrowingSupplier action) {
        try {
            action.get();
            return false;
        } catch (InvalidInputException expected) {
            return true;
        }
    }

    private static long readNonce(TtlvStructureView view) {
        assertEquals(2L, view.itemCount());
        List<Integer> tags = new ArrayList<>();
        List<byte[]> values = new ArrayList<>();
        for (long index = 0; index < view.itemCount(); index++) {
            try (TtlvItemView item = view.itemAt(index);
                    Tag tag = item.tag();
                    TtlvValueView value = item.value()) {
                tags.add(tag.raw());
                int length = Math.toIntExact(value.byteLength());
                byte[] bytes = new byte[length];
                for (int byteIndex = 0; byteIndex < length; byteIndex++) {
                    bytes[byteIndex] = (byte) value.byteAt(byteIndex);
                }
                values.add(bytes);
            }
        }
        assertEquals(List.of(NONCE_ID_TAG, NONCE_VALUE_TAG), tags);
        assertEquals(hex(NONCE_ID), hex(values.get(0)));
        assertEquals(hex(NONCE_VALUE), hex(values.get(1)));
        return view.itemCount();
    }

    private static TtlvStructure nonceStructure() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        structure = addByteString(structure, NONCE_ID_TAG, NONCE_ID, limits);
        structure = addByteString(structure, NONCE_VALUE_TAG, NONCE_VALUE, limits);
        return structure;
    }

    private static TtlvStructure addByteString(
            TtlvStructure structure, int rawTag, byte[] bytes, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked();
                TtlvValue value = TtlvValue.ByteString(bytes, limits)) {
            TtlvItem item = TtlvItem.create(tag, value, limits);
            return TtlvStructure.withItem(structure, item, limits);
        }
    }

    private static String hex(byte[] bytes) {
        StringBuilder result = new StringBuilder(bytes.length * 2);
        for (byte value : bytes) {
            result.append(String.format("%02x", value & 0xff));
        }
        return result.toString();
    }

    @FunctionalInterface
    private interface ThrowingSupplier {
        Object get();
    }
}
