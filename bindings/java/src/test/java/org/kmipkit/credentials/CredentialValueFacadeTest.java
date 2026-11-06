package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;
import org.junit.jupiter.api.Test;

final class CredentialValueFacadeTest {
    private static final int USERNAME_TAG = 0x0042_0099;
    private static final int ONE_TIME_PASSWORD_TAG = 0x0042_0156;
    private static final int OPAQUE_CHILD_TAG = 0x0054_0001;

    @Test
    // KMIP Specification v2.1 §9.11 Table 414 and §11.11 Table 442.
    void preservesKnownExtensionAndUnknownCredentialValueDiscriminators() {
        List<CredentialCase> cases = List.of(
                new CredentialCase(4, List.of(USERNAME_TAG, ONE_TIME_PASSWORD_TAG), null),
                new CredentialCase(0x8000_4321L, List.of(OPAQUE_CHILD_TAG), 7),
                new CredentialCase(0xf123_4567L, List.of(OPAQUE_CHILD_TAG), 7));

        for (CredentialCase credentialCase : cases) {
            try (CredentialValue value = CredentialValue.fromTtlv(
                    credentialCase.rawType(), credentialValueStructure(credentialCase.rawType()))) {
                assertEquals(credentialCase.rawType(), value.credentialTypeRaw());
                assertFalse(value.toString().contains("credential-value-secret"));
                List<TtlvStructureView> borrowed = new ArrayList<>();
                assertEquals(credentialCase.expectedTags(), value.withTtlv(view -> {
                    borrowed.add(view);
                    assertThrows(InvalidInputException.class, value::intoTtlv);
                    if (credentialCase.expectedInteger() != null) {
                        assertEquals(credentialCase.expectedInteger().intValue(), readFirstInteger(view));
                    }
                    return readTags(view);
                }));
                assertThrows(InvalidInputException.class, borrowed.get(0)::itemCount);

                TtlvStructure restored = value.intoTtlv();
                try (TtlvStructureView view = restored.view()) {
                    assertEquals(credentialCase.expectedTags(), readTags(view));
                    if (credentialCase.expectedInteger() != null) {
                        assertEquals(credentialCase.expectedInteger().intValue(), readFirstInteger(view));
                    }
                }
                restored.close();
            }
        }
    }

    @Test
    void failedTransferWithRetainedChildViewKeepsCredentialValueUsable() {
        long rawType = 0x8000_4321L;
        try (CredentialValue value = CredentialValue.fromTtlv(
                rawType, credentialValueStructure(rawType))) {
            try (TtlvItemView retained = value.withTtlv(view -> view.itemAt(0))) {
                assertEquals(TtlvItemType.Integer, retained.itemType());
                assertThrows(InvalidInputException.class, value::intoTtlv);
                assertEquals(rawType, value.credentialTypeRaw());
            }

            try (TtlvStructure restored = value.intoTtlv()) {
                try (TtlvStructureView restoredView = restored.view()) {
                    assertEquals(List.of(OPAQUE_CHILD_TAG), readTags(restoredView));
                }
            }
        }
    }

    @Test
    // KMIP Specification v2.1 §9.11 Table 414 requires Username and One Time Password.
    void rejectsMissingUsernameWithPayloadFreeValidation() {
        CredentialValidationException error = assertThrows(
                CredentialValidationException.class,
                () -> CredentialValue.fromTtlv(4, credentialValueStructureMissingUsername()));
        assertEquals(CredentialValidationErrorKind.MISSING_FIELD, error.kind());
        assertFalse(error.getMessage().contains("credential-value-secret"));
    }

    private static List<Integer> readTags(TtlvStructureView view) {
        List<Integer> result = new ArrayList<>();
        for (long index = 0; index < view.itemCount(); index++) {
            try (TtlvItemView item = view.itemAt(index); Tag tag = item.tag()) {
                result.add(tag.raw());
            }
        }
        return List.copyOf(result);
    }

    private static int readFirstInteger(TtlvStructureView view) {
        try (TtlvItemView item = view.itemAt(0); TtlvValueView value = item.value()) {
            assertEquals(TtlvItemType.Integer, value.itemType());
            return value.integer();
        }
    }

    private static TtlvStructure credentialValueStructure(long rawType) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        if (rawType == 4) {
            structure = appendText(structure, USERNAME_TAG, "alice", limits);
            structure = appendText(structure, ONE_TIME_PASSWORD_TAG,
                    "credential-value-secret", limits);
        } else {
            structure = appendInteger(structure, OPAQUE_CHILD_TAG, limits);
        }
        return structure;
    }

    private static TtlvStructure credentialValueStructureMissingUsername() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = appendText(TtlvStructure.create(), ONE_TIME_PASSWORD_TAG,
                "credential-value-secret", limits);
        return structure;
    }

    private static TtlvStructure appendText(
            TtlvStructure structure, int rawTag, String text, CodecLimits limits) {
        return append(structure, rawTag, TtlvValue.TextString(
                text.getBytes(StandardCharsets.UTF_8), limits), limits);
    }

    private static TtlvStructure appendInteger(
            TtlvStructure structure, int rawTag, CodecLimits limits) {
        return append(structure, rawTag, TtlvValue.Integer(7), limits);
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked(); value) {
            return TtlvStructure.withItem(structure, TtlvItem.create(tag, value, limits), limits);
        }
    }

    private record CredentialCase(
            long rawType, List<Integer> expectedTags, Integer expectedInteger) {}
}
