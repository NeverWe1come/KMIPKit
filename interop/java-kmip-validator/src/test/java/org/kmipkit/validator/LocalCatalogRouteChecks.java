package org.kmipkit.validator;

import com.fasterxml.jackson.databind.JsonNode;
import org.kmipkit.extensions.InvalidExtensionSchemaException;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TypedAttribute;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvDecodeErrorKind;
import org.kmipkit.ttlv.TtlvDecodeException;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

import java.nio.ByteBuffer;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

/** Executes source-backed local catalog obligations through the installed public package. */
final class LocalCatalogRouteChecks {
    private static final int TAG_VALUE_SENTINEL = 0x12345678;

    private LocalCatalogRouteChecks() { }

    @FunctionalInterface
    interface EvidenceSink {
        void record(Invocation invocation) throws Exception;
    }

    record Invocation(String caseId, String targetId, String routeId, String symbol,
                      String action, byte[] wire, String rejectionCode) { }

    // OASIS KMIP 2.0 §11.54 / 2.1 §11.56 and §10.1: tag identity and allocation.
    static void executeTags(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        for (JsonNode testCase : catalog.path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if (!"tag".equals(requires.path("target_kind").asText()) || !applies(testCase, version)) continue;
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId)) continue;
            String targetId = requires.path("normative_target_id").asText();
            int tagCode = Integer.parseUnsignedInt(
                    requires.path("wire_value").asText().replaceFirst("^0[xX]", ""), 16);
            boolean reserved = "reserved".equals(requires.path("allocation").asText());
            byte[] wire = scalarWire(tagCode, 2, TAG_VALUE_SENTINEL);
            try (RawTag raw = RawTag.fromRaw(tagCode)) {
                assertEquals(tagCode, raw.raw(), "The exact normative 24-bit tag must survive construction.");
                emit(sink, caseId, targetId, "model.org.kmipkit.ttlv.rawtag.fromraw",
                        "org.kmipkit.ttlv.RawTag.fromRaw", "construct", wire);
                if (reserved) {
                    assertThrows(InvalidExtensionSchemaException.class, raw::checked,
                            "A reserved allocation cannot become an allocation-checked Tag.");
                    emit(sink, caseId, targetId, "model.org.kmipkit.ttlv.rawtag.checked",
                            "org.kmipkit.ttlv.RawTag.checked", "construct", wire);
                } else {
                    try (Tag checked = raw.checked(); TtlvValue value = TtlvValue.Integer(TAG_VALUE_SENTINEL);
                            TtlvItem item = TtlvItem.create(checked, value, CodecLimits.defaults())) {
                        assertEquals(tagCode, checked.raw());
                        assertScalarItem(item, tagCode, TtlvItemType.Integer, TAG_VALUE_SENTINEL);
                    }
                    emit(sink, caseId, targetId, "model.org.kmipkit.ttlv.rawtag.checked",
                            "org.kmipkit.ttlv.RawTag.checked", "construct", wire);
                    emit(sink, caseId, targetId, "ttlv_item.create", "org.kmipkit.ttlv.TtlvItem.create",
                            "construct", wire);
                    emit(sink, caseId, targetId, "function.ttlv_item_view_from_item",
                            "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
                }
            }
            if (reserved) {
                TtlvDecodeException failure = assertThrows(TtlvDecodeException.class,
                        () -> TtlvCodec.decode(wire));
                assertEquals(TtlvDecodeErrorKind.RESERVED_TAG, failure.kind());
                sink.record(new Invocation(caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                        "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire, "tag_reserved"));
            } else {
                try (TtlvItem decoded = TtlvCodec.decode(wire)) {
                    assertScalarItem(decoded, tagCode, TtlvItemType.Integer, TAG_VALUE_SENTINEL);
                }
                emit(sink, caseId, targetId, "ttlv_codec.decode", "org.kmipkit.ttlv.TtlvCodec.decode",
                        "decode", wire);
            }
        }
    }

    /** Exercises assigned tag identities whose targets also have Java typed-protocol routes. */
    static void executeTagIdentities(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        for (JsonNode testCase : catalog.path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if (!"tag".equals(requires.path("target_kind").asText()) || !applies(testCase, version)) continue;
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId)) continue;
            assertEquals("assigned", requires.path("allocation").asText(),
                    "Typed Java tag targets must use an assigned KMIP tag.");
            String targetId = requires.path("normative_target_id").asText();
            int tagCode = Integer.parseUnsignedInt(
                    requires.path("wire_value").asText().replaceFirst("^0[xX]", ""), 16);
            StandardAttribute attribute = standardAttributeForTag(tagCode);
            if (attribute == null) throw new AssertionError("Catalogued Java typed tag has no StandardAttribute.");
            try (AttributeValue attributeValue = attributeValue(attribute);
                    AttributeValue typedAttributeValue = attributeValue(attribute);
                    RawTag raw = RawTag.fromRaw(tagCode); Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, attributeValue.value(), CodecLimits.defaults());
                    TypedAttribute typed = TypedAttribute.create(attribute, typedAttributeValue.value());
                    TtlvItem typedItem = typed.asItem();
                    TypedAttribute decoded = TypedAttribute.fromItem(attribute, typedItem)) {
                byte[] itemWire = itemWire(tagCode, attributeValue.typeCode(), attributeValue.payload());
                assertEquals(tagCode, raw.raw(), "The exact assigned tag must survive RawTag construction.");
                assertEquals(tagCode, tag.raw(), "The checked Tag must preserve its exact raw value.");
                assertEquals(attribute, decoded.attribute(), "TypedAttribute.fromItem must preserve the attribute.");
                try (var view = decoded.view(); var viewedTag = view.tag(); var viewedValue = view.value()) {
                    assertEquals(tagCode, viewedTag.raw());
                    assertEquals(attributeValue.typeCode(), viewedValue.itemType().code());
                }

                byte[] attributeSetWire = structureWire(0x420125, itemWire);
                emitRoute(sink, caseId, targetId, "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
                        "org.kmipkit.ttlv.TypedAttribute.create", "construct", itemWire);
                emitRoute(sink, caseId, targetId, "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem",
                        "org.kmipkit.ttlv.TypedAttribute.fromItem", "construct", itemWire);
                try (TtlvStructure empty = TtlvStructure.create();
                        TtlvStructure items = TtlvStructure.withItem(empty, typedItem, CodecLimits.defaults());
                        AttributeSet attributes = AttributeSet.tryNew(items)) {
                    assertEquals(1, attributes.len(), "AttributeSet.tryNew must retain the typed attribute item.");
                    emitRoute(sink, caseId, targetId,
                            "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew",
                            "org.kmipkit.extensions.AttributeSet.tryNew", "construct", attributeSetWire);
                }
            }
        }
    }

    private static StandardAttribute standardAttributeForTag(int tagCode) {
        for (StandardAttribute attribute : StandardAttribute.values()) {
            if (attribute.tag() == tagCode) return attribute;
        }
        return null;
    }

    private static AttributeValue attributeValue(StandardAttribute attribute) {
        String kind = attribute.valueKind().toLowerCase(java.util.Locale.ROOT).replaceAll("[^a-z0-9]", "");
        return switch (kind) {
            case "integer" -> new AttributeValue(TtlvValue.Integer(1), null, 2, ByteBuffer.allocate(4).putInt(1).array());
            case "longinteger" -> new AttributeValue(TtlvValue.LongInteger(1), null, 3,
                    ByteBuffer.allocate(8).putLong(1).array());
            case "enumeration" -> new AttributeValue(TtlvValue.Enumeration(1), null, 5,
                    ByteBuffer.allocate(4).putInt(1).array());
            case "boolean" -> new AttributeValue(TtlvValue.Boolean(true), null, 6,
                    ByteBuffer.allocate(8).putLong(1).array());
            case "datetime" -> new AttributeValue(TtlvValue.DateTime(1), null, 9,
                    ByteBuffer.allocate(8).putLong(1).array());
            case "interval" -> new AttributeValue(TtlvValue.Interval(1), null, 10,
                    ByteBuffer.allocate(4).putInt(1).array());
            case "textstring", "uniqueidentifier" -> {
                byte[] payload = "object-7".getBytes(java.nio.charset.StandardCharsets.UTF_8);
                yield new AttributeValue(TtlvValue.TextString(payload, CodecLimits.defaults()), null, 7, payload);
            }
            case "bytestring" -> {
                byte[] payload = new byte[] {0x01, 0x02};
                yield new AttributeValue(TtlvValue.ByteString(payload, CodecLimits.defaults()), null, 8, payload);
            }
            case "structure" -> {
                byte[] children = new byte[0];
                TtlvStructure structure = TtlvStructure.create();
                try {
                    if (attribute == StandardAttribute.VENDOR_ATTRIBUTE) {
                        byte[] vendor = "KMIPKit.TestVendor_1".getBytes(java.nio.charset.StandardCharsets.UTF_8);
                        byte[] name = "Opaque.Attribute".getBytes(java.nio.charset.StandardCharsets.UTF_8);
                        byte[] value = new byte[] {0x01};
                        structure = append(structure, 0x42009d,
                                TtlvValue.TextString(vendor, CodecLimits.defaults()));
                        structure = append(structure, 0x42000a,
                                TtlvValue.TextString(name, CodecLimits.defaults()));
                        structure = append(structure, 0x42000b,
                                TtlvValue.ByteString(value, CodecLimits.defaults()));
                        children = concat(itemWire(0x42009d, 7, vendor), itemWire(0x42000a, 7, name),
                                itemWire(0x42000b, 8, value));
                    } else if (attribute == StandardAttribute.APPLICATION_SPECIFIC_INFORMATION) {
                        byte[] first = "first unknown".getBytes(java.nio.charset.StandardCharsets.UTF_8);
                        byte[] known = "known child".getBytes(java.nio.charset.StandardCharsets.UTF_8);
                        byte[] last = "second unknown".getBytes(java.nio.charset.StandardCharsets.UTF_8);
                        structure = append(structure, 0x540001,
                                TtlvValue.TextString(first, CodecLimits.defaults()));
                        structure = append(structure, 0x42000a,
                                TtlvValue.TextString(known, CodecLimits.defaults()));
                        structure = append(structure, 0x540001,
                                TtlvValue.TextString(last, CodecLimits.defaults()));
                        children = concat(itemWire(0x540001, 7, first), itemWire(0x42000a, 7, known),
                                itemWire(0x540001, 7, last));
                    }
                    yield new AttributeValue(TtlvValue.structure(structure, CodecLimits.defaults()), structure,
                            1, children);
                } catch (RuntimeException failure) {
                    structure.close();
                    throw failure;
                }
            }
            default -> throw new IllegalArgumentException("Unsupported StandardAttribute value kind.");
        };
    }

    private static byte[] itemWire(int tag, int type, byte[] value) {
        int paddedLength = Math.addExact(value.length, 7) & ~7;
        return ByteBuffer.allocate(Math.addExact(8, paddedLength))
                .put((byte) (tag >>> 16)).put((byte) (tag >>> 8)).put((byte) tag)
                .put((byte) type).putInt(value.length).put(value).put(new byte[paddedLength - value.length]).array();
    }

    private static byte[] structureWire(int tag, byte[] child) {
        return itemWire(tag, 1, child);
    }

    private static TtlvStructure append(TtlvStructure structure, int tagCode, TtlvValue value) {
        try (RawTag raw = RawTag.fromRaw(tagCode); Tag tag = raw.checked();
                TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults())) {
            return TtlvStructure.withItem(structure, item, CodecLimits.defaults());
        }
    }

    private static byte[] concat(byte[]... values) {
        int length = 0;
        for (byte[] value : values) length = Math.addExact(length, value.length);
        byte[] result = new byte[length];
        int offset = 0;
        for (byte[] value : values) {
            System.arraycopy(value, 0, result, offset, value.length);
            offset += value.length;
        }
        return result;
    }

    private record AttributeValue(TtlvValue value, TtlvStructure ownedStructure, int typeCode, byte[] payload)
            implements AutoCloseable {
        @Override
        public void close() {
            value.close();
            if (ownedStructure != null) ownedStructure.close();
        }
    }

    private static boolean applies(JsonNode testCase, String version) {
        for (JsonNode candidate : testCase.path("execution").path("protocol_versions")) {
            if (version.equals(candidate.asText())) return true;
        }
        return false;
    }

    // OASIS KMIP 2.0/2.1 §§11.38-11.39 reference the pinned PKCS#11 value inventories.
    static void executeReferencedEnums(JsonNode catalog, String version, EvidenceSink sink) throws Exception {
        for (JsonNode testCase : catalog.path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if (!"referenced_enum_value".equals(requires.path("target_kind").asText())
                    || !applies(testCase, version)) continue;
            int tagCode = switch (requires.path("parent_name").asText()) {
                case "PKCS#11 Function" -> 0x42015a;
                case "PKCS#11 Return Code" -> 0x42015d;
                default -> throw new AssertionError("Referenced enumeration has no source-backed tag fixture.");
            };
            String aggregateCaseId = switch (requires.path("parent_name").asText()) {
                case "PKCS#11 Function" -> "test-enum-pkcs#11-function";
                case "PKCS#11 Return Code" -> "test-enum-pkcs#11-return-code";
                default -> throw new AssertionError("Referenced enumeration has no aggregate route case.");
            };
            String caseId = testCase.path("case_id").asText();
            String targetId = requires.path("normative_target_id").asText();
            long expected = Long.parseUnsignedLong(
                    requires.path("wire_value").asText().replaceFirst("^0[xX]", ""), 16);
            byte[] wire = scalarWire(tagCode, 5, (int) expected);
            try (RawTag raw = RawTag.fromRaw(tagCode); Tag tag = raw.checked();
                    TtlvValue value = TtlvValue.Enumeration(expected);
                    TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults())) {
                assertScalarItem(item, tagCode, TtlvItemType.Enumeration, expected);
            }
            for (String evidenceCaseId : List.of(caseId, aggregateCaseId)) {
                emit(sink, evidenceCaseId, targetId, "model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                        "org.kmipkit.ttlv.TtlvValue.Enumeration", "construct", wire);
                emit(sink, evidenceCaseId, targetId, "ttlv_item.create", "org.kmipkit.ttlv.TtlvItem.create",
                        "construct", wire);
                emit(sink, evidenceCaseId, targetId, "function.ttlv_item_view_from_item",
                        "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
            }
            try (TtlvItem decoded = TtlvCodec.decode(wire)) {
                assertScalarItem(decoded, tagCode, TtlvItemType.Enumeration, expected);
            }
            for (String evidenceCaseId : List.of(caseId, aggregateCaseId)) {
                emit(sink, evidenceCaseId, targetId, "ttlv_codec.decode", "org.kmipkit.ttlv.TtlvCodec.decode",
                        "decode", wire);
            }
        }
    }

    private static void assertScalarItem(TtlvItem item, int tag, TtlvItemType type, long expected) {
        try (var view = item.view(); var viewedTag = view.tag(); var value = view.value()) {
            assertEquals(tag, viewedTag.raw());
            assertEquals(type, view.itemType());
            assertEquals(expected, type == TtlvItemType.Enumeration ? value.enumeration() : value.integer(),
                    "The installed API must preserve the exact tag, type, and scalar wire value.");
        }
    }

    private static void emit(EvidenceSink sink, String caseId, String targetId, String suffix,
            String symbol, String action, byte[] wire) throws Exception {
        emitRoute(sink, caseId, targetId, "java.generic-ttlv." + suffix, symbol, action, wire);
    }

    private static void emitRoute(EvidenceSink sink, String caseId, String targetId, String routeId,
            String symbol, String action, byte[] wire) throws Exception {
        sink.record(new Invocation(caseId, targetId, routeId,
                symbol, action, wire, null));
    }

    private static byte[] scalarWire(int tag, int type, int value) {
        return ByteBuffer.allocate(16).put((byte) (tag >>> 16)).put((byte) (tag >>> 8)).put((byte) tag)
                .put((byte) type).putInt(4).putInt(value).putInt(0).array();
    }
}
