package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import org.junit.jupiter.api.Test;
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

final class QueryResponseTypedFieldsTest {
    private static final int OPERATION = 0x0042_005c;
    private static final int OBJECT_TYPE = 0x0042_0057;
    private static final int VENDOR_IDENTIFICATION = 0x0042_009d;
    private static final int SERVER_INFORMATION = 0x0042_0088;
    private static final int APPLICATION_NAMESPACE = 0x0042_0003;
    private static final int EXTENSION_INFORMATION = 0x0042_00a4;
    private static final int ATTESTATION_TYPE = 0x0042_00c7;
    private static final int RNG_PARAMETERS = 0x0042_00d9;
    private static final int PROFILE_INFORMATION = 0x0042_00eb;
    private static final int VALIDATION_INFORMATION = 0x0042_00df;
    private static final int CAPABILITY_INFORMATION = 0x0042_00f7;
    private static final int CLIENT_REGISTRATION_METHOD = 0x0042_00f6;
    private static final int DEFAULTS_INFORMATION = 0x0042_0152;
    private static final int PROTECTION_STORAGE_MASKS = 0x0042_015f;
    private static final int UNKNOWN_VENDOR_TAG = 0x0054_1234;

    @Test
    void typedFieldsRetainRepeatedValuesUnknownItemsAndRawOrder() {
        TtlvStructure payload = queryPayload();
        try (QueryResponse response = QueryResponse.fromClassifiedPayload(payload, classifiedFields())) {
            assertEquals(17, response.fieldCount());
            assertEquals(List.of(
                    OPERATION, APPLICATION_NAMESPACE, UNKNOWN_VENDOR_TAG, OPERATION, OBJECT_TYPE,
                    SERVER_INFORMATION, APPLICATION_NAMESPACE, EXTENSION_INFORMATION,
                    ATTESTATION_TYPE, RNG_PARAMETERS, PROFILE_INFORMATION,
                    VALIDATION_INFORMATION, CAPABILITY_INFORMATION, CLIENT_REGISTRATION_METHOD,
                    DEFAULTS_INFORMATION, PROTECTION_STORAGE_MASKS, VENDOR_IDENTIFICATION),
                    response.rawFieldTags());
            assertEquals(List.of(UNKNOWN_VENDOR_TAG), response.unknownFieldTags());
            assertEquals(QueryResponseFieldKind.ENUMERATION, response.fieldAt(0).kind());
            assertEquals(0x3bL, response.fieldAt(0).enumeration().orElseThrow());
            QueryResponseField unknown = response.fieldAt(2);
            assertEquals(UNKNOWN_VENDOR_TAG, unknown.tag());
            assertEquals(QueryResponseFieldKind.UNKNOWN, unknown.kind());
            try (TtlvItemView item = unknown.item().orElseThrow()) {
                assertEquals(UNKNOWN_VENDOR_TAG, tagValue(item));
            }
            assertEquals(List.of(0x3bL, 0xf1234567L), response.operations());
            assertEquals(List.of(7L), response.objectTypes());
            assertEquals(List.of("vendor-a"), response.vendorIdentifications());
            assertEquals(List.of("namespace-ñ", "namespace-b"), response.applicationNamespaces());
            try (var raw = response.rawView();
                    TtlvItemView namespace = raw.itemAt(1);
                    TtlvValueView value = namespace.value()) {
                assertArrayEquals("namespace-ñ".getBytes(StandardCharsets.UTF_8), value.bytes());
            }
            assertEquals(List.of(0xf1234567L), response.attestationTypes());
            assertEquals(List.of(0x80000001L), response.clientRegistrationMethods());
            assertOne(response.extensionInformations());
            assertOne(response.rngParameters());
            assertOne(response.profileInformation());
            assertOne(response.validationInformation());
            assertOne(response.capabilityInformation());
            try (var defaults = response.defaultsInformation().orElseThrow();
                    var masks = response.protectionStorageMasks().orElseThrow()) {
                assertEquals(TtlvItemType.Structure, defaults.itemType());
                assertEquals(TtlvItemType.Structure, masks.itemType());
            }
            List<TtlvItemView> serverInformation = response.serverInformations();
            assertEquals(1, serverInformation.size());
            try (TtlvItemView server = serverInformation.get(0);
                    TtlvValueView value = server.value();
                    TtlvStructureView serverFields = value.structure()) {
                assertEquals(1, serverFields.itemCount());
                try (TtlvItemView vendorField = serverFields.itemAt(0);
                        var vendorTag = vendorField.tag()) {
                    assertEquals(0x0054_0001, vendorTag.raw());
                }
            }
            List<TtlvItemView> unknownItems = response.unknownItems();
            try {
                assertEquals(List.of(UNKNOWN_VENDOR_TAG), unknownItems.stream()
                        .map(QueryResponseTypedFieldsTest::tagValue).toList());
            } finally {
                unknownItems.forEach(TtlvItemView::close);
            }

            try (var raw = response.rawView()) {
                assertEquals(17, raw.itemCount());
                int[] expectedOrder = {
                    OPERATION, APPLICATION_NAMESPACE, UNKNOWN_VENDOR_TAG,
                    OPERATION, OBJECT_TYPE, SERVER_INFORMATION, APPLICATION_NAMESPACE,
                    EXTENSION_INFORMATION, ATTESTATION_TYPE, RNG_PARAMETERS,
                    PROFILE_INFORMATION, VALIDATION_INFORMATION, CAPABILITY_INFORMATION,
                    CLIENT_REGISTRATION_METHOD, DEFAULTS_INFORMATION,
                    PROTECTION_STORAGE_MASKS, VENDOR_IDENTIFICATION
                };
                for (int index = 0; index < expectedOrder.length; index++) {
                    try (TtlvItemView item = raw.itemAt(index)) {
                        assertEquals(expectedOrder[index], item.tag().raw());
                    }
                }
            }
        }
    }

    @Test
    void absentOptionalStructureFieldsStayAbsent() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure payload = append(TtlvStructure.create(), OPERATION,
                TtlvValue.Enumeration(0x3b), limits);
        try (QueryResponse response = QueryResponse.fromClassifiedPayload(payload, List.of(
                new QueryResponse.FieldData(OPERATION, QueryResponseFieldKind.ENUMERATION, 0x3b)))) {
            assertTrue(response.serverInformations().isEmpty());
            assertTrue(response.extensionInformations().isEmpty());
            assertTrue(response.defaultsInformation().isEmpty());
            assertTrue(response.protectionStorageMasks().isEmpty());
            try (var raw = response.rawView()) {
                assertEquals(1, raw.itemCount());
            }
        }
    }

    private static int tagValue(TtlvItemView item) {
        try (var tag = item.tag()) {
            return tag.raw();
        }
    }

    private static List<QueryResponse.FieldData> classifiedFields() {
        return List.of(
                new QueryResponse.FieldData(OPERATION, QueryResponseFieldKind.ENUMERATION, 0x3b),
                new QueryResponse.FieldData(APPLICATION_NAMESPACE, QueryResponseFieldKind.TEXT_STRING, 0),
                new QueryResponse.FieldData(UNKNOWN_VENDOR_TAG, QueryResponseFieldKind.UNKNOWN, 0),
                new QueryResponse.FieldData(OPERATION, QueryResponseFieldKind.ENUMERATION, 0xf1234567L),
                new QueryResponse.FieldData(OBJECT_TYPE, QueryResponseFieldKind.ENUMERATION, 7),
                new QueryResponse.FieldData(SERVER_INFORMATION, QueryResponseFieldKind.STRUCTURE, 0),
                new QueryResponse.FieldData(APPLICATION_NAMESPACE, QueryResponseFieldKind.TEXT_STRING, 0),
                new QueryResponse.FieldData(EXTENSION_INFORMATION, QueryResponseFieldKind.STRUCTURE, 0),
                new QueryResponse.FieldData(ATTESTATION_TYPE, QueryResponseFieldKind.ENUMERATION, 0xf1234567L),
                new QueryResponse.FieldData(RNG_PARAMETERS, QueryResponseFieldKind.STRUCTURE, 0),
                new QueryResponse.FieldData(PROFILE_INFORMATION, QueryResponseFieldKind.STRUCTURE, 0),
                new QueryResponse.FieldData(VALIDATION_INFORMATION, QueryResponseFieldKind.STRUCTURE, 0),
                new QueryResponse.FieldData(CAPABILITY_INFORMATION, QueryResponseFieldKind.STRUCTURE, 0),
                new QueryResponse.FieldData(CLIENT_REGISTRATION_METHOD, QueryResponseFieldKind.ENUMERATION, 0x80000001L),
                new QueryResponse.FieldData(DEFAULTS_INFORMATION, QueryResponseFieldKind.STRUCTURE, 0),
                new QueryResponse.FieldData(PROTECTION_STORAGE_MASKS, QueryResponseFieldKind.STRUCTURE, 0),
                new QueryResponse.FieldData(VENDOR_IDENTIFICATION, QueryResponseFieldKind.TEXT_STRING, 0));
    }

    private static void assertOne(List<TtlvItemView> items) {
        assertEquals(1, items.size());
        items.forEach(TtlvItemView::close);
    }

    private static TtlvStructure queryPayload() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        structure = append(structure, OPERATION, TtlvValue.Enumeration(0x3b), limits);
        structure = append(structure, APPLICATION_NAMESPACE,
                TtlvValue.TextString("namespace-ñ".getBytes(StandardCharsets.UTF_8), limits), limits);
        structure = append(structure, UNKNOWN_VENDOR_TAG,
                TtlvValue.TextString("vendor-extension".getBytes(StandardCharsets.UTF_8), limits), limits);
        structure = append(structure, OPERATION,
                TtlvValue.Enumeration(0xf1234567L), limits);
        structure = append(structure, OBJECT_TYPE, TtlvValue.Enumeration(7), limits);
        TtlvStructure server = TtlvStructure.create();
        server = append(server, 0x0054_0001,
                TtlvValue.TextString("server-extension".getBytes(StandardCharsets.UTF_8), limits), limits);
        structure = append(structure, SERVER_INFORMATION, TtlvValue.structure(server, limits), limits);
        structure = append(structure, APPLICATION_NAMESPACE,
                TtlvValue.TextString("namespace-b".getBytes(StandardCharsets.UTF_8), limits), limits);
        structure = append(structure, EXTENSION_INFORMATION, emptyStructureValue(limits), limits);
        structure = append(structure, ATTESTATION_TYPE, TtlvValue.Enumeration(0xf1234567L), limits);
        structure = append(structure, RNG_PARAMETERS, emptyStructureValue(limits), limits);
        TtlvStructure profile = TtlvStructure.create();
        profile = append(profile, 0x0042_00ec, TtlvValue.Enumeration(0x80000001L), limits);
        structure = append(structure, PROFILE_INFORMATION, TtlvValue.structure(profile, limits), limits);
        structure = append(structure, VALIDATION_INFORMATION, emptyStructureValue(limits), limits);
        structure = append(structure, CAPABILITY_INFORMATION, emptyStructureValue(limits), limits);
        structure = append(structure, CLIENT_REGISTRATION_METHOD,
                TtlvValue.Enumeration(0x80000001L), limits);
        structure = append(structure, DEFAULTS_INFORMATION, emptyStructureValue(limits), limits);
        structure = append(structure, PROTECTION_STORAGE_MASKS, emptyStructureValue(limits), limits);
        structure = append(structure, VENDOR_IDENTIFICATION,
                TtlvValue.TextString("vendor-a".getBytes(StandardCharsets.UTF_8), limits), limits);
        return structure;
    }

    private static TtlvValue emptyStructureValue(CodecLimits limits) {
        return TtlvValue.structure(TtlvStructure.create(), limits);
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag);
                Tag tag = raw.checked();
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, item, limits);
        }
    }
}
