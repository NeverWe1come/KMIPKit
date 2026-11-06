package org.kmipkit.typed;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.KeyMaterial;
import org.kmipkit.ttlv.ManagedObject;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

final class ManagedObjectParityTest {
    @Test
    void preservesUnknownObjectAndOwnedGenericTtlv() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag rawTag = RawTag.fromRaw(0x00540001);
                Tag tag = rawTag.checked();
                TtlvStructure structure = TtlvStructure.create();
                TtlvValue value = TtlvValue.structure(structure, limits);
                TtlvItem item = TtlvItem.create(tag, value, limits);
                ManagedObject managed = ManagedObject.fromItem(0x80000001, item);
                TtlvItem copy = managed.asItem();
                TtlvItemView view = copy.view();
                Tag copyTag = view.tag()) {
            assertEquals(0x80000001, managed.objectType());
            assertEquals(0x00540001, copyTag.raw());
            try (TtlvItemView sourceView = item.view(); Tag sourceTag = sourceView.tag()) {
                assertEquals(0x00540001, sourceTag.raw());
            }
        }
    }

    @Test
    void rejectsKnownObjectTypeMismatchWithSanitizedException() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag rawTag = RawTag.fromRaw(0x00540001);
                Tag tag = rawTag.checked();
                TtlvStructure structure = TtlvStructure.create();
                TtlvValue value = TtlvValue.structure(structure, limits);
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            InvalidInputException error = assertThrows(
                    InvalidInputException.class, () -> ManagedObject.fromItem(2, item));
            assertFalse(error.getMessage().contains("secret"));
        }
    }

    @Test
    void readsKeyMaterialWithoutFormattingSecretBytes() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag rawTag = RawTag.fromRaw(0x00420043);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.ByteString("secret-canary".getBytes(), limits);
                TtlvItem item = TtlvItem.create(tag, value, limits);
                KeyMaterial material = KeyMaterial.fromItem(1, item)) {
            assertEquals(1, material.formatRaw());
            assertFalse(material.toString().contains("secret-canary"));
        }
    }
}
