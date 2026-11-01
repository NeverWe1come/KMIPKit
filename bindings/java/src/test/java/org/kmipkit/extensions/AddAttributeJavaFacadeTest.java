package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;

final class AddAttributeJavaFacadeTest {
    @Test
    void standaloneAddAttributeRequestCopiesItsItemAndCreatesABatchItem() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag rawTag = RawTag.fromRaw(0x0042_000a);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.TextString(
                        "Counter".getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
                TtlvItem input = TtlvItem.create(tag, value, limits);
                AddAttributeRequest request = AddAttributeRequest.create(
                        UniqueIdentifier.textString("object-7"), input)) {
            input.close();
            assertEquals(java.util.Optional.of(UniqueIdentifier.textString("object-7")),
                    request.uniqueIdentifier());
            try (TtlvItem copied = request.newAttribute();
                    TtlvStructure payload = request.toTtlvPayload();
                    TtlvStructureView view = payload.view();
                    ClientBatchItem batchItem = request.toBatchItem()) {
                try (TtlvItemView copiedView = copied.view(); Tag copiedTag = copiedView.tag()) {
                    assertEquals(0x0042_000a, copiedTag.raw());
                }
                assertEquals(2, view.itemCount());
                assertNotNull(batchItem);
            }
        }
    }

    @Test
    void successfulResponseCopiesTheRequiredIdentifierWithoutFormattingIt() {
        ClientBatchItemResult.AddAttributeResponse response = ClientBatchItemResult
                .parseAddAttributeResponse(new Object[] {new long[] {1}, "object-7"})
                .orElseThrow();

        assertEquals("object-7", response.uniqueIdentifier());
        assertFalse(response.toString().contains("object-7"));
    }

    @Test
    void nonSuccessHasNoTypedAddAttributeResponse() {
        assertEquals(
                java.util.Optional.empty(),
                ClientBatchItemResult.parseAddAttributeResponse(
                        new Object[] {new long[] {0}, null}));
    }

    @Test
    void malformedSuccessfulResponseRejectsMissingOrNonTextIdentifier() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseAddAttributeResponse(
                        new Object[] {new long[] {1}, null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseAddAttributeResponse(
                        new Object[] {new long[] {1}, new byte[] {0x01}}));
    }
}
