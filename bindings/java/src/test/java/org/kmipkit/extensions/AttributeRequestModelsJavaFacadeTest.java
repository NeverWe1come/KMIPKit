package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;

import org.junit.jupiter.api.Test;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;

final class AttributeRequestModelsJavaFacadeTest {
    @Test
    void adjustRequestRetainsItsFieldsAndProducesPayloadAndBatchCopies() {
        TtlvValue input = TtlvValue.Integer(5);
        AttributeReference reference = AttributeReference.name("KMIPKit", "Counter");
        try (AdjustAttributeRequest request = AdjustAttributeRequest.create(
                UniqueIdentifier.textString("object-7"), reference, 1, input)) {
            input.close();
            assertEquals(reference, request.attributeReference());
            assertEquals(1, request.adjustmentType());
            try (TtlvValue copied = request.adjustmentValue().orElseThrow()) {
                assertNotNull(copied);
            }
            assertPayloadAndBatch(request, 4);
        }
    }

    @Test
    void deleteRequestPreservesTheIndependentOptionalSelectors() {
        try (TtlvItem current = textItem("current");
                DeleteAttributeRequest request = DeleteAttributeRequest.create(
                        UniqueIdentifier.textString("object-7"), current,
                        AttributeReference.tag(0x0042_000a))) {
            current.close();
            assertTrue(request.currentAttribute().isPresent());
            assertEquals(0x0042_000a,
                    request.attributeReference().orElseThrow().tagValue().orElseThrow());
            assertPayloadAndBatch(request, 3);
        }
    }

    @Test
    void deleteRequestExposesAbsentCurrentAttributeAsEmpty() {
        try (DeleteAttributeRequest request = DeleteAttributeRequest.create(
                UniqueIdentifier.textString("object-7"), null, null)) {
            assertFalse(request.currentAttribute().isPresent());
            assertFalse(request.attributeReference().isPresent());
            assertPayloadAndBatch(request, 1);
        }
    }

    @Test
    void modifyRequestKeepsBothAttributesAfterCallerInputsClose() {
        try (TtlvItem current = textItem("current");
                TtlvItem replacement = textItem("replacement");
                ModifyAttributeRequest request = ModifyAttributeRequest.create(
                        UniqueIdentifier.textString("object-7"), current, replacement)) {
            current.close();
            replacement.close();
            assertTrue(request.currentAttribute().isPresent());
            try (TtlvItem copied = request.newAttribute()) {
                assertNotNull(copied);
            }
            assertPayloadAndBatch(request, 3);
        }
    }

    @Test
    void modifyRequestExposesAbsentCurrentAttributeAsEmpty() {
        try (TtlvItem replacement = textItem("replacement");
                ModifyAttributeRequest request = ModifyAttributeRequest.create(
                        UniqueIdentifier.textString("object-7"), null, replacement)) {
            replacement.close();
            assertFalse(request.currentAttribute().isPresent());
            try (TtlvItem copied = request.newAttribute()) {
                assertNotNull(copied);
            }
            assertPayloadAndBatch(request, 2);
        }
    }

    @Test
    void setRequestKeepsTheNewAttributeAfterCallerInputCloses() {
        try (TtlvItem input = textItem("replacement");
                SetAttributeRequest request = SetAttributeRequest.create(
                        UniqueIdentifier.textString("object-7"), input)) {
            input.close();
            try (TtlvItem copied = request.newAttribute()) {
                assertNotNull(copied);
            }
            assertPayloadAndBatch(request, 2);
        }
    }

    private static void assertPayloadAndBatch(AttributeRequest request, long expectedItems) {
        try (TtlvStructure payload = request.toTtlvPayload();
                TtlvStructureView view = payload.view();
                ClientBatchItem batchItem = request.toBatchItem()) {
            assertEquals(expectedItems, view.itemCount());
            assertNotNull(batchItem);
        }
        assertTrue(request.uniqueIdentifier().isPresent());
    }

    private static TtlvItem textItem(String value) {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag rawTag = RawTag.fromRaw(0x0042_000a);
                Tag tag = rawTag.checked();
                TtlvValue ttlvValue = TtlvValue.TextString(
                        value.getBytes(StandardCharsets.UTF_8), limits)) {
            return TtlvItem.create(tag, ttlvValue, limits);
        }
    }
}
