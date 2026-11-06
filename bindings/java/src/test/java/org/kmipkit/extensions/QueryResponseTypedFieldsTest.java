package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

final class QueryResponseTypedFieldsTest {
    private static final int OPERATION = 0x0042_005c;
    private static final int OBJECT_TYPE = 0x0042_0057;
    private static final int VENDOR_IDENTIFICATION = 0x0042_009d;
    private static final int APPLICATION_NAMESPACE = 0x0042_0003;
    private static final int UNKNOWN_VENDOR_TAG = 0x0054_1234;

    @Test
    void typedFieldsRetainRepeatedValuesUnknownItemsAndRawOrder() {
        TtlvStructure payload = queryPayload();
        try (QueryResponse response = QueryResponse.fromPayload(payload)) {
            assertEquals(List.of(0x3bL, 0xf1234567L), response.operations());
            assertEquals(List.of(7L), response.objectTypes());
            assertEquals(List.of("vendor-a"), response.vendorIdentifications());
            assertEquals(List.of("namespace-a", "namespace-b"), response.applicationNamespaces());
            assertEquals(List.of(UNKNOWN_VENDOR_TAG), response.unknownItems().stream()
                    .map(QueryResponseTypedFieldsTest::tagValue).toList());

            try (var raw = response.rawView()) {
                assertEquals(6, raw.itemCount());
                int[] expectedOrder = {
                    OPERATION, APPLICATION_NAMESPACE, UNKNOWN_VENDOR_TAG,
                    OPERATION, OBJECT_TYPE, VENDOR_IDENTIFICATION
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
    void structureFieldsRemainNavigableAndAbsentFieldsStayAbsent() {
        TtlvStructure payload = queryPayload();
        try (QueryResponse response = QueryResponse.fromPayload(payload)) {
            assertTrue(response.serverInformations().isEmpty());
            assertTrue(response.extensionInformations().isEmpty());
            assertTrue(response.defaultsInformation().isEmpty());
            assertFalse(response.protectionStorageMasks().isPresent());
        }
    }

    private static int tagValue(TtlvItemView item) {
        return item.tag().raw();
    }

    private static TtlvStructure queryPayload() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        structure = append(structure, OPERATION, TtlvValue.Enumeration(0x3b), limits);
        structure = append(structure, APPLICATION_NAMESPACE,
                TtlvValue.TextString("namespace-a".getBytes(StandardCharsets.UTF_8), limits), limits);
        structure = append(structure, UNKNOWN_VENDOR_TAG,
                TtlvValue.TextString("vendor-extension".getBytes(StandardCharsets.UTF_8), limits), limits);
        structure = append(structure, OPERATION,
                TtlvValue.Enumeration(0xf1234567L), limits);
        structure = append(structure, OBJECT_TYPE, TtlvValue.Enumeration(7), limits);
        structure = append(structure, APPLICATION_NAMESPACE,
                TtlvValue.TextString("namespace-b".getBytes(StandardCharsets.UTF_8), limits), limits);
        structure = append(structure, VENDOR_IDENTIFICATION,
                TtlvValue.TextString("vendor-a".getBytes(StandardCharsets.UTF_8), limits), limits);
        return structure;
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
