package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
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

final class DeviceCredentialFacadeTest {
    private static final int DEVICE_SERIAL_NUMBER_TAG = 0x0042_00B0;
    private static final int PASSWORD_TAG = 0x0042_00A1;
    private static final int DEVICE_IDENTIFIER_TAG = 0x0042_00A2;
    private static final int NETWORK_IDENTIFIER_TAG = 0x0042_00AB;
    private static final int MACHINE_IDENTIFIER_TAG = 0x0042_00A9;
    private static final int MEDIA_IDENTIFIER_TAG = 0x0042_00AA;
    private static final int EXTENSION_CHILD_TAG = 0x0054_0001;

    @Test
    void deviceCredentialPreservesTable412FieldsAndScopedTtlv() {
        TtlvStructure source = deviceStructure(true);
        DeviceCredential credential = DeviceCredential.fromTtlv(source);
        AtomicReference<TtlvStructureView> borrowed = new AtomicReference<>();
        List<Integer> expected = List.of(DEVICE_SERIAL_NUMBER_TAG, PASSWORD_TAG,
                DEVICE_IDENTIFIER_TAG, NETWORK_IDENTIFIER_TAG, MACHINE_IDENTIFIER_TAG,
                MEDIA_IDENTIFIER_TAG, EXTENSION_CHILD_TAG);
        assertTrue(!credential.toString().contains("sensitive-sentinel"));
        assertEquals(expected, credential.withTtlv(view -> {
            borrowed.set(view);
            assertThrows(InvalidInputException.class, credential::intoTtlv);
            assertTrue(deviceValuesMatch(view));
            return readTags(view);
        }));
        assertThrows(InvalidInputException.class, borrowed.get()::itemCount);

        TtlvStructure restored = credential.intoTtlv();
        try (TtlvStructureView view = restored.view()) {
            assertEquals(expected, readTags(view));
            assertTrue(deviceValuesMatch(view));
        }
        assertThrows(InvalidInputException.class,
                () -> credential.withTtlv(TtlvStructureView::itemCount));
        restored.close();
    }

    @Test
    void deviceCredentialRequiresOneOfTheFourIdentifierFields() {
        TtlvStructure source = deviceStructure(false);
        CredentialValidationException error = assertThrows(
                CredentialValidationException.class, () -> DeviceCredential.fromTtlv(source));
        assertEquals(CredentialValidationErrorKind.MISSING_FIELD, error.kind());
        assertFalse(error.getMessage().contains("sensitive-sentinel"));
    }

    @Test
    void unknownCredentialValidationKindsRemainRaw() {
        CredentialValidationException error = new CredentialValidationException(0x7f);

        assertEquals(0x7f, error.kind());
        assertFalse(error.getMessage().contains("sensitive-sentinel"));
    }

    private static List<Integer> readTags(TtlvStructureView view) {
        List<Integer> tags = new ArrayList<>();
        for (long index = 0; index < view.itemCount(); index++) {
            try (TtlvItemView item = view.itemAt(index); Tag tag = item.tag()) {
                tags.add(tag.raw());
            }
        }
        return List.copyOf(tags);
    }

    private static boolean deviceValuesMatch(TtlvStructureView view) {
        byte[][] expectedText = {
            "".getBytes(StandardCharsets.UTF_8),
            "sensitive-sentinel".getBytes(StandardCharsets.UTF_8),
            "device-id".getBytes(StandardCharsets.UTF_8),
            "network-id".getBytes(StandardCharsets.UTF_8),
            "machine-id".getBytes(StandardCharsets.UTF_8),
            "media-id".getBytes(StandardCharsets.UTF_8)
        };
        for (int index = 0; index < expectedText.length; index++) {
            try (TtlvItemView item = view.itemAt(index); TtlvValueView value = item.value()) {
                if (value.itemType() != org.kmipkit.ttlv.TtlvItemType.TextString
                        || value.byteLength() != expectedText[index].length) {
                    return false;
                }
                for (int offset = 0; offset < expectedText[index].length; offset++) {
                    if ((byte) value.byteAt(offset) != expectedText[index][offset]) return false;
                }
            }
        }
        try (TtlvItemView item = view.itemAt(6); TtlvValueView value = item.value()) {
            return value.itemType() == org.kmipkit.ttlv.TtlvItemType.Integer
                    && value.integer() == 7;
        }
    }

    private static TtlvStructure deviceStructure(boolean includeIdentifiers) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        if (includeIdentifiers) {
            structure = append(structure, DEVICE_SERIAL_NUMBER_TAG, "", limits);
        }
        structure = append(structure, PASSWORD_TAG, "sensitive-sentinel", limits);
        structure = append(structure, DEVICE_IDENTIFIER_TAG, "device-id", limits);
        if (includeIdentifiers) {
            structure = append(structure, NETWORK_IDENTIFIER_TAG, "network-id", limits);
            structure = append(structure, MACHINE_IDENTIFIER_TAG, "machine-id", limits);
            structure = append(structure, MEDIA_IDENTIFIER_TAG, "media-id", limits);
            try (RawTag raw = RawTag.fromRaw(EXTENSION_CHILD_TAG);
                    Tag tag = raw.checked();
                    TtlvValue value = TtlvValue.Integer(7)) {
                TtlvItem item = TtlvItem.create(tag, value, limits);
                structure = TtlvStructure.withItem(structure, item, limits);
            }
        }
        return structure;
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, String text, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.TextString(
                        text.getBytes(StandardCharsets.UTF_8), limits)) {
            TtlvItem item = TtlvItem.create(tag, value, limits);
            return TtlvStructure.withItem(structure, item, limits);
        }
    }
}
