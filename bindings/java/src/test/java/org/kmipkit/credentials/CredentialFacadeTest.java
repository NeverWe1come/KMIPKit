package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
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
import org.junit.jupiter.api.Test;

final class CredentialFacadeTest {
    private static final int CREDENTIAL_TYPE_TAG = 0x0042_0024;
    private static final int CREDENTIAL_VALUE_TAG = 0x0042_0025;

    @Test
    void credentialPreservesUnknownTypeAndMovesTheOriginalTtlv() throws Exception {
        Class<?> facade = Class.forName("org.kmipkit.credentials.Credential");
        Method fromTtlv = facade.getMethod("fromTtlv", TtlvStructure.class);
        Method credentialType = facade.getMethod("credentialType");
        Method credentialTypeRaw = facade.getMethod("credentialTypeRaw");
        Method withTtlv = facade.getMethod("withTtlv", Function.class);
        Method intoTtlv = facade.getMethod("intoTtlv");

        long rawType = 0xf123_4567L;
        TtlvStructure source = credentialStructure(rawType);
        Object credential = fromTtlv.invoke(null, source);
        assertTrue(throwsInvalidInput(source::view), "fromTtlv must consume the input handle");
        assertEquals("Credential[redacted]", credential.toString());
        assertEquals(rawType, credentialTypeRaw.invoke(credential));
        assertEquals(rawType, ((CredentialType) credentialType.invoke(credential)).raw());

        AtomicReference<TtlvStructureView> borrowed = new AtomicReference<>();
        assertEquals(2L, withTtlv.invoke(credential,
                (Function<TtlvStructureView, Long>) view -> {
                    borrowed.set(view);
                    return readTags(view);
                }));
        assertThrows(InvalidInputException.class, () -> borrowed.get().itemCount());

        TtlvStructure restored = (TtlvStructure) intoTtlv.invoke(credential);
        try (restored; TtlvStructureView view = restored.view()) {
            assertEquals(2L, readTags(view));
        }
        ((AutoCloseable) credential).close();
    }

    @Test
    void credentialCannotBeConsumedInsideItsScopedView() throws Exception {
        Class<?> facade = Class.forName("org.kmipkit.credentials.Credential");
        Method fromTtlv = facade.getMethod("fromTtlv", TtlvStructure.class);
        Method withTtlv = facade.getMethod("withTtlv", Function.class);
        Method intoTtlv = facade.getMethod("intoTtlv");
        Object credential = fromTtlv.invoke(null, credentialStructure(1));

        withTtlv.invoke(credential, (Function<TtlvStructureView, Long>) view -> {
            InvocationTargetException error = assertThrows(
                    InvocationTargetException.class, () -> intoTtlv.invoke(credential));
            assertInstanceOf(InvalidInputException.class, error.getCause());
            assertEquals(2L, view.itemCount());
            return null;
        });

        TtlvStructure restored = (TtlvStructure) intoTtlv.invoke(credential);
        try (restored; TtlvStructureView view = restored.view()) {
            assertEquals(2L, readTags(view));
        }
        ((AutoCloseable) credential).close();
    }

    private static boolean throwsInvalidInput(ThrowingSupplier action) {
        try {
            action.get();
            return false;
        } catch (InvalidInputException expected) {
            return true;
        }
    }

    private static long readTags(TtlvStructureView view) {
        assertEquals(2L, view.itemCount());
        int first = readTag(view, 0);
        int second = readTag(view, 1);
        assertEquals(CREDENTIAL_TYPE_TAG, first);
        assertEquals(CREDENTIAL_VALUE_TAG, second);
        return view.itemCount();
    }

    private static int readTag(TtlvStructureView view, long index) {
        try (TtlvItemView item = view.itemAt(index); Tag tag = item.tag()) {
            return tag.raw();
        }
    }

    private static TtlvStructure credentialStructure(long rawType) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        try (RawTag raw = RawTag.fromRaw(CREDENTIAL_TYPE_TAG);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Enumeration(rawType)) {
            TtlvItem item = TtlvItem.create(tag, value, limits);
            structure = TtlvStructure.withItem(structure, item, limits);
        }
        TtlvStructure valueStructure = TtlvStructure.create();
        try (RawTag raw = RawTag.fromRaw(CREDENTIAL_VALUE_TAG);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.structure(valueStructure, limits)) {
            TtlvItem item = TtlvItem.create(tag, value, limits);
            structure = TtlvStructure.withItem(structure, item, limits);
        }
        return structure;
    }

    @FunctionalInterface
    private interface ThrowingSupplier {
        Object get();
    }
}
