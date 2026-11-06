package org.kmipkit.typed;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.KeyMaterial;
import org.kmipkit.ttlv.ManagedObject;
import org.kmipkit.ttlv.KeyBlock;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.KeyValue;
import org.kmipkit.ttlv.KeyWrappingData;
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
                ManagedObject managed = ManagedObject.fromItem(0x80000001L, item);
                TtlvItemView view = managed.view();
                Tag copyTag = view.tag()) {
            assertEquals(0x80000001L, managed.objectType());
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

    @Test
    void dispatchesAllNineKnownObjectTypesAndReadsKeyStructures() {
        CodecLimits limits = CodecLimits.defaults();
        List<Integer> objectTypes = List.of(1, 2, 3, 4, 5, 7, 8, 9, 10);
        List<Integer> objectTags = List.of(
                0x00420013, 0x0042008f, 0x0042006d, 0x00420064, 0x00420089,
                0x00420085, 0x0042005b, 0x004200bd, 0x00420018);
        List<TtlvItem> objects = List.of(
                structureItem(0x00420013, limits,
                        item(0x0042001d, TtlvValue.Enumeration(1), limits),
                        item(0x0042001e, TtlvValue.ByteString(new byte[] {1}, limits), limits)),
                structureItem(0x0042008f, limits, keyBlock(limits)),
                structureItem(0x0042006d, limits, keyBlock(limits)),
                structureItem(0x00420064, limits, keyBlock(limits)),
                structureItem(0x00420089, limits,
                        item(0x0042008b, TtlvValue.Integer(2), limits),
                        item(0x00420044, TtlvValue.Integer(1), limits),
                        item(0x0042008c, TtlvValue.Integer(2), limits),
                        item(0x0042008a, TtlvValue.Enumeration(1), limits), keyBlock(limits)),
                structureItem(0x00420085, limits,
                        item(0x00420086, TtlvValue.Enumeration(1), limits), keyBlock(limits)),
                structureItem(0x0042005b, limits,
                        item(0x00420059, TtlvValue.Enumeration(1), limits),
                        item(0x0042005a, TtlvValue.ByteString(new byte[] {1}, limits), limits)),
                structureItem(0x004200bd, limits,
                        item(0x004200be, TtlvValue.Integer(7), limits), keyBlock(limits)),
                structureItem(0x00420018, limits,
                        item(0x00420019, TtlvValue.Enumeration(1), limits),
                        item(0x00420140, TtlvValue.ByteString(new byte[] {1}, limits), limits)));

        for (int index = 0; index < objects.size(); index++) {
            TtlvItem source = objects.get(index);
            try (source;
                    ManagedObject managed = ManagedObject.fromItem(objectTypes.get(index), source);
                    TtlvItemView view = managed.view();
                    Tag tag = view.tag()) {
                assertEquals(objectTags.get(index), tag.raw(), "Object Type " + objectTypes.get(index));
                try (TtlvItemView sourceView = source.view(); Tag sourceTag = sourceView.tag()) {
                    assertEquals(objectTags.get(index), sourceTag.raw());
                }
            }
        }

        TtlvItem keyValueItem = structureItem(0x00420045, limits,
                item(0x00420043,
                        TtlvValue.ByteString("secret-canary".getBytes(), limits), limits));
        try (keyValueItem; KeyValue keyValue = KeyValue.fromItem(1, keyValueItem);
                TtlvItemView keyValueView = keyValue.view();
                Tag keyValueTag = keyValueView.tag()) {
            assertEquals(0x00420045, keyValueTag.raw());
        }

        TtlvItem wrappingItem = structureItem(0x00420046, limits,
                item(0x0042009e, TtlvValue.Enumeration(1), limits),
                structureItem(0x00420036, limits,
                        item(0x00420094,
                                TtlvValue.TextString("wrapping-key".getBytes(), limits), limits)));
        try (wrappingItem; KeyWrappingData wrapping = KeyWrappingData.fromItem(wrappingItem);
                TtlvItemView wrappingView = wrapping.view();
                Tag wrappingTag = wrappingView.tag()) {
            assertEquals(0x00420046, wrappingTag.raw());
        }

        TtlvItem keyBlockItem = structureItem(0x00420040, limits,
                item(0x00420042, TtlvValue.Enumeration(1), limits));
        try (keyBlockItem; KeyBlock keyBlock = KeyBlock.fromItem(keyBlockItem);
                TtlvItemView keyBlockView = keyBlock.view();
                Tag keyBlockTag = keyBlockView.tag()) {
            assertEquals(0x00420040, keyBlockTag.raw());
        }
    }

    private static TtlvItem item(int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            return TtlvItem.create(tag, value, limits);
        }
    }

    private static TtlvItem structureItem(
            int rawTag, CodecLimits limits, TtlvItem... children) {
        TtlvStructure structure = TtlvStructure.create();
        for (TtlvItem child : children) {
            structure = TtlvStructure.withItem(structure, child, limits);
        }
        return item(rawTag, TtlvValue.structure(structure, limits), limits);
    }

    private static TtlvItem keyBlock(CodecLimits limits) {
        return structureItem(0x00420040, limits,
                item(0x00420042, TtlvValue.Enumeration(1), limits));
    }
}
