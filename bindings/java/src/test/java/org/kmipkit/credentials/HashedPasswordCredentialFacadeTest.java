package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.OptionalLong;
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

final class HashedPasswordCredentialFacadeTest {
    private static final int USERNAME_TAG = 0x0042_0099;
    private static final int TIMESTAMP_TAG = 0x0042_0092;
    private static final int HASHING_ALGORITHM_TAG = 0x0042_0038;
    private static final int HASHED_PASSWORD_TAG = 0x0042_0157;
    private static final long UNKNOWN_HASHING_ALGORITHM = 0xe123_4567L;
    private static final byte[] HASHED_PASSWORD_BYTES =
            "hashed-password-sentinel".getBytes(StandardCharsets.UTF_8);

    @Test
    void preservesUnknownAlgorithmAndHashedPasswordTtlv() {
        try (HashedPasswordCredential credential = HashedPasswordCredential.fromTtlv(
                hashedPasswordStructure(true))) {
            assertEquals(OptionalLong.of(UNKNOWN_HASHING_ALGORITHM),
                    credential.hashingAlgorithmRaw());
            assertEquals(UNKNOWN_HASHING_ALGORITHM,
                    credential.effectiveHashingAlgorithmRaw());
            assertFalse(credential.toString().contains("hashed-password-sentinel"));
            AtomicReference<TtlvStructureView> borrowed = new AtomicReference<>();

            assertEquals(List.of(USERNAME_TAG, TIMESTAMP_TAG,
                            HASHING_ALGORITHM_TAG, HASHED_PASSWORD_TAG),
                    credential.withTtlv(view -> {
                        borrowed.set(view);
                        assertThrows(InvalidInputException.class, credential::intoTtlv);
                        assertTrue(hashedPasswordMatches(view));
                        return readTags(view);
                    }));
            assertThrows(InvalidInputException.class, borrowed.get()::itemCount);

            TtlvStructure restored = credential.intoTtlv();
            try (TtlvStructureView view = restored.view()) {
                assertEquals(List.of(USERNAME_TAG, TIMESTAMP_TAG,
                                HASHING_ALGORITHM_TAG, HASHED_PASSWORD_TAG), readTags(view));
                assertTrue(hashedPasswordMatches(view));
            }
            restored.close();
        }
    }

    @Test
    void preservesAbsentAlgorithmAndReportsSha256Default() {
        try (HashedPasswordCredential credential = HashedPasswordCredential.fromTtlv(
                hashedPasswordStructure(false))) {
            assertEquals(OptionalLong.empty(), credential.hashingAlgorithmRaw());
            assertEquals(6, credential.effectiveHashingAlgorithmRaw());
            TtlvStructure restored = credential.intoTtlv();
            try (TtlvStructureView view = restored.view()) {
                assertEquals(List.of(USERNAME_TAG, TIMESTAMP_TAG, HASHED_PASSWORD_TAG),
                        readTags(view));
            }
            restored.close();
        }
    }

    @Test
    void requiresHashedPasswordAndRedactsValidationErrors() {
        CredentialValidationException error = assertThrows(
                CredentialValidationException.class,
                () -> HashedPasswordCredential.fromTtlv(hashedPasswordStructureMissingHash()));

        assertEquals(CredentialValidationErrorKind.MISSING_FIELD, error.kind());
        assertFalse(error.getMessage().contains("alice"));
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

    private static boolean hashedPasswordMatches(TtlvStructureView view) {
        long index = view.itemCount() == 3 ? 2 : 3;
        try (TtlvItemView item = view.itemAt(index); TtlvValueView value = item.value()) {
            if (value.itemType() != org.kmipkit.ttlv.TtlvItemType.ByteString
                    || value.byteLength() != HASHED_PASSWORD_BYTES.length) return false;
            for (int offset = 0; offset < HASHED_PASSWORD_BYTES.length; offset++) {
                if ((byte) value.byteAt(offset) != HASHED_PASSWORD_BYTES[offset]) return false;
            }
            return true;
        }
    }

    private static TtlvStructure hashedPasswordStructure(boolean includeAlgorithm) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        structure = appendText(structure, USERNAME_TAG, "alice", limits);
        try (RawTag raw = RawTag.fromRaw(TIMESTAMP_TAG);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.DateTimeExtended(1_800_000_000L)) {
            structure = TtlvStructure.withItem(structure, TtlvItem.create(tag, value, limits), limits);
        }
        if (includeAlgorithm) {
            try (RawTag raw = RawTag.fromRaw(HASHING_ALGORITHM_TAG);
                    Tag tag = raw.checked();
                    TtlvValue value = TtlvValue.Enumeration(UNKNOWN_HASHING_ALGORITHM)) {
                structure = TtlvStructure.withItem(structure,
                        TtlvItem.create(tag, value, limits), limits);
            }
        }
        try (RawTag raw = RawTag.fromRaw(HASHED_PASSWORD_TAG);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.ByteString(HASHED_PASSWORD_BYTES, limits)) {
            structure = TtlvStructure.withItem(structure, TtlvItem.create(tag, value, limits), limits);
        }
        return structure;
    }

    private static TtlvStructure hashedPasswordStructureMissingHash() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        structure = appendText(structure, USERNAME_TAG, "alice", limits);
        try (RawTag raw = RawTag.fromRaw(TIMESTAMP_TAG);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.DateTimeExtended(1_800_000_000L)) {
            structure = TtlvStructure.withItem(structure, TtlvItem.create(tag, value, limits), limits);
        }
        return structure;
    }

    private static TtlvStructure appendText(
            TtlvStructure structure, int rawTag, String text, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.TextString(text.getBytes(StandardCharsets.UTF_8), limits)) {
            return TtlvStructure.withItem(structure, TtlvItem.create(tag, value, limits), limits);
        }
    }
}
