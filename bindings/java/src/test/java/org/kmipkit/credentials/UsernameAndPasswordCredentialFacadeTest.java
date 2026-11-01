package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.concurrent.atomic.AtomicReference;
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

final class UsernameAndPasswordCredentialFacadeTest {
    private static final int USERNAME_TAG = 0x0042_0099;
    private static final int PASSWORD_TAG = 0x0042_00A1;
    private static final int EXTENSION_CHILD_TAG = 0x0054_0001;

    @Test
    void usernameAndPasswordCredentialPreservesTable411AndScopedTtlv() {
        UsernameAndPasswordCredential credential = UsernameAndPasswordCredential.fromTtlv(
                usernamePasswordStructure(true));
        AtomicReference<TtlvStructureView> borrowed = new AtomicReference<>();

        assertFalse(credential.toString().contains("sensitive-sentinel"));
        assertEquals(List.of(USERNAME_TAG, PASSWORD_TAG, EXTENSION_CHILD_TAG),
                credential.withTtlv(view -> {
                    borrowed.set(view);
                    assertThrows(InvalidInputException.class, credential::intoTtlv);
                    assertTrue(valuesMatch(view));
                    return readTags(view);
                }));
        assertThrows(InvalidInputException.class, borrowed.get()::itemCount);

        TtlvStructure restored = credential.intoTtlv();
        try (TtlvStructureView view = restored.view()) {
            assertEquals(List.of(USERNAME_TAG, PASSWORD_TAG, EXTENSION_CHILD_TAG), readTags(view));
            assertTrue(valuesMatch(view));
        }
        restored.close();
    }

    @Test
    void usernameAndPasswordCredentialRequiresUsernameAndRedactsErrors() {
        CredentialValidationException error = assertThrows(
                CredentialValidationException.class,
                () -> UsernameAndPasswordCredential.fromTtlv(usernamePasswordStructure(false)));

        assertEquals(CredentialValidationErrorKind.MISSING_FIELD, error.kind());
        assertFalse(error.getMessage().contains("sensitive-sentinel"));
    }

    private static List<Integer> readTags(TtlvStructureView view) {
        java.util.ArrayList<Integer> tags = new java.util.ArrayList<>();
        for (long index = 0; index < view.itemCount(); index++) {
            try (TtlvItemView item = view.itemAt(index); Tag tag = item.tag()) {
                tags.add(tag.raw());
            }
        }
        return List.copyOf(tags);
    }

    private static boolean valuesMatch(TtlvStructureView view) {
        byte[][] expected = {
            "alice".getBytes(StandardCharsets.UTF_8),
            "sensitive-sentinel".getBytes(StandardCharsets.UTF_8)
        };
        for (int index = 0; index < expected.length; index++) {
            try (TtlvItemView item = view.itemAt(index); TtlvValueView value = item.value()) {
                if (value.itemType() != org.kmipkit.ttlv.TtlvItemType.TextString
                        || value.byteLength() != expected[index].length) return false;
                for (int offset = 0; offset < expected[index].length; offset++) {
                    if ((byte) value.byteAt(offset) != expected[index][offset]) return false;
                }
            }
        }
        return true;
    }

    private static TtlvStructure usernamePasswordStructure(boolean includeUsername) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        if (includeUsername) structure = append(structure, USERNAME_TAG, "alice", limits);
        structure = append(structure, PASSWORD_TAG, "sensitive-sentinel", limits);
        try (RawTag raw = RawTag.fromRaw(EXTENSION_CHILD_TAG);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Integer(7)) {
            TtlvItem item = TtlvItem.create(tag, value, limits);
            structure = TtlvStructure.withItem(structure, item, limits);
        }
        return structure;
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, String text, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.TextString(text.getBytes(StandardCharsets.UTF_8), limits)) {
            TtlvItem item = TtlvItem.create(tag, value, limits);
            return TtlvStructure.withItem(structure, item, limits);
        }
    }
}
