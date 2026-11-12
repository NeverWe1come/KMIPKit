package org.kmipkit.validator;

import com.fasterxml.jackson.databind.JsonNode;
import org.kmipkit.extensions.InvalidExtensionSchemaException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvDecodeErrorKind;
import org.kmipkit.ttlv.TtlvDecodeException;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvValue;

import java.nio.ByteBuffer;
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
            emit(sink, caseId, targetId, "model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                    "org.kmipkit.ttlv.TtlvValue.Enumeration", "construct", wire);
            emit(sink, caseId, targetId, "ttlv_item.create", "org.kmipkit.ttlv.TtlvItem.create",
                    "construct", wire);
            emit(sink, caseId, targetId, "function.ttlv_item_view_from_item",
                    "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
            try (TtlvItem decoded = TtlvCodec.decode(wire)) {
                assertScalarItem(decoded, tagCode, TtlvItemType.Enumeration, expected);
            }
            emit(sink, caseId, targetId, "ttlv_codec.decode", "org.kmipkit.ttlv.TtlvCodec.decode",
                    "decode", wire);
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
        sink.record(new Invocation(caseId, targetId, "java.generic-ttlv." + suffix,
                symbol, action, wire, null));
    }

    private static byte[] scalarWire(int tag, int type, int value) {
        return ByteBuffer.allocate(16).put((byte) (tag >>> 16)).put((byte) (tag >>> 8)).put((byte) tag)
                .put((byte) type).putInt(4).putInt(value).putInt(0).array();
    }
}
