package org.kmipkit.typed;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;
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
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

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

    @Test
    void buildsCertificateInSchemaOrderAndReadsNamedValues() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvItem certificateValue = item(0x0042001e,
                TtlvValue.ByteString("certificate-canary".getBytes(), limits), limits);
        TtlvItem vendorFirst = item(0x00540010,
                TtlvValue.ByteString("vendor-one".getBytes(), limits), limits);
        TtlvItem certificateType = item(0x0042001d,
                TtlvValue.Enumeration(0x80000042L), limits);
        TtlvItem vendorRepeat = item(0x00540010,
                TtlvValue.ByteString("vendor-two".getBytes(), limits), limits);
        try (certificateValue; vendorFirst; certificateType; vendorRepeat;
                ManagedObject managed = ManagedObject.fromFields(1,
                        List.of(certificateValue, vendorFirst, certificateType, vendorRepeat))) {
            assertEquals(0x80000042L, managed.certificateType());
            assertEquals("certificate-canary", new String(managed.certificateValue()));
            try (TtlvItemView objectView = managed.view();
                    TtlvValueView objectValue = objectView.value();
                    TtlvStructureView structure = objectValue.structure()) {
                assertEquals(4, structure.itemCount());
                List<Integer> tags = new java.util.ArrayList<>();
                for (long index = 0; index < structure.itemCount(); index++) {
                    try (TtlvItemView field = structure.itemAt(index); Tag tag = field.tag()) {
                        tags.add(tag.raw());
                    }
                }
                assertEquals(List.of(0x0042001d, 0x00540010, 0x0042001e, 0x00540010), tags);
            }
        }
    }

    @Test
    void buildsTransparentRsaPublicMaterialInSchemaOrder() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvItem exponent = item(0x0042006c,
                TtlvValue.BigInteger(new byte[] {1, 0, 1}, limits), limits);
        TtlvItem modulus = item(0x00420052,
                TtlvValue.BigInteger(new byte[] {(byte) 0x80, 1}, limits), limits);
        try (exponent; modulus;
                KeyMaterial material = KeyMaterial.fromFields(11, List.of(exponent, modulus));
                TtlvItemView itemView = material.view();
                TtlvValueView valueView = itemView.value();
                TtlvStructureView structure = valueView.structure()) {
            assertEquals(11, material.formatRaw());
            List<Integer> tags = new java.util.ArrayList<>();
            for (long index = 0; index < structure.itemCount(); index++) {
                try (TtlvItemView field = structure.itemAt(index); Tag tag = field.tag()) {
                    tags.add(tag.raw());
                }
            }
            assertEquals(List.of(0x00420052, 0x0042006c), tags);
        }
    }

    @Test
    void namedConstructorsCoverAllNineObjectsAndReadNormativeFields() {
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvItem certificateType = item(0x0042001d,
                     TtlvValue.Enumeration(0x80000042L), limits);
                TtlvItem certificateValue = item(0x0042001e,
                     TtlvValue.ByteString(new byte[] {1, 2}, limits), limits);
                ManagedObject certificate = ManagedObject.certificate(
                        List.of(certificateValue, certificateType))) {
            assertEquals(0x80000042L, certificate.certificateType());
            assertEquals(0x80000042L, certificate.readField(0x0042001d, 3));
            assertEquals(TtlvItemType.ByteString, certificate.readField(0x0042001e, 2));
            byte[] certificateBytes = certificate.certificateValue();
            assertEquals(1, Byte.toUnsignedInt(certificateBytes[0]));
            assertEquals(2, Byte.toUnsignedInt(certificateBytes[1]));
        }
        try (TtlvItem type = item(0x00420059, TtlvValue.Enumeration(9), limits);
                TtlvItem value = item(0x0042005a, TtlvValue.ByteString(new byte[] {3}, limits), limits);
                ManagedObject opaque = ManagedObject.opaqueObject(List.of(type, value))) {
            assertEquals(9, opaque.opaqueDataType());
            assertEquals(3, opaque.opaqueDataValue()[0]);
        }
        try (TtlvItem type = item(0x00420019, TtlvValue.Enumeration(4), limits);
                TtlvItem value = item(0x00420140, TtlvValue.ByteString(new byte[] {5}, limits), limits);
                ManagedObject request = ManagedObject.certificateRequest(List.of(type, value))) {
            assertEquals(4, request.certificateRequestType());
            assertEquals(5, request.certificateRequestValue()[0]);
        }
        try (TtlvItem version = item(0x004200be, TtlvValue.Integer(7), limits);
                ManagedObject pgp = ManagedObject.pgpKey(List.of(version, keyBlock(limits)))) {
            assertEquals(7, pgp.pgpKeyVersion());
        }
        try (TtlvItem type = item(0x00420086, TtlvValue.Enumeration(3), limits);
                ManagedObject secret = ManagedObject.secretData(List.of(type, keyBlock(limits)))) {
            assertEquals(3, secret.secretDataType());
        }
        try (ManagedObject symmetric = ManagedObject.symmetricKey(List.of(keyBlock(limits)));
                ManagedObject publicKey = ManagedObject.publicKey(List.of(keyBlock(limits)));
                ManagedObject privateKey = ManagedObject.privateKey(List.of(keyBlock(limits)))) {
            assertEquals(2, symmetric.objectType());
            assertEquals(3, publicKey.objectType());
            assertEquals(4, privateKey.objectType());
        }
        try (TtlvItem parts = item(0x0042008b, TtlvValue.Integer(2), limits);
                TtlvItem identifier = item(0x00420044, TtlvValue.Integer(1), limits);
                TtlvItem threshold = item(0x0042008c, TtlvValue.Integer(2), limits);
                TtlvItem method = item(0x0042008a, TtlvValue.Enumeration(1), limits);
                ManagedObject split = ManagedObject.splitKey(
                        List.of(parts, identifier, threshold, method, keyBlock(limits)))) {
            assertEquals(1, split.keyPartIdentifier());
            assertEquals(1, split.splitKeyMethod());
            assertEquals(2, split.splitKeyParts());
            assertEquals(2, split.splitKeyThreshold());
        }
    }

    @Test
    void buildsKeyStructuresAndWrappingDataFromTypedFields() {
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvItem materialField = item(0x00420043,
                     TtlvValue.ByteString(new byte[] {9, 8}, limits), limits);
                KeyMaterial material = KeyMaterial.fromFields(1, List.of(materialField))) {
            assertEquals(1, material.formatRaw());
        }
        try (TtlvItem keyMaterial = item(0x00420043,
                     TtlvValue.ByteString(new byte[] {9, 8}, limits), limits);
                KeyValue keyValue = KeyValue.fromFields(1, List.of(keyMaterial));
                TtlvItemView keyValueView = keyValue.view();
                Tag keyValueTag = keyValueView.tag()) {
            assertEquals(0x00420045, keyValueTag.raw());
            try (TtlvItem format = item(0x00420042, TtlvValue.Enumeration(1), limits);
                    TtlvItem plainValue = structureItem(0x00420045, limits,
                            item(0x00420043,
                                    TtlvValue.ByteString(new byte[] {9, 8}, limits), limits),
                            item(0x00420125,
                                    TtlvValue.structure(TtlvStructure.create(), limits), limits));
                    KeyBlock block = KeyBlock.fromFields(List.of(format, plainValue));
                    TtlvItemView view = block.view();
                    Tag tag = view.tag()) {
                assertEquals(0x00420040, tag.raw());
            }
        }
        try (TtlvItem method = item(0x0042009e, TtlvValue.Enumeration(1), limits);
                TtlvItem uniqueId = item(0x00420094,
                        TtlvValue.TextString("wrapping-key".getBytes(), limits), limits);
                TtlvItem keyInfo = structureItem(0x00420036, limits, uniqueId);
                KeyWrappingData wrapping = KeyWrappingData.fromFields(List.of(method, keyInfo));
                TtlvItemView view = wrapping.view();
                Tag tag = view.tag()) {
            assertEquals(1, wrapping.wrappingMethod());
            assertEquals(0x00420046, tag.raw());
        }
    }

    @Test
    void readsKeyBlockKeyValueWrappingAndTransparentFields() {
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvItem modulus = item(0x00420052,
                     TtlvValue.BigInteger(new byte[] {(byte) 0x80, 1}, limits), limits);
                TtlvItem exponent = item(0x0042006c,
                     TtlvValue.BigInteger(new byte[] {1, 0, 1}, limits), limits);
                KeyMaterial material = KeyMaterial.fromFields(11, List.of(modulus, exponent))) {
            assertEquals(11, material.formatRaw());
            assertEquals(TtlvItemType.BigInteger, material.fieldType(0x00420052));
            assertEquals(0x80, Byte.toUnsignedInt(material.modulus()[0]));
            assertEquals(1, Byte.toUnsignedInt(material.publicExponent()[0]));
        }

        try (TtlvItem modulus = item(0x00420052,
                     TtlvValue.BigInteger(new byte[] {(byte) 0x80, 1}, limits), limits);
                TtlvItem exponent = item(0x0042006c,
                     TtlvValue.BigInteger(new byte[] {1, 0, 1}, limits), limits);
                TtlvItem material = structureItem(0x00420043, limits, modulus, exponent);
                TtlvItem attributes = structureItem(0x00420125, limits);
                TtlvItem keyValueItem = structureItem(0x00420045, limits, material, attributes);
                KeyValue keyValue = KeyValue.fromItem(11, keyValueItem);
                TtlvItem format = item(0x00420042, TtlvValue.Enumeration(11), limits);
                TtlvItem algorithm = item(0x00420028,
                        TtlvValue.Enumeration(0x80000010L), limits);
                TtlvItem length = item(0x0042002a, TtlvValue.Integer(2048), limits);
                KeyBlock keyBlock = KeyBlock.fromFields(List.of(format, keyValueItem, algorithm, length))) {
            assertEquals(TtlvItemType.Structure, keyValue.keyMaterialType());
            assertEquals(true, keyValue.hasAttributes());
            assertEquals(11, keyBlock.keyFormatType());
            assertEquals(TtlvItemType.Structure, keyBlock.keyValueType());
            assertEquals(0x80000010L, keyBlock.cryptographicAlgorithm());
            assertEquals(2048, keyBlock.cryptographicLength());
            assertEquals(false, keyBlock.hasKeyWrappingData());
        }

        try (TtlvItem method = item(0x0042009e,
                     TtlvValue.Enumeration(0x80000020L), limits);
                TtlvItem uniqueId = item(0x00420094,
                        TtlvValue.TextString("key-id".getBytes(), limits), limits);
                TtlvItem keyInfo = structureItem(0x00420036, limits, uniqueId);
                TtlvItem mac = item(0x0042004d,
                        TtlvValue.ByteString(new byte[] {9, 8}, limits), limits);
                TtlvItem iv = item(0x0042003d,
                        TtlvValue.ByteString(new byte[] {7, 6}, limits), limits);
                TtlvItem encoding = item(0x004200a3, TtlvValue.Enumeration(2), limits);
                KeyWrappingData wrapping = KeyWrappingData.fromFields(
                        List.of(method, keyInfo, mac, iv, encoding))) {
            assertEquals(0x80000020L, wrapping.wrappingMethod());
            assertEquals(true, wrapping.hasEncryptionKeyInformation());
            assertEquals(false, wrapping.hasMacSignatureKeyInformation());
            assertEquals(9, Byte.toUnsignedInt(wrapping.macSignature()[0]));
            assertEquals(7, Byte.toUnsignedInt(wrapping.ivCounterNonce()[0]));
            assertEquals(2, wrapping.encodingOption());
        }

        try (TtlvItem block = keyBlock(limits);
                ManagedObject symmetric = ManagedObject.symmetricKey(List.of(block))) {
            assertEquals(true, symmetric.hasKeyBlock());
        }
    }

    @Test
    void typedReadersCoverEveryTransparentKeyFormat() {
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvItem key = item(0x0042003f,
                     TtlvValue.ByteString(new byte[] {7}, limits), limits);
                KeyMaterial material = KeyMaterial.fromFields(7, List.of(key))) {
            assertEquals(TtlvItemType.ByteString, material.fieldType(0x0042003f));
            assertArrayEquals(new byte[] {7}, material.keyByteString());
        }
        try (TtlvItem p = item(0x0042005e, TtlvValue.BigInteger(new byte[] {8}, limits), limits);
                TtlvItem q = item(0x00420071, TtlvValue.BigInteger(new byte[] {9}, limits), limits);
                TtlvItem g = item(0x00420037, TtlvValue.BigInteger(new byte[] {10}, limits), limits);
                TtlvItem x = item(0x0042009f, TtlvValue.BigInteger(new byte[] {11}, limits), limits);
                KeyMaterial material = KeyMaterial.fromFields(8, List.of(p, q, g, x))) {
            assertArrayEquals(new byte[] {8}, material.p());
            assertArrayEquals(new byte[] {9}, material.q());
            assertArrayEquals(new byte[] {10}, material.g());
            assertArrayEquals(new byte[] {11}, material.x());
        }
        try (TtlvItem p = item(0x0042005e, TtlvValue.BigInteger(new byte[] {12}, limits), limits);
                TtlvItem q = item(0x00420071, TtlvValue.BigInteger(new byte[] {13}, limits), limits);
                TtlvItem g = item(0x00420037, TtlvValue.BigInteger(new byte[] {14}, limits), limits);
                TtlvItem y = item(0x004200a0, TtlvValue.BigInteger(new byte[] {15}, limits), limits);
                KeyMaterial material = KeyMaterial.fromFields(9, List.of(p, q, g, y))) {
            assertArrayEquals(new byte[] {12}, material.p());
            assertArrayEquals(new byte[] {13}, material.q());
            assertArrayEquals(new byte[] {14}, material.g());
            assertArrayEquals(new byte[] {15}, material.y());
        }
        try (TtlvItem modulus = item(0x00420052,
                     TtlvValue.BigInteger(new byte[] {16}, limits), limits);
                TtlvItem privateExponent = item(0x00420063,
                     TtlvValue.BigInteger(new byte[] {17}, limits), limits);
                KeyMaterial material = KeyMaterial.fromFields(10, List.of(modulus, privateExponent))) {
            assertArrayEquals(new byte[] {16}, material.modulus());
            assertArrayEquals(new byte[] {17}, material.privateExponent());
        }
        try (TtlvItem modulus = item(0x00420052,
                     TtlvValue.BigInteger(new byte[] {18}, limits), limits);
                TtlvItem exponent = item(0x0042006c,
                     TtlvValue.BigInteger(new byte[] {19}, limits), limits);
                KeyMaterial material = KeyMaterial.fromFields(11, List.of(modulus, exponent))) {
            assertArrayEquals(new byte[] {18}, material.modulus());
            assertArrayEquals(new byte[] {19}, material.publicExponent());
        }
        try (TtlvItem p = item(0x0042005e, TtlvValue.BigInteger(new byte[] {20}, limits), limits);
                TtlvItem g = item(0x00420037, TtlvValue.BigInteger(new byte[] {21}, limits), limits);
                TtlvItem x = item(0x0042009f, TtlvValue.BigInteger(new byte[] {22}, limits), limits);
                KeyMaterial material = KeyMaterial.fromFields(12, List.of(p, g, x))) {
            assertArrayEquals(new byte[] {20}, material.p());
            assertArrayEquals(new byte[] {21}, material.g());
            assertArrayEquals(new byte[] {22}, material.x());
        }
        try (TtlvItem p = item(0x0042005e, TtlvValue.BigInteger(new byte[] {23}, limits), limits);
                TtlvItem g = item(0x00420037, TtlvValue.BigInteger(new byte[] {24}, limits), limits);
                TtlvItem y = item(0x004200a0, TtlvValue.BigInteger(new byte[] {25}, limits), limits);
                KeyMaterial material = KeyMaterial.fromFields(13, List.of(p, g, y))) {
            assertArrayEquals(new byte[] {23}, material.p());
            assertArrayEquals(new byte[] {24}, material.g());
            assertArrayEquals(new byte[] {25}, material.y());
        }
        try (TtlvItem curve = item(0x00420075,
                     TtlvValue.Enumeration(0x80000014L), limits);
                TtlvItem d = item(0x0042002e, TtlvValue.BigInteger(new byte[] {26}, limits), limits);
                KeyMaterial material = KeyMaterial.fromFields(14, List.of(curve, d))) {
            assertEquals(0x80000014L, material.recommendedCurve());
            assertArrayEquals(new byte[] {26}, material.d());
        }
        try (TtlvItem curve = item(0x00420075,
                     TtlvValue.Enumeration(0x80000015L), limits);
                TtlvItem qString = item(0x00420072,
                     TtlvValue.ByteString(new byte[] {27, 28}, limits), limits);
                KeyMaterial material = KeyMaterial.fromFields(15, List.of(curve, qString))) {
            assertEquals(0x80000015L, material.recommendedCurve());
            assertArrayEquals(new byte[] {27, 28}, material.qString());
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
