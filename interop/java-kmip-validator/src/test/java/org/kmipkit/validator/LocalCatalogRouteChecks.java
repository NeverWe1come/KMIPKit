package org.kmipkit.validator;

import com.fasterxml.jackson.databind.JsonNode;
import org.kmipkit.extensions.InvalidExtensionSchemaException;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TypedAttribute;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvDecodeErrorKind;
import org.kmipkit.ttlv.TtlvDecodeException;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

/** Executes source-backed local catalog obligations through the installed public package. */
final class LocalCatalogRouteChecks {
    private static final int TAG_VALUE_SENTINEL = 0x12345678;
    static final String ROTATE_NAME_TYPE_CASE_ID =
            "test-norm-structure-member-structure-member-4-53-rotate-name-type";
    static final String ROTATE_NAME_TYPE_TARGET_ID = "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-53-ROTATE-NAME-TYPE";
    static final String ROTATE_NAME_VALUE_CASE_ID =
            "test-norm-structure-member-structure-member-4-53-rotate-name-value";
    static final String ROTATE_NAME_VALUE_TARGET_ID = "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-53-ROTATE-NAME-VALUE";
    static final String REQUEST_HEADER_SCALAR_CASE_PREFIX =
            "test-norm-message-field-message-field-8-2-";
    static final String BATCH_ORDER_OPTION_CASE_ID =
            REQUEST_HEADER_SCALAR_CASE_PREFIX + "batch-order-option";
    static final String BATCH_ORDER_OPTION_TARGET_ID = "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ORDER-OPTION";
    static final String ATTESTATION_CAPABLE_CASE_ID =
            "test-norm-message-field-message-field-8-2-attestation-capable-indicator";
    static final String ATTESTATION_CAPABLE_TARGET_ID =
            "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ATTESTATION-CAPABLE-INDICATOR";
    static final Set<String> STRUCTURE_ATTRIBUTE_VALUE_CASE_IDS = Set.of(
            "test-attr-alternative-name",
            "test-attr-application-specific-information",
            "test-attr-key-value-location",
            "test-attr-link",
            "test-attr-name",
            "test-attr-revocation-reason",
            "test-attr-rotate-name");
    static final Set<String> TEXT_ATTRIBUTE_VALUE_CASE_IDS = Set.of(
            "test-attr-comment",
            "test-attr-contact-information",
            "test-attr-description",
            "test-attr-object-group",
            "test-attr-pkcs#12-friendly-name");
    static final Set<String> BOOLEAN_ATTRIBUTE_VALUE_CASE_IDS = Set.of(
            "test-attr-always-sensitive",
            "test-attr-extractable",
            "test-attr-fresh",
            "test-attr-key-value-present",
            "test-attr-never-extractable",
            "test-attr-quantum-safe",
            "test-attr-rotate-automatic",
            "test-attr-rotate-latest",
            "test-attr-sensitive");
    static final Set<String> DATE_TIME_ATTRIBUTE_VALUE_CASE_IDS = Set.of(
            "test-attr-activation-date",
            "test-attr-archive-date",
            "test-attr-compromise-date",
            "test-attr-compromise-occurrence-date",
            "test-attr-deactivation-date",
            "test-attr-destroy-date",
            "test-attr-initial-date",
            "test-attr-last-change-date",
            "test-attr-original-creation-date",
            "test-attr-process-start-date",
            "test-attr-protect-stop-date",
            "test-attr-rotate-date");
    static final Set<String> ENUMERATION_ATTRIBUTE_VALUE_CASE_IDS = Set.of(
            "test-attr-certificate-type",
            "test-attr-cryptographic-algorithm",
            "test-attr-digital-signature-algorithm",
            "test-attr-key-format-type",
            "test-attr-nist-key-type",
            "test-attr-object-type",
            "test-attr-opaque-data-type",
            "test-attr-protection-level",
            "test-attr-state");
    static final Set<String> SCALAR_ATTRIBUTE_VALUE_CASE_IDS = Set.of(
            "test-attr-certificate-length",
            "test-attr-cryptographic-length",
            "test-attr-cryptographic-usage-mask",
            "test-attr-protection-storage-mask",
            "test-attr-rotate-generation",
            "test-attr-lease-time",
            "test-attr-protection-period",
            "test-attr-rotate-interval",
            "test-attr-rotate-offset",
            "test-attr-short-unique-identifier");

    private LocalCatalogRouteChecks() { }

    @FunctionalInterface
    interface EvidenceSink {
        void record(Invocation invocation) throws Exception;
    }

    record Invocation(String caseId, String targetId, String routeId, String symbol,
                      String action, byte[] wire, String rejectionCode) { }

    private record StructureAttributeMember(String name, String textValue) { }

    /** Executes generic TTLV routes for Boolean fields using their catalogued tag identities. */
    static void executeBooleanFields(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        Set<String> emitted = new java.util.HashSet<>();
        JsonNode targets = catalog.path("normative_targets");
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId) || !"local_model".equals(
                    testCase.path("execution").path("mode").asText()) || !applies(testCase, version)) continue;
            String targetId = testCase.path("requires").path("normative_target_id").asText();
            JsonNode target = null;
            for (JsonNode candidate : targets) {
                if (targetId.equals(candidate.path("target_id").asText())) {
                    target = candidate;
                    break;
                }
            }
            if (target == null || !"Boolean".equals(target.path("source_encoding").asText())) {
                throw new IllegalArgumentException("Selected case is not a catalogued Boolean field: " + caseId);
            }
            String name = normalizeName(target.path("name").asText());
            int tagCode = -1;
            int matches = 0;
            for (JsonNode candidate : targets) {
                if (!"tag".equals(candidate.path("kind").asText())
                        || !name.equals(normalizeName(candidate.path("name").asText()))
                        || !supportsVersion(candidate, version)) continue;
                if (!"assigned".equals(candidate.path("allocation").asText())) {
                    throw new IllegalArgumentException("Boolean field tag is not assigned: " + targetId);
                }
                tagCode = Integer.parseUnsignedInt(
                        candidate.path("wire_value").asText().replaceFirst("^0[xX]", ""), 16);
                matches++;
            }
            if (matches != 1) {
                throw new IllegalArgumentException("Expected one assigned Tag for catalogued Boolean field: "
                        + targetId);
            }

            byte[] wire = booleanWire(tagCode, true);
            try (TtlvValue value = TtlvValue.Boolean(true);
                    RawTag raw = RawTag.fromRaw(tagCode);
                    Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults());
                    var itemView = item.view();
                    var itemTag = itemView.tag();
                    var itemValue = itemView.value();
                    TtlvItem decoded = TtlvCodec.decode(wire);
                    var decodedView = decoded.view();
                    var decodedTag = decodedView.tag();
                    var decodedValue = decodedView.value()) {
                assertEquals(tagCode, raw.raw());
                assertEquals(tagCode, itemTag.raw());
                assertEquals(TtlvItemType.Boolean, itemView.itemType());
                assertEquals(true, itemValue.booleanValue());
                assertEquals(tagCode, decodedTag.raw());
                assertEquals(TtlvItemType.Boolean, decodedView.itemType());
                assertEquals(true, decodedValue.booleanValue());
            }
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.boolean",
                    "org.kmipkit.ttlv.TtlvValue.Boolean", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                    "org.kmipkit.ttlv.TtlvItem.create", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                    "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire);
            if ("KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-RANDOM-IV".equals(targetId)) {
                executeTypedRandomIv(caseId, targetId, tagCode, sink);
            }
            emitted.add(caseId);
        }
        if (!emitted.equals(selectedCaseIds)) {
            throw new IllegalArgumentException("Selected Boolean field cases were not executed for " + version);
        }
    }

    /** Executes generic TTLV routes for catalogued scalar message fields. */
    static void executeScalarMessageFields(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        Set<String> emitted = new java.util.HashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId)
                    || !"local_model".equals(testCase.path("execution").path("mode").asText())
                    || !applies(testCase, version)) continue;
            JsonNode requires = testCase.path("requires");
            if (!"message_field".equals(requires.path("target_kind").asText())) {
                throw new IllegalArgumentException("Selected case is not a message field: " + caseId);
            }
            String targetId = requires.path("normative_target_id").asText();
            String encoding = requires.path("source_encoding").asText(
                    scalarMessageFieldEncoding(catalog, targetId, version));
            int tagCode = messageFieldTag(catalog, targetId, version);
            TtlvItemType itemType = scalarMessageFieldType(encoding);
            byte[] valueBytes = scalarMessageFieldBytes(caseId, encoding);
            long numericValue = scalarMessageFieldNumber(caseId, encoding);
            byte[] wire = scalarWire(tagCode, itemType.code(), valueBytes);
            CodecLimits limits = CodecLimits.defaults();
            try (TtlvValue value = scalarMessageFieldValue(encoding, valueBytes, numericValue, limits);
                    RawTag raw = RawTag.fromRaw(tagCode);
                    Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, value, limits);
                    var itemView = item.view();
                    var itemTag = itemView.tag();
                    var itemValue = itemView.value()) {
                assertEquals(tagCode, itemTag.raw());
                assertScalarMessageFieldValue(itemValue, itemType, valueBytes, numericValue);
            }
            try (TtlvItem decoded = TtlvCodec.decode(wire);
                    var decodedView = decoded.view();
                    var decodedTag = decodedView.tag();
                    var decodedValue = decodedView.value()) {
                assertEquals(tagCode, decodedTag.raw());
                assertScalarMessageFieldValue(decodedValue, itemType, valueBytes, numericValue);
            }
            String constructorRoute = scalarMessageFieldConstructorRoute(encoding);
            if (!caseId.startsWith(REQUEST_HEADER_SCALAR_CASE_PREFIX)) {
                emitRoute(sink, caseId, targetId, constructorRoute, scalarMessageFieldConstructor(encoding),
                        "construct", wire);
            }
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                    "org.kmipkit.ttlv.TtlvItem.create", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                    "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire);
            emitted.add(caseId);
        }
        if (!emitted.equals(selectedCaseIds)) {
            throw new IllegalArgumentException("Selected scalar message field cases were not executed for "
                    + version);
        }
    }

    /** Executes generic TTLV routes for catalogued scalar structure members. */
    static void executeScalarStructureMembers(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        Set<String> emitted = new java.util.HashSet<>();
        CodecLimits limits = CodecLimits.defaults();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId)
                    || !"local_model".equals(testCase.path("execution").path("mode").asText())
                    || !applies(testCase, version)) continue;
            JsonNode requires = testCase.path("requires");
            if (!"structure_member".equals(requires.path("target_kind").asText())) {
                throw new IllegalArgumentException("Selected case is not a structure member: " + caseId);
            }
            String targetId = requires.path("normative_target_id").asText();
            JsonNode target = findTarget(catalog, targetId);
            String encoding = target.path("source_encoding").asText();
            TtlvItemType itemType = scalarMessageFieldType(encoding);
            String targetName = target.path("name").asText();
            int tagCode = tagCodeNamed(catalog, targetName, version);
            long numericValue = "Enumeration".equals(encoding)
                    ? representativeEnumerationValue(catalog, targetName, version)
                    : scalarMessageFieldNumber(caseId, encoding);
            byte[] valueBytes = "Enumeration".equals(encoding)
                    ? ByteBuffer.allocate(Integer.BYTES).putInt((int) numericValue).array()
                    : scalarMessageFieldBytes(caseId, encoding);
            byte[] wire = scalarWire(tagCode, itemType.code(), valueBytes);
            try (TtlvValue value = scalarMessageFieldValue(encoding, valueBytes, numericValue, limits);
                    RawTag raw = RawTag.fromRaw(tagCode);
                    Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, value, limits);
                    var itemView = item.view();
                    var itemTag = itemView.tag();
                    var itemValue = itemView.value()) {
                assertEquals(tagCode, itemTag.raw());
                assertScalarMessageFieldValue(itemValue, itemType, valueBytes, numericValue);
            }
            try (TtlvItem decoded = TtlvCodec.decode(wire);
                    var decodedView = decoded.view();
                    var decodedTag = decodedView.tag();
                    var decodedValue = decodedView.value()) {
                assertEquals(tagCode, decodedTag.raw());
                assertScalarMessageFieldValue(decodedValue, itemType, valueBytes, numericValue);
            }
            emitRoute(sink, caseId, targetId, scalarMessageFieldConstructorRoute(encoding),
                    scalarMessageFieldConstructor(encoding), "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                    "org.kmipkit.ttlv.TtlvItem.create", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                    "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire);
            emitted.add(caseId);
        }
        if (!emitted.equals(selectedCaseIds)) {
            throw new IllegalArgumentException("Selected scalar structure member cases were not executed for "
                    + version);
        }
    }

    private static long representativeEnumerationValue(JsonNode catalog, String fieldName, String version) {
        String enumTypeName = "Mask Generator Hashing Algorithm".equals(fieldName)
                ? "HASHING-ALGORITHM"
                : fieldName.toUpperCase(Locale.ROOT).replaceAll("[^A-Z0-9]+", "-");
        String enumTypeId = "KMIPKIT-ELEM-ENUMERATION-" + enumTypeName;
        String normalizedFieldName = normalizeName(fieldName);
        long firstValue = Long.MAX_VALUE;
        boolean found = false;
        String extensionRange = null;
        for (JsonNode candidate : catalog.path("normative_targets")) {
            String kind = candidate.path("kind").asText();
            boolean standardEnum = "enumeration_value".equals(kind) && hasParent(candidate, enumTypeId);
            boolean referencedEnum = "referenced_enum_value".equals(kind)
                    && normalizedFieldName.equals(normalizeName(candidate.path("parent_name").asText()));
            if ((!standardEnum && !referencedEnum) || !supportsVersion(candidate, version)) continue;
            if ("extension".equals(candidate.path("allocation").asText())) {
                extensionRange = candidate.path("wire_value").asText();
                continue;
            }
            if (!"assigned".equals(candidate.path("allocation").asText())) continue;
            long value = Integer.toUnsignedLong(Integer.parseUnsignedInt(
                    candidate.path("wire_value").asText().replaceFirst("^0[xX]", ""), 16));
            if (!found || Long.compareUnsigned(value, firstValue) < 0) {
                firstValue = value;
                found = true;
            }
        }
        if (!found && extensionRange != null && extensionRange.matches("[0-9A-Fa-f]X{7}")) {
            return Long.parseLong(extensionRange.substring(0, 1), 16) << 28;
        }
        if (!found) {
            throw new IllegalArgumentException("No source-backed enumeration value for structure member: "
                    + fieldName + " / " + version);
        }
        return firstValue;
    }

    /** Executes selected scalar fields from OASIS KMIP 2.0/2.1 server-initiated operation payloads. */
    static void executeServerPayloadScalarFields(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        Set<String> emitted = new java.util.HashSet<>();
        CodecLimits limits = CodecLimits.defaults();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId)
                    || !"local_model".equals(testCase.path("execution").path("mode").asText())
                    || !applies(testCase, version)) continue;
            JsonNode requires = testCase.path("requires");
            String operationId = requires.path("operation_id").asText();
            if (!"operation_payload_field".equals(requires.path("target_kind").asText())
                    || !Set.of("KMIPKIT-ELEM-OP-S2C-QUERY", "KMIPKIT-ELEM-OP-S2C-NOTIFY")
                            .contains(operationId)) {
                throw new IllegalArgumentException("Selected case is not a supported server payload field: " + caseId);
            }
            String fieldName = requires.path("field_name").asText();
            ServerPayloadValue fixture = serverPayloadValue(catalog, operationId, fieldName);
            int tagCode = tagCodeNamed(catalog, fieldName, version);
            TtlvItemType itemType = scalarMessageFieldType(fixture.encoding());
            byte[] wire = scalarWire(tagCode, itemType.code(), fixture.bytes());
            try (TtlvValue value = scalarMessageFieldValue(fixture.encoding(), fixture.bytes(),
                    fixture.number(), limits);
                    RawTag raw = RawTag.fromRaw(tagCode);
                    Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, value, limits);
                    var itemView = item.view();
                    var itemTag = itemView.tag();
                    var itemValue = itemView.value()) {
                assertEquals(tagCode, itemTag.raw());
                assertScalarMessageFieldValue(itemValue, itemType, fixture.bytes(), fixture.number());
            }
            try (TtlvItem decoded = TtlvCodec.decode(wire);
                    var decodedView = decoded.view();
                    var decodedTag = decodedView.tag();
                    var decodedValue = decodedView.value()) {
                assertEquals(tagCode, decodedTag.raw());
                assertScalarMessageFieldValue(decodedValue, itemType, fixture.bytes(), fixture.number());
            }
            String targetId = requires.path("normative_target_id").asText();
            for (String evidenceTargetId : List.of(targetId, operationId)) {
                emitRoute(sink, caseId, evidenceTargetId, "java.generic-ttlv.ttlv_item.create",
                        "org.kmipkit.ttlv.TtlvItem.create", "construct", wire);
                emitRoute(sink, caseId, evidenceTargetId,
                        "java.generic-ttlv.function.ttlv_item_view_from_item",
                        "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
                emitRoute(sink, caseId, evidenceTargetId, "java.generic-ttlv.ttlv_codec.decode",
                        "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire);
            }
            emitted.add(caseId);
        }
        if (!emitted.equals(selectedCaseIds)) {
            throw new IllegalArgumentException("Selected Query payload cases were not executed for " + version);
        }
    }

    /** Executes generic TTLV routes for operation payload correlation values. */
    static void executeCorrelationPayloadScalarFields(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        Set<String> emitted = new java.util.HashSet<>();
        CodecLimits limits = CodecLimits.defaults();
        byte[] correlation = "kmipkit-correlation-fixture".getBytes(StandardCharsets.US_ASCII);
        List<String> genericRoutes = List.of("java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item", "java.generic-ttlv.ttlv_codec.decode");
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId)
                    || !"local_model".equals(testCase.path("execution").path("mode").asText())
                    || !applies(testCase, version)) continue;
            JsonNode requires = testCase.path("requires");
            String fieldName = requires.path("field_name").asText();
            String operationId = requires.path("operation_id").asText();
            if (!"operation_payload_field".equals(requires.path("target_kind").asText())
                    || !"Asynchronous Correlation Value".equals(fieldName)
                    || !Set.of("request", "response").contains(requires.path("payload_role").asText())) {
                throw new IllegalArgumentException("Selected case is not a correlation payload field: "
                        + caseId);
            }
            String targetId = requires.path("normative_target_id").asText();
            int tagCode = tagCodeNamed(catalog, fieldName, version);
            byte[] wire = scalarWire(tagCode, TtlvItemType.ByteString.code(), correlation);
            try (TtlvValue value = TtlvValue.ByteString(correlation, limits);
                    RawTag raw = RawTag.fromRaw(tagCode);
                    Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, value, limits);
                    var view = item.view();
                    var viewedTag = view.tag();
                    var viewedValue = view.value()) {
                assertEquals(tagCode, viewedTag.raw());
                assertEquals(TtlvItemType.ByteString, view.itemType());
                assertArrayEquals(correlation, viewedValue.bytes());
            }
            try (TtlvItem decoded = TtlvCodec.decode(wire);
                    var view = decoded.view();
                    var viewedTag = view.tag();
                    var viewedValue = view.value()) {
                assertEquals(tagCode, viewedTag.raw());
                assertEquals(TtlvItemType.ByteString, view.itemType());
                assertArrayEquals(correlation, viewedValue.bytes());
            }
            for (String evidenceTargetId : List.of(targetId, operationId)) {
                emitRoute(sink, caseId, evidenceTargetId, genericRoutes.get(0),
                        "org.kmipkit.ttlv.TtlvItem.create", "construct", wire);
                emitRoute(sink, caseId, evidenceTargetId, genericRoutes.get(1),
                        "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
                emitRoute(sink, caseId, evidenceTargetId, genericRoutes.get(2),
                        "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire);
            }
            emitted.add(caseId);
        }
        if (!emitted.equals(selectedCaseIds)) {
            throw new IllegalArgumentException("Selected client correlation payload cases were not executed for "
                    + version);
        }
    }

    /** Exercises the generic structure-builder routes with the source-defined KMIP 5.1 Attributes shape. */
    static void executeAttributeStructures(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        Set<String> emitted = new java.util.HashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId) || !applies(testCase, version)) continue;
            JsonNode requires = testCase.path("requires");
            if (!"attribute_structure".equals(requires.path("target_kind").asText())
                    || !"KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-1-ATTRIBUTES"
                            .equals(requires.path("normative_target_id").asText())) {
                throw new IllegalArgumentException("Selected case is not the KMIP 5.1 Attributes structure: "
                        + caseId);
            }
            String targetId = requires.path("normative_target_id").asText();
            int outerTag = tagCodeNamed(catalog, "Attributes", version);
            byte[] name = "Comment".getBytes(StandardCharsets.UTF_8);
            byte[] value = "route-matrix-fixture".getBytes(StandardCharsets.UTF_8);
            byte[] wire = structureWire(outerTag,
                    structureWire(0x420008, concat(itemWire(0x42000a, 7, name), itemWire(0x42000b, 7, value))));

            TtlvStructure attribute = appendText(TtlvStructure.create(), 0x42000a, name, limits);
            attribute = appendText(attribute, 0x42000b, value, limits);
            TtlvValue attributeValue = TtlvValue.structure(attribute, limits);
            TtlvItem attributeItem;
            try (RawTag raw = RawTag.fromRaw(0x420008); Tag tag = raw.checked()) {
                attributeItem = TtlvItem.create(tag, attributeValue, limits);
            }
            TtlvStructure attributes = TtlvStructure.withItem(TtlvStructure.create(), attributeItem, limits);
            TtlvValue attributesValue = TtlvValue.structure(attributes, limits);
            TtlvItem item;
            try (RawTag raw = RawTag.fromRaw(outerTag); Tag tag = raw.checked()) {
                item = TtlvItem.create(tag, attributesValue, limits);
            }
            try (item;
                    var view = item.view(); var tag = view.tag(); var itemValue = view.value();
                    TtlvItem decoded = TtlvCodec.decode(wire);
                    var decodedView = decoded.view(); var decodedTag = decodedView.tag();
                    var decodedValue = decodedView.value()) {
                assertEquals(outerTag, tag.raw());
                assertEquals(TtlvItemType.Structure, view.itemType());
                try (TtlvStructureView structure = itemValue.structure()) {
                    assertEquals(1, structure.itemCount());
                }
                assertEquals(outerTag, decodedTag.raw());
                assertEquals(TtlvItemType.Structure, decodedView.itemType());
                try (TtlvStructureView structure = decodedValue.structure()) {
                    assertEquals(1, structure.itemCount());
                }
            }
            emitRoute(sink, caseId, targetId,
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.create",
                    "org.kmipkit.ttlv.TtlvStructure.create", "construct", wire);
            emitRoute(sink, caseId, targetId,
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.withitem",
                    "org.kmipkit.ttlv.TtlvStructure.withItem", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                    "org.kmipkit.ttlv.TtlvItem.create", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                    "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire);
            emitted.add(caseId);
        }
        if (!emitted.equals(selectedCaseIds)) {
            throw new IllegalArgumentException("Selected Attributes structure cases were not executed for "
                    + version);
        }
    }

    /** Exercises generic TTLV routes for the source-defined scalar children of selected Structure attributes. */
    static void executeStructureAttributeValues(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        Set<String> applicable = new java.util.LinkedHashSet<>();
        Set<String> emitted = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId) || !applies(testCase, version)) continue;
            applicable.add(caseId);
            JsonNode requires = testCase.path("requires");
            if (!"Structure".equals(requires.path("attribute_policy").path("value_type").asText())) {
                throw new IllegalArgumentException("Selected attribute case is not Structure-valued: " + caseId);
            }
            String targetId = requires.path("normative_target_id").asText();
            String attributeName = requires.path("attribute").asText();
            List<StructureAttributeMember> members = structureAttributeMembers(attributeName);
            int attributeTag = tagCodeNamed(catalog, attributeName, version);
            TtlvStructure structure = TtlvStructure.create();
            List<Integer> memberTags = new ArrayList<>();
            List<TtlvItemType> memberTypes = new ArrayList<>();
            List<byte[]> memberValues = new ArrayList<>();
            byte[] children = new byte[0];
            try {
                for (StructureAttributeMember member : members) {
                    String encoding = structureMemberEncoding(catalog, targetId, member.name(), version);
                    int tagCode = tagCodeNamed(catalog, member.name(), version);
                    TtlvItemType itemType;
                    byte[] bytes;
                    TtlvValue value;
                    if (encoding.contains("Enumeration") && !encoding.contains("Text String")) {
                        long enumeration = representativeEnumerationValue(catalog, member.name(), version);
                        itemType = TtlvItemType.Enumeration;
                        bytes = ByteBuffer.allocate(Integer.BYTES).putInt(Math.toIntExact(enumeration)).array();
                        value = TtlvValue.Enumeration(enumeration);
                    } else if (encoding.contains("Text String")) {
                        itemType = TtlvItemType.TextString;
                        bytes = member.textValue().getBytes(StandardCharsets.UTF_8);
                        value = TtlvValue.TextString(bytes, limits);
                    } else {
                        throw new IllegalArgumentException("No source-backed scalar fixture for "
                                + attributeName + "." + member.name() + ": " + encoding);
                    }
                    structure = append(structure, tagCode, value);
                    children = concat(children, itemWire(tagCode, itemType.code(), bytes));
                    memberTags.add(tagCode);
                    memberTypes.add(itemType);
                    memberValues.add(bytes);
                }
            } catch (RuntimeException failure) {
                structure.close();
                throw failure;
            }

            byte[] wire = structureWire(attributeTag, children);
            TtlvValue outerValue = TtlvValue.structure(structure, limits);
            TtlvItem item;
            try (RawTag raw = RawTag.fromRaw(attributeTag); Tag tag = raw.checked()) {
                item = TtlvItem.create(tag, outerValue, limits);
            }
            try (item) {
                assertStructureAttributeItem(item, attributeTag, memberTags, memberTypes, memberValues);
            }
            try (TtlvItem decoded = TtlvCodec.decode(wire)) {
                assertStructureAttributeItem(decoded, attributeTag, memberTags, memberTypes, memberValues);
            }
            emitRoute(sink, caseId, targetId,
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure",
                    "org.kmipkit.ttlv.TtlvValue.structure", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                    "org.kmipkit.ttlv.TtlvItem.create", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                    "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire);
            emitted.add(caseId);
        }
        if (!emitted.equals(applicable)) {
            throw new IllegalArgumentException("Selected structure-valued attribute cases were not executed for "
                    + version);
        }
    }

    static Set<String> missingStructureAttributeValueCaseIds(JsonNode catalog, JsonNode events, String version) {
        List<String> requiredRoutes = List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode");
        Set<String> missing = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!STRUCTURE_ATTRIBUTE_VALUE_CASE_IDS.contains(caseId) || !applies(testCase, version)) continue;
            String targetId = testCase.path("requires").path("normative_target_id").asText();
            int present = 0;
            for (String route : requiredRoutes) {
                boolean found = false;
                for (JsonNode event : events) {
                    if (caseId.equals(event.path("case_id").asText())
                            && targetId.equals(event.path("target_id").asText())
                            && route.equals(event.path("route_id").asText())
                            && version.equals(event.path("protocol_version").asText())
                            && "TTLV".equals(event.path("encoding").asText())
                            && event.path("transport").isNull()) {
                        found = true;
                        break;
                    }
                }
                if (found) present++;
            }
            if (present == 0) missing.add(caseId);
            else assertEquals(requiredRoutes.size(), present,
                    "Structure attribute route evidence must be complete before reuse for " + caseId);
        }
        return missing;
    }

    /** Exercises generic TTLV construction and decoding for catalogued Text String Attributes. */
    static void executeTextAttributeValues(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        Set<String> applicable = new java.util.LinkedHashSet<>();
        Set<String> emitted = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId) || !applies(testCase, version)) continue;
            applicable.add(caseId);
            JsonNode requires = testCase.path("requires");
            if (!"Text String".equals(requires.path("attribute_policy").path("value_type").asText())) {
                throw new IllegalArgumentException("Selected attribute case is not Text String-valued: " + caseId);
            }
            String targetId = requires.path("normative_target_id").asText();
            String attributeName = requires.path("attribute").asText();
            int attributeTag = tagCodeNamed(catalog, attributeName, version);
            byte[] textBytes = textAttributeFixture(attributeName).getBytes(StandardCharsets.UTF_8);
            byte[] wire = itemWire(attributeTag, TtlvItemType.TextString.code(), textBytes);
            try (TtlvValue text = TtlvValue.TextString(textBytes, limits);
                    RawTag raw = RawTag.fromRaw(attributeTag); Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, text, limits)) {
                assertTextAttributeItem(item, attributeTag, textBytes);
            }
            try (TtlvItem decoded = TtlvCodec.decode(wire)) {
                assertTextAttributeItem(decoded, attributeTag, textBytes);
            }
            emitRoute(sink, caseId, targetId,
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.textstring",
                    "org.kmipkit.ttlv.TtlvValue.TextString", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                    "org.kmipkit.ttlv.TtlvItem.create", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                    "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire);
            emitted.add(caseId);
        }
        if (!emitted.equals(applicable)) {
            throw new IllegalArgumentException("Selected Text String Attribute cases were not executed for "
                    + version);
        }
    }

    static Set<String> missingTextAttributeCaseIds(JsonNode catalog, JsonNode events, String version) {
        List<String> requiredRoutes = List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.textstring",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode");
        Set<String> missing = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!TEXT_ATTRIBUTE_VALUE_CASE_IDS.contains(caseId) || !applies(testCase, version)) continue;
            String targetId = testCase.path("requires").path("normative_target_id").asText();
            int present = 0;
            for (String route : requiredRoutes) {
                boolean found = false;
                for (JsonNode event : events) {
                    if (caseId.equals(event.path("case_id").asText())
                            && targetId.equals(event.path("target_id").asText())
                            && route.equals(event.path("route_id").asText())
                            && version.equals(event.path("protocol_version").asText())
                            && "TTLV".equals(event.path("encoding").asText())
                            && event.path("transport").isNull()) {
                        found = true;
                        break;
                    }
                }
                if (found) present++;
            }
            if (present == 0) missing.add(caseId);
            else assertEquals(requiredRoutes.size(), present,
                    "Text String Attribute route evidence must be complete before reuse for " + caseId);
        }
        return missing;
    }

    /** Exercises generic TTLV construction and decoding for catalogued Boolean Attributes. */
    static void executeBooleanAttributeValues(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        Set<String> applicable = new java.util.LinkedHashSet<>();
        Set<String> emitted = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId) || !applies(testCase, version)) continue;
            applicable.add(caseId);
            JsonNode requires = testCase.path("requires");
            if (!"Boolean".equals(requires.path("attribute_policy").path("value_type").asText())) {
                throw new IllegalArgumentException("Selected attribute case is not Boolean-valued: " + caseId);
            }
            String targetId = requires.path("normative_target_id").asText();
            String attributeName = requires.path("attribute").asText();
            int attributeTag = tagCodeNamed(catalog, attributeName, version);
            byte[] wire = booleanWire(attributeTag, true);
            try (TtlvValue value = TtlvValue.Boolean(true);
                    RawTag raw = RawTag.fromRaw(attributeTag); Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults())) {
                assertBooleanAttributeItem(item, attributeTag);
            }
            try (TtlvItem decoded = TtlvCodec.decode(wire)) {
                assertBooleanAttributeItem(decoded, attributeTag);
            }
            emitRoute(sink, caseId, targetId,
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.boolean",
                    "org.kmipkit.ttlv.TtlvValue.Boolean", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                    "org.kmipkit.ttlv.TtlvItem.create", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                    "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire);
            emitted.add(caseId);
        }
        if (!emitted.equals(applicable)) {
            throw new IllegalArgumentException("Selected Boolean Attribute cases were not executed for " + version);
        }
    }

    static Set<String> missingBooleanAttributeCaseIds(JsonNode catalog, JsonNode events, String version) {
        List<String> requiredRoutes = List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.boolean",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode");
        Set<String> missing = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!BOOLEAN_ATTRIBUTE_VALUE_CASE_IDS.contains(caseId) || !applies(testCase, version)) continue;
            String targetId = testCase.path("requires").path("normative_target_id").asText();
            int present = 0;
            for (String route : requiredRoutes) {
                boolean found = false;
                for (JsonNode event : events) {
                    if (caseId.equals(event.path("case_id").asText())
                            && targetId.equals(event.path("target_id").asText())
                            && route.equals(event.path("route_id").asText())
                            && version.equals(event.path("protocol_version").asText())
                            && "TTLV".equals(event.path("encoding").asText())
                            && event.path("transport").isNull()) {
                        found = true;
                        break;
                    }
                }
                if (found) present++;
            }
            if (present == 0) missing.add(caseId);
            else assertEquals(requiredRoutes.size(), present,
                    "Boolean Attribute route evidence must be complete before reuse for " + caseId);
        }
        return missing;
    }

    /** Exercises generic TTLV construction and decoding for catalogued Date-Time Attributes. */
    static void executeDateTimeAttributeValues(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        Set<String> applicable = new java.util.LinkedHashSet<>();
        Set<String> emitted = new java.util.LinkedHashSet<>();
        long dateTime = 1_700_000_000L;
        byte[] valueBytes = ByteBuffer.allocate(Long.BYTES).putLong(dateTime).array();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId) || !applies(testCase, version)) continue;
            applicable.add(caseId);
            JsonNode requires = testCase.path("requires");
            String valueType = requires.path("attribute_policy").path("value_type").asText();
            if (!Set.of("Date-Time", "Date Time").contains(valueType)) {
                throw new IllegalArgumentException("Selected attribute case is not Date-Time-valued: " + caseId);
            }
            String targetId = requires.path("normative_target_id").asText();
            String attributeName = requires.path("attribute").asText();
            int attributeTag = tagCodeNamed(catalog, attributeName, version);
            byte[] wire = itemWire(attributeTag, TtlvItemType.DateTime.code(), valueBytes);
            try (TtlvValue value = TtlvValue.DateTime(dateTime);
                    RawTag raw = RawTag.fromRaw(attributeTag); Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults())) {
                assertDateTimeAttributeItem(item, attributeTag, dateTime);
            }
            try (TtlvItem decoded = TtlvCodec.decode(wire)) {
                assertDateTimeAttributeItem(decoded, attributeTag, dateTime);
            }
            emitRoute(sink, caseId, targetId,
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.datetime",
                    "org.kmipkit.ttlv.TtlvValue.DateTime", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                    "org.kmipkit.ttlv.TtlvItem.create", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                    "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire);
            emitted.add(caseId);
        }
        if (!emitted.equals(applicable)) {
            throw new IllegalArgumentException("Selected Date-Time Attribute cases were not executed for "
                    + version);
        }
    }

    static Set<String> missingDateTimeAttributeCaseIds(JsonNode catalog, JsonNode events, String version) {
        List<String> requiredRoutes = List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.datetime",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode");
        Set<String> missing = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!DATE_TIME_ATTRIBUTE_VALUE_CASE_IDS.contains(caseId) || !applies(testCase, version)) continue;
            String targetId = testCase.path("requires").path("normative_target_id").asText();
            int present = 0;
            for (String route : requiredRoutes) {
                boolean found = false;
                for (JsonNode event : events) {
                    if (caseId.equals(event.path("case_id").asText())
                            && targetId.equals(event.path("target_id").asText())
                            && route.equals(event.path("route_id").asText())
                            && version.equals(event.path("protocol_version").asText())
                            && "TTLV".equals(event.path("encoding").asText())
                            && event.path("transport").isNull()) {
                        found = true;
                        break;
                    }
                }
                if (found) present++;
            }
            if (present == 0) missing.add(caseId);
            else assertEquals(requiredRoutes.size(), present,
                    "Date-Time Attribute route evidence must be complete before reuse for " + caseId);
        }
        return missing;
    }

    /** Exercises generic TTLV construction and decoding for catalogued Enumeration Attributes. */
    static void executeEnumerationAttributeValues(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        Set<String> applicable = new java.util.LinkedHashSet<>();
        Set<String> emitted = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId) || !applies(testCase, version)) continue;
            applicable.add(caseId);
            JsonNode requires = testCase.path("requires");
            if (!"Enumeration".equals(requires.path("attribute_policy").path("value_type").asText())) {
                throw new IllegalArgumentException("Selected attribute case is not Enumeration-valued: " + caseId);
            }
            String targetId = requires.path("normative_target_id").asText();
            String attributeName = requires.path("attribute").asText();
            int attributeTag = tagCodeNamed(catalog, attributeName, version);
            long enumeration = representativeEnumerationValue(catalog, attributeName, version);
            byte[] valueBytes = ByteBuffer.allocate(Integer.BYTES).putInt((int) enumeration).array();
            byte[] wire = itemWire(attributeTag, TtlvItemType.Enumeration.code(), valueBytes);
            try (TtlvValue value = TtlvValue.Enumeration(enumeration);
                    RawTag raw = RawTag.fromRaw(attributeTag); Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults())) {
                assertEnumerationAttributeItem(item, attributeTag, enumeration);
            }
            try (TtlvItem decoded = TtlvCodec.decode(wire)) {
                assertEnumerationAttributeItem(decoded, attributeTag, enumeration);
            }
            emitRoute(sink, caseId, targetId,
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                    "org.kmipkit.ttlv.TtlvValue.Enumeration", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                    "org.kmipkit.ttlv.TtlvItem.create", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                    "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire);
            emitted.add(caseId);
        }
        if (!emitted.equals(applicable)) {
            throw new IllegalArgumentException("Selected Enumeration Attribute cases were not executed for "
                    + version);
        }
    }

    static Set<String> missingEnumerationAttributeCaseIds(JsonNode catalog, JsonNode events, String version) {
        List<String> requiredRoutes = List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode");
        Set<String> missing = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!ENUMERATION_ATTRIBUTE_VALUE_CASE_IDS.contains(caseId) || !applies(testCase, version)) continue;
            String targetId = testCase.path("requires").path("normative_target_id").asText();
            int present = 0;
            for (String route : requiredRoutes) {
                boolean found = false;
                for (JsonNode event : events) {
                    if (caseId.equals(event.path("case_id").asText())
                            && targetId.equals(event.path("target_id").asText())
                            && route.equals(event.path("route_id").asText())
                            && version.equals(event.path("protocol_version").asText())
                            && "TTLV".equals(event.path("encoding").asText())
                            && event.path("transport").isNull()) {
                        found = true;
                        break;
                    }
                }
                if (found) present++;
            }
            if (present == 0) missing.add(caseId);
            else assertEquals(requiredRoutes.size(), present,
                    "Enumeration Attribute route evidence must be complete before reuse for " + caseId);
        }
        return missing;
    }

    /** Exercises generic TTLV construction and decoding for catalogued scalar Attributes. */
    static void executeScalarAttributeValues(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        Set<String> applicable = new java.util.LinkedHashSet<>();
        Set<String> emitted = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId) || !applies(testCase, version)) continue;
            applicable.add(caseId);
            JsonNode requires = testCase.path("requires");
            String valueType = requires.path("attribute_policy").path("value_type").asText();
            String targetId = requires.path("normative_target_id").asText();
            String attributeName = requires.path("attribute").asText();
            int attributeTag = tagCodeNamed(catalog, attributeName, version);
            byte[] wire;
            String constructorRoute;
            String constructorSymbol;
            String action;
            switch (valueType) {
                case "Integer" -> {
                    int value = integerAttributeFixture(attributeName);
                    wire = itemWire(attributeTag, TtlvItemType.Integer.code(),
                            ByteBuffer.allocate(Integer.BYTES).putInt(value).array());
                    constructorRoute = "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.integer";
                    constructorSymbol = "org.kmipkit.ttlv.TtlvValue.Integer";
                    try (TtlvValue ttlvValue = TtlvValue.Integer(value);
                            RawTag raw = RawTag.fromRaw(attributeTag); Tag tag = raw.checked();
                            TtlvItem item = TtlvItem.create(tag, ttlvValue, CodecLimits.defaults())) {
                        assertScalarAttributeItem(item, attributeTag, TtlvItemType.Integer, value);
                    }
                    try (TtlvItem decoded = TtlvCodec.decode(wire)) {
                        assertScalarAttributeItem(decoded, attributeTag, TtlvItemType.Integer, value);
                    }
                }
                case "Interval" -> {
                    long value = 60;
                    wire = itemWire(attributeTag, TtlvItemType.Interval.code(),
                            ByteBuffer.allocate(Integer.BYTES).putInt(Math.toIntExact(value)).array());
                    constructorRoute = "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.interval";
                    constructorSymbol = "org.kmipkit.ttlv.TtlvValue.Interval";
                    try (TtlvValue ttlvValue = TtlvValue.Interval(value);
                            RawTag raw = RawTag.fromRaw(attributeTag); Tag tag = raw.checked();
                            TtlvItem item = TtlvItem.create(tag, ttlvValue, CodecLimits.defaults())) {
                        assertScalarAttributeItem(item, attributeTag, TtlvItemType.Interval, value);
                    }
                    try (TtlvItem decoded = TtlvCodec.decode(wire)) {
                        assertScalarAttributeItem(decoded, attributeTag, TtlvItemType.Interval, value);
                    }
                }
                case "Byte String" -> {
                    byte[] value = new byte[] {0x01, 0x23, 0x45, 0x67};
                    wire = itemWire(attributeTag, TtlvItemType.ByteString.code(), value);
                    constructorRoute = "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.bytestring";
                    constructorSymbol = "org.kmipkit.ttlv.TtlvValue.ByteString";
                    try (TtlvValue ttlvValue = TtlvValue.ByteString(value, CodecLimits.defaults());
                            RawTag raw = RawTag.fromRaw(attributeTag); Tag tag = raw.checked();
                            TtlvItem item = TtlvItem.create(tag, ttlvValue, CodecLimits.defaults())) {
                        assertByteStringAttributeItem(item, attributeTag, value);
                    }
                    try (TtlvItem decoded = TtlvCodec.decode(wire)) {
                        assertByteStringAttributeItem(decoded, attributeTag, value);
                    }
                }
                default -> throw new IllegalArgumentException("Selected attribute case is not scalar-valued: "
                        + caseId);
            }
            emitRoute(sink, caseId, targetId, constructorRoute, constructorSymbol, "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                    "org.kmipkit.ttlv.TtlvItem.create", "construct", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                    "org.kmipkit.ttlv.TtlvCodec.decode", "decode", wire);
            emitted.add(caseId);
        }
        if (!emitted.equals(applicable)) {
            throw new IllegalArgumentException("Selected scalar Attribute cases were not executed for " + version);
        }
    }

    static Set<String> missingScalarAttributeCaseIds(JsonNode catalog, JsonNode events, String version) {
        Set<String> missing = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!SCALAR_ATTRIBUTE_VALUE_CASE_IDS.contains(caseId) || !applies(testCase, version)) continue;
            String valueType = testCase.path("requires").path("attribute_policy").path("value_type").asText();
            String constructorRoute = switch (valueType) {
                case "Integer" -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.integer";
                case "Interval" -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.interval";
                case "Byte String" -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.bytestring";
                default -> throw new IllegalArgumentException("Selected attribute case is not scalar-valued: "
                        + caseId);
            };
            List<String> requiredRoutes = List.of(constructorRoute, "java.generic-ttlv.ttlv_item.create",
                    "java.generic-ttlv.function.ttlv_item_view_from_item", "java.generic-ttlv.ttlv_codec.decode");
            String targetId = testCase.path("requires").path("normative_target_id").asText();
            int present = 0;
            for (String route : requiredRoutes) {
                boolean found = false;
                for (JsonNode event : events) {
                    if (caseId.equals(event.path("case_id").asText())
                            && targetId.equals(event.path("target_id").asText())
                            && route.equals(event.path("route_id").asText())
                            && version.equals(event.path("protocol_version").asText())
                            && "TTLV".equals(event.path("encoding").asText())
                            && event.path("transport").isNull()) {
                        found = true;
                        break;
                    }
                }
                if (found) present++;
            }
            if (present == 0) missing.add(caseId);
            else assertEquals(requiredRoutes.size(), present,
                    "Scalar Attribute route evidence must be complete before reuse for " + caseId);
        }
        return missing;
    }

    private static int integerAttributeFixture(String attributeName) {
        return switch (attributeName) {
            case "Certificate Length", "Cryptographic Length" -> 256;
            case "Cryptographic Usage Mask", "Protection Storage Mask", "Rotate Generation" -> 1;
            default -> throw new IllegalArgumentException("No scalar Integer fixture for " + attributeName);
        };
    }

    private static void assertScalarAttributeItem(TtlvItem item, int attributeTag, TtlvItemType itemType,
            long expectedValue) {
        try (TtlvItemView view = item.view(); Tag tag = view.tag(); TtlvValueView value = view.value()) {
            assertEquals(attributeTag, tag.raw());
            assertEquals(itemType, view.itemType());
            if (itemType == TtlvItemType.Integer) assertEquals((int) expectedValue, value.integer());
            else assertEquals(expectedValue, value.interval());
        }
    }

    private static void assertByteStringAttributeItem(TtlvItem item, int attributeTag, byte[] expectedValue) {
        try (TtlvItemView view = item.view(); Tag tag = view.tag(); TtlvValueView value = view.value()) {
            assertEquals(attributeTag, tag.raw());
            assertEquals(TtlvItemType.ByteString, view.itemType());
            assertArrayEquals(expectedValue, value.bytes());
        }
    }

    private static String textAttributeFixture(String attributeName) {
        return switch (attributeName) {
            case "Comment" -> "KMIPKit generic TTLV fixture";
            case "Contact Information" -> "kmipkit@example.invalid";
            case "Description" -> "Java route validator fixture";
            case "Object Group" -> "kmipkit-java-validator-fixture";
            case "PKCS#12 Friendly Name" -> "KMIPKit Java fixture";
            default -> throw new IllegalArgumentException("No source-backed text fixture for " + attributeName);
        };
    }

    private static List<StructureAttributeMember> structureAttributeMembers(String attributeName) {
        return switch (attributeName) {
            case "Alternative Name" -> List.of(
                    new StructureAttributeMember("Alternative Name Value", "https://kmipkit.example/fixture"),
                    new StructureAttributeMember("Alternative Name Type", null));
            case "Application Specific Information" -> List.of(
                    new StructureAttributeMember("Application Namespace", "org.kmipkit.fixture"),
                    new StructureAttributeMember("Application Data", "route-matrix-fixture"));
            case "Key Value Location" -> List.of(
                    new StructureAttributeMember("Key Value Location Value", "test://key-material"),
                    new StructureAttributeMember("Key Value Location Type", null));
            case "Link" -> List.of(
                    new StructureAttributeMember("Link Type", null),
                    new StructureAttributeMember("Linked Object Identifier", "linked-object-7"));
            case "Name" -> List.of(
                    new StructureAttributeMember("Name Value", "KMIPKit route fixture"),
                    new StructureAttributeMember("Name Type", null));
            case "Revocation Reason" -> List.of(
                    new StructureAttributeMember("Revocation Reason Code", null),
                    new StructureAttributeMember("Revocation Message", "controlled fixture"));
            case "Rotate Name" -> List.of(
                    new StructureAttributeMember("Rotate Name Value", "KMIPKit route fixture"),
                    new StructureAttributeMember("Rotate Name Type", null));
            default -> throw new IllegalArgumentException("No source-backed Structure attribute recipe for: "
                    + attributeName);
        };
    }

    private static String structureMemberEncoding(JsonNode catalog, String parentTargetId, String memberName,
            String version) {
        String encoding = null;
        for (JsonNode target : catalog.path("normative_targets")) {
            if (!"structure_member".equals(target.path("kind").asText())
                    || !memberName.equals(target.path("name").asText())
                    || !hasParent(target, parentTargetId) || !supportsVersion(target, version)) continue;
            if (encoding != null) {
                throw new IllegalArgumentException("Structure member is ambiguous in the source inventory: "
                        + parentTargetId + "." + memberName);
            }
            encoding = target.path("source_encoding").asText();
        }
        if (encoding == null || encoding.isBlank()) {
            throw new IllegalArgumentException("Structure member has no source encoding: "
                    + parentTargetId + "." + memberName);
        }
        return encoding;
    }

    private static void assertStructureAttributeItem(TtlvItem item, int attributeTag, List<Integer> memberTags,
            List<TtlvItemType> memberTypes, List<byte[]> memberValues) {
        try (TtlvItemView view = item.view(); Tag tag = view.tag(); TtlvValueView value = view.value();
                TtlvStructureView structure = value.structure()) {
            assertEquals(attributeTag, tag.raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertEquals(memberTags.size(), structure.itemCount());
            for (int i = 0; i < memberTags.size(); i++) {
                try (TtlvItemView member = structure.itemAt(i);
                        Tag memberTag = member.tag(); TtlvValueView memberValue = member.value()) {
                    assertEquals(memberTags.get(i), memberTag.raw());
                    assertEquals(memberTypes.get(i), member.itemType());
                    if (memberTypes.get(i) == TtlvItemType.Enumeration) {
                        assertEquals(Integer.toUnsignedLong(ByteBuffer.wrap(memberValues.get(i)).getInt()),
                                memberValue.enumeration());
                    } else {
                        assertArrayEquals(memberValues.get(i), memberValue.bytes());
                    }
                }
            }
        }
    }

    private static void assertTextAttributeItem(TtlvItem item, int attributeTag, byte[] expectedText) {
        try (TtlvItemView view = item.view(); Tag tag = view.tag(); TtlvValueView value = view.value()) {
            assertEquals(attributeTag, tag.raw());
            assertEquals(TtlvItemType.TextString, view.itemType());
            assertArrayEquals(expectedText, value.bytes());
        }
    }

    private static void assertBooleanAttributeItem(TtlvItem item, int attributeTag) {
        try (TtlvItemView view = item.view(); Tag tag = view.tag(); TtlvValueView value = view.value()) {
            assertEquals(attributeTag, tag.raw());
            assertEquals(TtlvItemType.Boolean, view.itemType());
            assertEquals(true, value.booleanValue());
        }
    }

    private static void assertDateTimeAttributeItem(TtlvItem item, int attributeTag, long expected) {
        try (TtlvItemView view = item.view(); Tag tag = view.tag(); TtlvValueView value = view.value()) {
            assertEquals(attributeTag, tag.raw());
            assertEquals(TtlvItemType.DateTime, view.itemType());
            assertEquals(expected, value.dateTime());
        }
    }

    private static void assertEnumerationAttributeItem(TtlvItem item, int attributeTag, long expected) {
        try (TtlvItemView view = item.view(); Tag tag = view.tag(); TtlvValueView value = view.value()) {
            assertEquals(attributeTag, tag.raw());
            assertEquals(TtlvItemType.Enumeration, view.itemType());
            assertEquals(expected, value.enumeration());
        }
    }

    private static TtlvStructure appendText(TtlvStructure structure, int tagCode, byte[] value,
            CodecLimits limits) {
        TtlvItem item;
        try (TtlvValue text = TtlvValue.TextString(value, limits);
                RawTag raw = RawTag.fromRaw(tagCode); Tag tag = raw.checked()) {
            item = TtlvItem.create(tag, text, limits);
        }
        return TtlvStructure.withItem(structure, item, limits);
    }

    private static ServerPayloadValue serverPayloadValue(JsonNode catalog, String operationId, String fieldName) {
        if ("KMIPKIT-ELEM-OP-S2C-NOTIFY".equals(operationId) && "Unique Identifier".equals(fieldName)) {
            return new ServerPayloadValue("Text String", 0,
                    "kmipkit-notify-validator".getBytes(StandardCharsets.US_ASCII));
        }
        if (!"KMIPKIT-ELEM-OP-S2C-QUERY".equals(operationId)) {
            throw new IllegalArgumentException("No source-backed scalar server payload fixture: " + operationId
                    + " / " + fieldName);
        }
        return switch (fieldName) {
            case "Query Function" -> queryEnumeration(catalog, "QUERY-FUNCTION", "Query Operations");
            case "Object Type" -> queryEnumeration(catalog, "OBJECT-TYPE", "Symmetric Key");
            case "Attestation Type" -> queryEnumeration(catalog, "ATTESTATION-TYPE", "TPM Quote");
            case "Client Registration Method" -> queryEnumeration(catalog,
                    "CLIENT-REGISTRATION-METHOD", "Client Generated");
            case "Vendor Identification" -> new ServerPayloadValue("Text String", 0,
                    "KMIPKit Java validator".getBytes(StandardCharsets.US_ASCII));
            default -> throw new IllegalArgumentException("No source-backed scalar Query payload fixture: "
                    + fieldName);
        };
    }

    private static ServerPayloadValue queryEnumeration(JsonNode catalog, String enumType, String valueName) {
        long number = enumValue(catalog, enumType, valueName);
        byte[] bytes = ByteBuffer.allocate(Integer.BYTES).putInt(Math.toIntExact(number)).array();
        return new ServerPayloadValue("Enumeration", number, bytes);
    }

    private static long enumValue(JsonNode catalog, String enumType, String valueName) {
        String parentId = "KMIPKIT-ELEM-ENUMERATION-" + enumType;
        JsonNode match = null;
        for (JsonNode target : catalog.path("normative_targets")) {
            if (!"enumeration_value".equals(target.path("kind").asText())
                    || !valueName.equals(target.path("name").asText())
                    || !"assigned".equals(target.path("allocation").asText())) continue;
            boolean belongsToType = false;
            for (JsonNode parent : target.path("parent_element_ids")) {
                if (parentId.equals(parent.asText())) belongsToType = true;
            }
            if (!belongsToType) continue;
            if (match != null) throw new IllegalArgumentException("Duplicate assigned enumeration value: "
                    + enumType + " / " + valueName);
            match = target;
        }
        if (match == null) throw new IllegalArgumentException("Missing assigned enumeration value: "
                + enumType + " / " + valueName);
        return Long.parseUnsignedLong(match.path("wire_value").asText().replaceFirst("^0[xX]", ""), 16);
    }

    private record ServerPayloadValue(String encoding, long number, byte[] bytes) { }

    /**
     * Executes the repeatable Any object-attribute member with a Name attribute.
     * The Name structure follows OASIS KMIP 2.0/2.1 §4.32; the six member cases
     * are defined by §§5.1–5.7, and the Name tags by §§11.54/11.56.
     */
    static void executeAnyObjectAttributeMembers(JsonNode catalog, String version, Set<String> selectedCaseIds,
            EvidenceSink sink) throws Exception {
        Set<String> emitted = new java.util.HashSet<>();
        CodecLimits limits = CodecLimits.defaults();
        int nameTag = tagCodeNamed(catalog, "Name", version);
        int nameTypeTag = tagCodeNamed(catalog, "Name Type", version);
        int nameValueTag = tagCodeNamed(catalog, "Name Value", version);
        byte[] nameBytes = "kmipkit-object-attribute-fixture".getBytes(StandardCharsets.UTF_8);
        byte[] nameTypeWire = scalarWire(nameTypeTag, TtlvItemType.Enumeration.code(), 1);
        byte[] nameValueWire = itemWire(nameValueTag, TtlvItemType.TextString.code(), nameBytes);
        byte[] attributeWire = structureWire(nameTag, concat(nameTypeWire, nameValueWire));

        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            if (!selectedCaseIds.contains(caseId)
                    || !"local_model".equals(testCase.path("execution").path("mode").asText())
                    || !applies(testCase, version)) continue;
            JsonNode requires = testCase.path("requires");
            String targetId = requires.path("normative_target_id").asText();
            JsonNode target = findTarget(catalog, targetId);
            if (!"structure_member".equals(requires.path("target_kind").asText())
                    || !target.path("name").asText().startsWith("Any attribute in ")
                    || !target.path("source_encoding").asText().startsWith("Any")) {
                throw new IllegalArgumentException("Selected case is not a repeatable Any object attribute: " + caseId);
            }

            try (TtlvStructure empty = TtlvStructure.create();
                    TtlvValue typeValue = TtlvValue.Enumeration(1);
                    TtlvStructure withNameType = append(empty, nameTypeTag, typeValue);
                    TtlvValue nameValue = TtlvValue.TextString(nameBytes, limits);
                    TtlvStructure nameAttribute = append(withNameType, nameValueTag, nameValue);
                    TtlvValue attributeValue = TtlvValue.structure(nameAttribute, limits);
                    RawTag raw = RawTag.fromRaw(nameTag);
                    Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, attributeValue, limits);
                    var itemView = item.view();
                    var itemTag = itemView.tag();
                    var itemValue = itemView.value();
                    TtlvItem decoded = TtlvCodec.decode(attributeWire);
                    var decodedView = decoded.view();
                    var decodedTag = decodedView.tag();
                    var decodedValue = decodedView.value()) {
                assertEquals(nameTag, itemTag.raw());
                assertEquals(TtlvItemType.Structure, itemView.itemType());
                assertEquals(nameTag, decodedTag.raw());
                assertEquals(TtlvItemType.Structure, decodedView.itemType());
                try (TtlvStructureView structure = decodedValue.structure()) {
                    assertEquals(2, structure.itemCount());
                }
                try (TtlvStructureView structure = itemValue.structure()) {
                    assertEquals(2, structure.itemCount());
                }
            }

        try (TtlvStructure emptyAttributes = TtlvStructure.create()) {
            try (TtlvStructureView absentView = emptyAttributes.view()) {
                assertEquals(0, absentView.itemCount());
            }
            try (TtlvStructure emptyName = TtlvStructure.create();
                    TtlvValue typeValue = TtlvValue.Enumeration(1);
                    TtlvStructure withNameType = append(emptyName, nameTypeTag, typeValue);
                    TtlvValue nameValue = TtlvValue.TextString(nameBytes, limits);
                    TtlvStructure nameAttribute = append(withNameType, nameValueTag, nameValue);
                    TtlvValue attributeValue = TtlvValue.structure(nameAttribute, limits);
                    RawTag raw = RawTag.fromRaw(nameTag); Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, attributeValue, limits);
                    TtlvStructure once = TtlvStructure.withItem(emptyAttributes, item, limits)) {
                try (TtlvStructureView onceView = once.view()) {
                    assertEquals(1, onceView.itemCount());
                }
                try (TtlvItem repeatedItem = TtlvCodec.decode(attributeWire);
                        TtlvStructure twice = TtlvStructure.withItem(once, repeatedItem, limits);
                        TtlvStructureView repeatedView = twice.view()) {
                    assertEquals(2, repeatedView.itemCount());
                }
            }
        }
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                    "org.kmipkit.ttlv.TtlvItem.create", "construct", attributeWire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "org.kmipkit.ttlv.TtlvItem.view", "decode", attributeWire);
            emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                    "org.kmipkit.ttlv.TtlvCodec.decode", "decode", attributeWire);
            emitted.add(caseId);
        }
        if (!emitted.equals(selectedCaseIds)) {
            throw new IllegalArgumentException("Selected Any object attribute cases were not executed for " + version);
        }
    }

    private static int tagCodeNamed(JsonNode catalog, String name, String version) {
        String normalizedName = normalizeName(name);
        int tagCode = -1;
        int matches = 0;
        for (JsonNode candidate : catalog.path("normative_targets")) {
            if (!"tag".equals(candidate.path("kind").asText())
                    || !normalizedName.equals(normalizeName(candidate.path("name").asText()))
                    || !supportsVersion(candidate, version)) continue;
            if (!"assigned".equals(candidate.path("allocation").asText())) {
                throw new IllegalArgumentException("Fixture Tag is not assigned: " + name);
            }
            tagCode = Integer.parseUnsignedInt(
                    candidate.path("wire_value").asText().replaceFirst("^0[xX]", ""), 16);
            matches++;
        }
        if (matches != 1) throw new IllegalArgumentException("Expected one assigned fixture Tag: " + name);
        return tagCode;
    }

    private static JsonNode findTarget(JsonNode catalog, String targetId) {
        for (JsonNode target : catalog.path("normative_targets")) {
            if (targetId.equals(target.path("target_id").asText())) return target;
        }
        throw new IllegalArgumentException("Missing catalogued message field target: " + targetId);
    }

    /** Resolves a shared §8.2 field's scalar encoding from its version-specific §9.x field definition. */
    static String scalarMessageFieldEncoding(JsonNode catalog, String targetId, String version) {
        JsonNode target = findTarget(catalog, targetId);
        String encoding = target.path("source_encoding").asText();
        if (!encoding.isEmpty()) return encoding;

        String name = normalizeName(target.path("name").asText());
        Set<String> siblingEncodings = new java.util.LinkedHashSet<>();
        for (JsonNode candidate : catalog.path("normative_targets")) {
            if (!"message_field".equals(candidate.path("kind").asText())
                    || !name.equals(normalizeName(candidate.path("name").asText()))
                    || !supportsVersion(candidate, version)) continue;
            String siblingEncoding = candidate.path("source_encoding").asText();
            if (!siblingEncoding.isEmpty()) siblingEncodings.add(siblingEncoding);
        }
        if (siblingEncodings.size() > 1) {
            throw new IllegalArgumentException("Shared message field has conflicting versioned encodings: "
                    + targetId);
        }
        return siblingEncodings.stream().findFirst().orElse("");
    }

    private static int messageFieldTag(JsonNode catalog, String targetId, String version) {
        JsonNode target = findTarget(catalog, targetId);
        String name = normalizeName(target.path("name").asText());
        int tagCode = -1;
        int matches = 0;
        for (JsonNode candidate : catalog.path("normative_targets")) {
            if (!"tag".equals(candidate.path("kind").asText())
                    || !name.equals(normalizeName(candidate.path("name").asText()))
                    || !supportsVersion(candidate, version)) continue;
            if (!"assigned".equals(candidate.path("allocation").asText())) {
                throw new IllegalArgumentException("Scalar message field tag is not assigned: " + targetId);
            }
            tagCode = Integer.parseUnsignedInt(
                    candidate.path("wire_value").asText().replaceFirst("^0[xX]", ""), 16);
            matches++;
        }
        if (matches != 1) {
            throw new IllegalArgumentException("Expected one assigned Tag for scalar message field: " + targetId);
        }
        return tagCode;
    }

    private static TtlvItemType scalarMessageFieldType(String encoding) {
        return switch (encoding) {
            case "Boolean" -> TtlvItemType.Boolean;
            case "Byte String" -> TtlvItemType.ByteString;
            case "Date-Time" -> TtlvItemType.DateTime;
            case "Enumeration" -> TtlvItemType.Enumeration;
            case "Integer" -> TtlvItemType.Integer;
            case "Text String",
                    "Text String (with usage limited to alphanumeric, underscore and period – i.e. [A-Za-z0-9_.])"
                    -> TtlvItemType.TextString;
            default -> throw new IllegalArgumentException("Unsupported scalar message field encoding: " + encoding);
        };
    }

    private static byte[] scalarMessageFieldBytes(String caseId, String encoding) {
        return switch (encoding) {
            case "Boolean" -> ByteBuffer.allocate(Long.BYTES).putLong(1L).array();
            case "Byte String" -> new byte[] {0x11, 0x22, 0x33};
            case "Date-Time" -> ByteBuffer.allocate(Long.BYTES).putLong(1_700_000_000L).array();
            case "Enumeration" -> ByteBuffer.allocate(Integer.BYTES)
                    .putInt(Math.toIntExact(scalarMessageFieldNumber(caseId, encoding))).array();
            case "Integer" -> ByteBuffer.allocate(Integer.BYTES)
                    .putInt(Math.toIntExact(scalarMessageFieldNumber(caseId, encoding))).array();
            case "Text String",
                    "Text String (with usage limited to alphanumeric, underscore and period – i.e. [A-Za-z0-9_.])"
                    -> "kmipkit_route".getBytes(StandardCharsets.US_ASCII);
            default -> throw new IllegalArgumentException("Unsupported scalar message field encoding: " + encoding);
        };
    }

    private static long scalarMessageFieldNumber(String caseId, String encoding) {
        if ("Date-Time".equals(encoding)) return 1_700_000_000L;
        if ("Integer".equals(encoding)) {
            if (caseId.endsWith("protocol-version-major")) return 2;
            if (caseId.endsWith("protocol-version-minor")) return 1;
            if (caseId.endsWith("batch-count")) return 1;
            return 1_024;
        }
        if ("Enumeration".equals(encoding)) {
            if (caseId.endsWith("operation")) return 1; // Create.
            if (caseId.endsWith("result-reason")) return 2; // Response Too Large.
            if (caseId.endsWith("result-status")) return 0; // Success.
            if (caseId.endsWith("batch-error-continuation-option")) return 2; // Stop.
            if (caseId.endsWith("asynchronous-indicator")) return 1; // Mandatory.
        }
        if ("Boolean".equals(encoding)) return 1;
        return 0;
    }

    private static TtlvValue scalarMessageFieldValue(String encoding, byte[] bytes, long number,
            CodecLimits limits) {
        return switch (encoding) {
            case "Boolean" -> TtlvValue.Boolean(number != 0);
            case "Byte String" -> TtlvValue.ByteString(bytes, limits);
            case "Date-Time" -> TtlvValue.DateTime(number);
            case "Enumeration" -> TtlvValue.Enumeration(number);
            case "Integer" -> TtlvValue.Integer(Math.toIntExact(number));
            case "Text String",
                    "Text String (with usage limited to alphanumeric, underscore and period – i.e. [A-Za-z0-9_.])"
                    -> TtlvValue.TextString(bytes, limits);
            default -> throw new IllegalArgumentException("Unsupported scalar message field encoding: " + encoding);
        };
    }

    private static String scalarMessageFieldConstructorRoute(String encoding) {
        return switch (encoding) {
            case "Boolean" -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.boolean";
            case "Byte String" -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.bytestring";
            case "Date-Time" -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.datetime";
            case "Enumeration" -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration";
            case "Integer" -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.integer";
            case "Text String",
                    "Text String (with usage limited to alphanumeric, underscore and period – i.e. [A-Za-z0-9_.])"
                    -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.textstring";
            default -> throw new IllegalArgumentException("Unsupported scalar message field encoding: " + encoding);
        };
    }

    private static String scalarMessageFieldConstructor(String encoding) {
        return switch (encoding) {
            case "Boolean" -> "org.kmipkit.ttlv.TtlvValue.Boolean";
            case "Byte String" -> "org.kmipkit.ttlv.TtlvValue.ByteString";
            case "Date-Time" -> "org.kmipkit.ttlv.TtlvValue.DateTime";
            case "Enumeration" -> "org.kmipkit.ttlv.TtlvValue.Enumeration";
            case "Integer" -> "org.kmipkit.ttlv.TtlvValue.Integer";
            case "Text String",
                    "Text String (with usage limited to alphanumeric, underscore and period – i.e. [A-Za-z0-9_.])"
                    -> "org.kmipkit.ttlv.TtlvValue.TextString";
            default -> throw new IllegalArgumentException("Unsupported scalar message field encoding: " + encoding);
        };
    }

    private static void assertScalarMessageFieldValue(org.kmipkit.ttlv.TtlvValueView value,
            TtlvItemType type, byte[] bytes, long number) {
        assertEquals(type, value.itemType());
        if (type == TtlvItemType.Boolean) {
            assertEquals(number != 0, value.booleanValue());
        } else if (type == TtlvItemType.ByteString || type == TtlvItemType.TextString) {
            assertArrayEquals(bytes, value.bytes());
        } else if (type == TtlvItemType.DateTime) {
            assertEquals(number, value.dateTime());
        } else if (type == TtlvItemType.Enumeration) {
            assertEquals(number, value.enumeration());
        } else if (type == TtlvItemType.Integer) {
            assertEquals(Math.toIntExact(number), value.integer());
        } else {
            throw new IllegalArgumentException("Unsupported scalar message field type: " + type);
        }
    }

    private static byte[] scalarWire(int tag, int type, byte[] value) {
        int padding = (8 - (value.length % 8)) % 8;
        return ByteBuffer.allocate(8 + value.length + padding)
                .put((byte) (tag >>> 16)).put((byte) (tag >>> 8)).put((byte) tag)
                .put((byte) type).putInt(value.length).put(value).put(new byte[padding]).array();
    }

    /** Executes the Item Type enum values, which have no same-named standalone TTLV Tag. */
    static void executeItemTypeEnumerations(JsonNode catalog, String version, EvidenceSink sink) throws Exception {
        String caseId = "test-enum-item-type";
        int rawTag = 0x420028;
        int count = 0;
        java.util.Map<String, String> caseIdByTarget = new java.util.HashMap<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String targetId = testCase.path("requires").path("normative_target_id").asText();
            if (targetId.startsWith("KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-")) {
                String previous = caseIdByTarget.put(targetId, testCase.path("case_id").asText());
                if (previous != null) {
                    throw new IllegalArgumentException("Item Type target has duplicate test cases: " + targetId);
                }
            }
        }
        for (JsonNode testCase : catalog.path("test_cases")) {
            if (!caseId.equals(testCase.path("case_id").asText()) || !applies(testCase, version)) continue;
            for (JsonNode target : catalog.path("normative_targets")) {
                if (!"enumeration_value".equals(target.path("kind").asText())
                        || !hasParent(target, "KMIPKIT-ELEM-ENUMERATION-ITEM-TYPE")) continue;
                String targetId = target.path("target_id").asText();
                long itemTypeCode = Long.parseUnsignedLong(
                        target.path("wire_value").asText().replaceFirst("^0[xX]", ""), 16);
                TtlvItemType itemType = TtlvItemType.fromCode(Math.toIntExact(itemTypeCode));
                if (!normalizeName(itemType.name()).equals(normalizeName(target.path("name").asText()))) {
                    throw new AssertionError("Item Type code does not match its catalogued name: " + targetId);
                }
                String valueCaseId = caseIdByTarget.get(targetId);
                if (valueCaseId == null) {
                    throw new IllegalArgumentException("Item Type value has no individual test case: " + targetId);
                }
                byte[] wire = scalarWire(rawTag, TtlvItemType.Enumeration.code(), Math.toIntExact(itemTypeCode));
                try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked();
                        TtlvValue value = TtlvValue.Enumeration(itemTypeCode);
                        TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults());
                        var view = item.view(); var viewedTag = view.tag(); var viewedValue = view.value()) {
                    assertEquals(rawTag, viewedTag.raw());
                    assertEquals(TtlvItemType.Enumeration, view.itemType());
                    assertEquals(itemTypeCode, viewedValue.enumeration());
                }
                try (TtlvItem decoded = TtlvCodec.decode(wire);
                        var view = decoded.view(); var decodedTag = view.tag(); var decodedValue = view.value()) {
                    assertEquals(rawTag, decodedTag.raw());
                    assertEquals(TtlvItemType.Enumeration, view.itemType());
                    assertEquals(itemTypeCode, decodedValue.enumeration());
                }
                for (String evidenceCaseId : List.of(caseId, valueCaseId)) {
                    emit(sink, evidenceCaseId, targetId, "model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                            "org.kmipkit.ttlv.TtlvValue.Enumeration", "construct", wire);
                    emit(sink, evidenceCaseId, targetId, "ttlv_item.create", "org.kmipkit.ttlv.TtlvItem.create",
                            "construct", wire);
                    emit(sink, evidenceCaseId, targetId, "function.ttlv_item_view_from_item",
                            "org.kmipkit.ttlv.TtlvItem.view", "decode", wire);
                    emit(sink, evidenceCaseId, targetId, "ttlv_codec.decode", "org.kmipkit.ttlv.TtlvCodec.decode",
                            "decode", wire);
                }
                count++;
            }
        }
        if (count != 11) {
            throw new IllegalArgumentException("Expected all 11 catalogued Item Type values for " + version);
        }
    }

    private static boolean hasParent(JsonNode target, String parentId) {
        for (JsonNode parent : target.path("parent_element_ids")) {
            if (parentId.equals(parent.asText())) return true;
        }
        return false;
    }

    // OASIS KMIP 2.0/2.1 §4.16 Table 59 defines Random IV in Cryptographic Parameters.
    private static void executeTypedRandomIv(String caseId, String targetId, int tagCode,
            EvidenceSink sink) throws Exception {
        byte[] childWire = booleanWire(tagCode, true);
        byte[] wire = structureWire(0x42002b, childWire);
        try (CryptographicParameters parameters = CryptographicParameters.builder().randomIv(true).build();
                TtlvStructure source = parameters.intoTtlv();
                CryptographicParameters parsed = CryptographicParameters.fromTtlv(source)) {
            assertEquals(java.util.Optional.of(true), parsed.randomIv());
            try (TtlvStructure output = parsed.intoTtlv(); TtlvStructureView structure = output.view();
                    TtlvItemView randomIv = structure.itemAt(0); Tag tag = randomIv.tag();
                    TtlvValueView value = randomIv.value()) {
                assertEquals(1L, structure.itemCount());
                assertEquals(tagCode, tag.raw());
                assertEquals(TtlvItemType.Boolean, value.itemType());
                assertEquals(true, value.booleanValue());
            }
        }
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder",
                "org.kmipkit.protocol.CryptographicParameters.builder", "construct", wire);
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder.randomiv",
                "org.kmipkit.protocol.CryptographicParameters.Builder.randomIv", "construct", wire);
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder.build",
                "org.kmipkit.protocol.CryptographicParameters.Builder.build", "construct", wire);
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.fromttlv",
                "org.kmipkit.protocol.CryptographicParameters.fromTtlv", "decode", wire);

        byte[] cryptographicParametersWire = structureWire(0x42002b, childWire);
        byte[] attributesWire = structureWire(0x420125, cryptographicParametersWire);
        try (CryptographicParameters parameters = CryptographicParameters.builder().randomIv(true).build();
                TtlvValue value = TtlvValue.structure(parameters.intoTtlv(), CodecLimits.defaults());
                TypedAttribute typed = TypedAttribute.create(StandardAttribute.CRYPTOGRAPHIC_PARAMETERS, value);
                TtlvItem item = typed.asItem();
                TypedAttribute decoded = TypedAttribute.fromItem(StandardAttribute.CRYPTOGRAPHIC_PARAMETERS, item);
                TtlvItem setItem = decoded.asItem();
                TtlvStructure empty = TtlvStructure.create();
                TtlvStructure items = TtlvStructure.withItem(empty, setItem, CodecLimits.defaults());
                AttributeSet attributes = AttributeSet.tryNew(items)) {
            assertEquals(StandardAttribute.CRYPTOGRAPHIC_PARAMETERS, decoded.attribute());
            assertEquals(1, attributes.len());
            assertEquals(1, attributes.typedCount(StandardAttribute.CRYPTOGRAPHIC_PARAMETERS));
        }
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
                "org.kmipkit.ttlv.TypedAttribute.create", "construct", cryptographicParametersWire);
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem",
                "org.kmipkit.ttlv.TypedAttribute.fromItem", "construct", cryptographicParametersWire);
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew",
                "org.kmipkit.extensions.AttributeSet.tryNew", "construct", attributesWire);
    }

    // OASIS KMIP 2.1 §4.53 Table 135: Rotate Name Value and Rotate Name Type are required members.
    static void executeRotateNameType(JsonNode catalog, String version, EvidenceSink sink) throws Exception {
        if (!"2.1".equals(version)) return;
        String caseId = ROTATE_NAME_TYPE_CASE_ID;
        String targetId = ROTATE_NAME_TYPE_TARGET_ID;
        JsonNode target = findTarget(catalog, targetId);
        assertEquals("Rotate Name Type", target.path("name").asText());
        assertEquals("Enumeration", target.path("source_encoding").asText());
        assertEquals("structure_member", target.path("kind").asText());

        int rotateNameValueTag = 0x420170;
        int rotateNameTypeTag = 0x420171;
        long rotateNameType = 2L; // URI, OASIS KMIP 2.1 §11.51 Table 482.
        byte[] nameBytes = "urn:kmipkit:rotate-name-fixture".getBytes(
                java.nio.charset.StandardCharsets.UTF_8);
        byte[] memberWire = itemWire(rotateNameTypeTag, 5,
                ByteBuffer.allocate(4).putInt((int) rotateNameType).array());
        byte[] rotateNameWire = rotateNameAttributeWire(nameBytes, rotateNameType);

        try (TtlvValue genericValue = TtlvValue.Enumeration(rotateNameType);
                RawTag raw = RawTag.fromRaw(rotateNameTypeTag);
                Tag tag = raw.checked();
                TtlvItem genericItem = TtlvItem.create(tag, genericValue, CodecLimits.defaults());
                TtlvItemView genericView = genericItem.view();
                Tag viewedTag = genericView.tag();
                TtlvValueView viewedValue = genericView.value();
                TtlvItem genericDecoded = TtlvCodec.decode(memberWire);
                TtlvItemView decodedView = genericDecoded.view();
                Tag decodedTag = decodedView.tag();
                TtlvValueView decodedValue = decodedView.value()) {
            assertEquals(rotateNameTypeTag, viewedTag.raw());
            assertEquals(TtlvItemType.Enumeration, genericView.itemType());
            assertEquals(rotateNameType, viewedValue.enumeration());
            assertEquals(rotateNameTypeTag, decodedTag.raw());
            assertEquals(TtlvItemType.Enumeration, decodedView.itemType());
            assertEquals(rotateNameType, decodedValue.enumeration());
        }
        emitRoute(sink, caseId, targetId,
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                "org.kmipkit.ttlv.TtlvValue.Enumeration", "construct", memberWire);
        emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                "org.kmipkit.ttlv.TtlvItem.create", "construct", memberWire);
        emitRoute(sink, caseId, targetId,
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "org.kmipkit.ttlv.TtlvItem.view", "decode", memberWire);
        emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                "org.kmipkit.ttlv.TtlvCodec.decode", "decode", memberWire);

        try (TtlvStructure empty = TtlvStructure.create();
                TtlvValue nameValue = TtlvValue.TextString(nameBytes, CodecLimits.defaults());
                TtlvStructure withName = append(empty, rotateNameValueTag, nameValue);
                TtlvValue typeValue = TtlvValue.Enumeration(rotateNameType);
                TtlvStructure rotateName = append(withName, rotateNameTypeTag, typeValue);
                TtlvValue rotateNameValue = TtlvValue.structure(rotateName, CodecLimits.defaults());
                TypedAttribute typed = TypedAttribute.create(StandardAttribute.ROTATE_NAME, rotateNameValue);
                TtlvItem typedItem = typed.asItem();
                TypedAttribute decoded = TypedAttribute.fromItem(StandardAttribute.ROTATE_NAME, typedItem);
                TtlvItem attributeSetItem = decoded.asItem();
                TtlvStructure emptyAttributes = TtlvStructure.create();
                TtlvStructure attributeItems = TtlvStructure.withItem(
                        emptyAttributes, attributeSetItem, CodecLimits.defaults());
                AttributeSet attributes = AttributeSet.tryNew(attributeItems);
                TtlvItem decodedItem = decoded.asItem();
                TtlvItemView attributeView = decodedItem.view();
                TtlvValueView attributeValue = attributeView.value();
                TtlvStructureView rotateNameView = attributeValue.structure();
                TtlvItemView decodedName = rotateNameView.itemAt(0);
                Tag decodedNameTag = decodedName.tag();
                TtlvValueView decodedNameValue = decodedName.value();
                TtlvItemView decodedType = rotateNameView.itemAt(1);
                Tag decodedTypeTag = decodedType.tag();
                TtlvValueView decodedTypeValue = decodedType.value()) {
            assertEquals(StandardAttribute.ROTATE_NAME, decoded.attribute());
            assertEquals(2L, rotateNameView.itemCount());
            assertEquals(rotateNameValueTag, decodedNameTag.raw());
            assertArrayEquals(nameBytes, decodedNameValue.bytes());
            assertEquals(rotateNameTypeTag, decodedTypeTag.raw());
            assertEquals(rotateNameType, decodedTypeValue.enumeration());
            assertEquals(1, attributes.len());
            assertEquals(1, attributes.typedCount(StandardAttribute.ROTATE_NAME));
        }
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
                "org.kmipkit.ttlv.TypedAttribute.create", "construct", rotateNameWire);
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem",
                "org.kmipkit.ttlv.TypedAttribute.fromItem", "construct", rotateNameWire);
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew",
                "org.kmipkit.extensions.AttributeSet.tryNew", "construct",
                structureWire(0x420125, rotateNameWire));
    }

    // OASIS KMIP 2.1 §4.53 Table 135: Rotate Name Value is a required Text String member.
    static void executeRotateNameValue(JsonNode catalog, String version, EvidenceSink sink) throws Exception {
        if (!"2.1".equals(version)) return;
        String caseId = ROTATE_NAME_VALUE_CASE_ID;
        String targetId = ROTATE_NAME_VALUE_TARGET_ID;
        JsonNode target = findTarget(catalog, targetId);
        assertEquals("Rotate Name Value", target.path("name").asText());
        assertEquals("Text String", target.path("source_encoding").asText());
        assertEquals("structure_member", target.path("kind").asText());

        int rotateNameValueTag = 0x420170;
        int rotateNameTypeTag = 0x420171;
        long rotateNameType = 2L; // URI, OASIS KMIP 2.1 §11.51 Table 482.
        byte[] nameBytes = "urn:kmipkit:rotate-name-fixture".getBytes(
                java.nio.charset.StandardCharsets.UTF_8);
        byte[] memberWire = itemWire(rotateNameValueTag, 7, nameBytes);
        byte[] rotateNameWire = rotateNameAttributeWire(nameBytes, rotateNameType);

        try (TtlvValue genericValue = TtlvValue.TextString(nameBytes, CodecLimits.defaults());
                RawTag raw = RawTag.fromRaw(rotateNameValueTag);
                Tag tag = raw.checked();
                TtlvItem genericItem = TtlvItem.create(tag, genericValue, CodecLimits.defaults());
                TtlvItemView genericView = genericItem.view();
                Tag viewedTag = genericView.tag();
                TtlvValueView viewedValue = genericView.value();
                TtlvItem genericDecoded = TtlvCodec.decode(memberWire);
                TtlvItemView decodedView = genericDecoded.view();
                Tag decodedTag = decodedView.tag();
                TtlvValueView decodedValue = decodedView.value()) {
            assertEquals(rotateNameValueTag, viewedTag.raw());
            assertEquals(TtlvItemType.TextString, genericView.itemType());
            assertArrayEquals(nameBytes, viewedValue.bytes());
            assertEquals(rotateNameValueTag, decodedTag.raw());
            assertEquals(TtlvItemType.TextString, decodedView.itemType());
            assertArrayEquals(nameBytes, decodedValue.bytes());
        }
        emitRoute(sink, caseId, targetId,
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.textstring",
                "org.kmipkit.ttlv.TtlvValue.TextString", "construct", memberWire);
        emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_item.create",
                "org.kmipkit.ttlv.TtlvItem.create", "construct", memberWire);
        emitRoute(sink, caseId, targetId,
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "org.kmipkit.ttlv.TtlvItem.view", "decode", memberWire);
        emitRoute(sink, caseId, targetId, "java.generic-ttlv.ttlv_codec.decode",
                "org.kmipkit.ttlv.TtlvCodec.decode", "decode", memberWire);

        try (TtlvStructure empty = TtlvStructure.create();
                TtlvValue nameValue = TtlvValue.TextString(nameBytes, CodecLimits.defaults());
                TtlvStructure withName = append(empty, rotateNameValueTag, nameValue);
                TtlvValue typeValue = TtlvValue.Enumeration(rotateNameType);
                TtlvStructure rotateName = append(withName, rotateNameTypeTag, typeValue);
                TtlvValue rotateNameValue = TtlvValue.structure(rotateName, CodecLimits.defaults());
                TypedAttribute typed = TypedAttribute.create(StandardAttribute.ROTATE_NAME, rotateNameValue);
                TtlvItem typedItem = typed.asItem();
                TypedAttribute decoded = TypedAttribute.fromItem(StandardAttribute.ROTATE_NAME, typedItem);
                TtlvItem attributeSetItem = decoded.asItem();
                TtlvStructure emptyAttributes = TtlvStructure.create();
                TtlvStructure attributeItems = TtlvStructure.withItem(
                        emptyAttributes, attributeSetItem, CodecLimits.defaults());
                AttributeSet attributes = AttributeSet.tryNew(attributeItems);
                TtlvItem decodedItem = decoded.asItem();
                TtlvItemView attributeView = decodedItem.view();
                TtlvValueView attributeValue = attributeView.value();
                TtlvStructureView rotateNameView = attributeValue.structure();
                TtlvItemView decodedName = rotateNameView.itemAt(0);
                Tag decodedNameTag = decodedName.tag();
                TtlvValueView decodedNameValue = decodedName.value();
                TtlvItemView decodedType = rotateNameView.itemAt(1);
                Tag decodedTypeTag = decodedType.tag();
                TtlvValueView decodedTypeValue = decodedType.value()) {
            assertEquals(StandardAttribute.ROTATE_NAME, decoded.attribute());
            assertEquals(2L, rotateNameView.itemCount());
            assertEquals(rotateNameValueTag, decodedNameTag.raw());
            assertArrayEquals(nameBytes, decodedNameValue.bytes());
            assertEquals(rotateNameTypeTag, decodedTypeTag.raw());
            assertEquals(rotateNameType, decodedTypeValue.enumeration());
            assertEquals(1, attributes.len());
            assertEquals(1, attributes.typedCount(StandardAttribute.ROTATE_NAME));
        }
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
                "org.kmipkit.ttlv.TypedAttribute.create", "construct", rotateNameWire);
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem",
                "org.kmipkit.ttlv.TypedAttribute.fromItem", "construct", rotateNameWire);
        emitRoute(sink, caseId, targetId,
                "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew",
                "org.kmipkit.extensions.AttributeSet.tryNew", "construct",
                structureWire(0x420125, rotateNameWire));
    }

    // OASIS KMIP 2.0/2.1 §9.8 Tables 369/407: Batch Order Option is a Boolean.
    static void executeBatchOrderOption(JsonNode catalog, String version, EvidenceSink sink) throws Exception {
        executeBooleanMessageField(catalog, version, BATCH_ORDER_OPTION_CASE_ID,
                BATCH_ORDER_OPTION_TARGET_ID, "Batch Order Option", sink);
    }

    // OASIS KMIP 2.0/2.1 §9.3 Tables 364/402: Attestation Capable Indicator is a Boolean.
    static void executeAttestationCapableIndicator(JsonNode catalog, String version, EvidenceSink sink)
            throws Exception {
        executeBooleanMessageField(catalog, version, ATTESTATION_CAPABLE_CASE_ID,
                ATTESTATION_CAPABLE_TARGET_ID, "Attestation Capable Indicator", sink);
    }

    private static void executeBooleanMessageField(JsonNode catalog, String version, String caseId,
            String targetId, String name, EvidenceSink sink) throws Exception {
        if (!"2.0".equals(version) && !"2.1".equals(version)) return;
        JsonNode target = findTarget(catalog, targetId);
        assertEquals(name, target.path("name").asText());
        assertEquals("message_field", target.path("kind").asText());
        int tagCode = messageFieldTag(catalog, targetId, version);
        byte[] wire = booleanWire(tagCode, true);

        try (TtlvValue value = TtlvValue.Boolean(true);
                RawTag raw = RawTag.fromRaw(tagCode);
                Tag tag = raw.checked();
                TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults());
                TtlvItemView itemView = item.view();
                Tag itemTag = itemView.tag();
                TtlvValueView itemValue = itemView.value();
                TtlvItem decoded = TtlvCodec.decode(wire);
                TtlvItemView decodedView = decoded.view();
                Tag decodedTag = decodedView.tag();
                TtlvValueView decodedValue = decodedView.value()) {
            assertEquals(tagCode, raw.raw());
            assertEquals(tagCode, itemTag.raw());
            assertEquals(TtlvItemType.Boolean, itemView.itemType());
            assertEquals(true, itemValue.booleanValue());
            assertEquals(tagCode, decodedTag.raw());
            assertEquals(TtlvItemType.Boolean, decodedView.itemType());
            assertEquals(true, decodedValue.booleanValue());
        }
        emitRoute(sink, caseId, targetId,
                "java.generic-ttlv.ttlv_item.create", "org.kmipkit.ttlv.TtlvItem.create",
                "construct", wire);
        emitRoute(sink, caseId, targetId,
                "java.generic-ttlv.function.ttlv_item_view_from_item", "org.kmipkit.ttlv.TtlvItem.view",
                "decode", wire);
        emitRoute(sink, caseId, targetId,
                "java.generic-ttlv.ttlv_codec.decode", "org.kmipkit.ttlv.TtlvCodec.decode",
                "decode", wire);
    }

    private static byte[] rotateNameAttributeWire(byte[] nameBytes, long nameType) {
        byte[] typeBytes = ByteBuffer.allocate(4).putInt((int) nameType).array();
        return structureWire(0x42016f,
                concat(itemWire(0x420170, 7, nameBytes), itemWire(0x420171, 5, typeBytes)));
    }

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

    private record ReferencedEnumTarget(int tagCode, String aggregateCaseId) { }

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
            ReferencedEnumTarget referencedTarget = switch (requires.path("parent_name").asText()) {
                case "PKCS#11 Function" -> new ReferencedEnumTarget(
                        0x42015a, "test-enum-pkcs#11-function");
                case "PKCS#11 Return Code" -> new ReferencedEnumTarget(
                        0x42015d, "test-enum-pkcs#11-return-code");
                default -> throw new AssertionError("Referenced enumeration has no source-backed route fixture.");
            };
            int tagCode = referencedTarget.tagCode();
            String aggregateCaseId = referencedTarget.aggregateCaseId();
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

    private static byte[] booleanWire(int tag, boolean value) {
        return ByteBuffer.allocate(16).put((byte) (tag >>> 16)).put((byte) (tag >>> 8)).put((byte) tag)
                .put((byte) 6).putInt(8).putLong(value ? 1 : 0).array();
    }

    private static String normalizeName(String name) {
        return name.toLowerCase(java.util.Locale.ROOT).replaceAll("[^a-z0-9]", "");
    }

    private static boolean supportsVersion(JsonNode target, String version) {
        for (JsonNode candidate : target.path("protocol_versions")) {
            if (version.equals(candidate.asText())) return true;
        }
        return false;
    }
}
