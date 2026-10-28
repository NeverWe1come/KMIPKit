package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.AbstractList;
import java.util.ArrayList;
import java.util.List;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

final class GetAttributesJavaFacadeTest {
    @Test
    void rejectsInvalidIdentifierAndNullReferences() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.getAttributes(UniqueIdentifier.integer(7), List.of()));
        ArrayList<AttributeReference> references = new ArrayList<>();
        references.add(null);
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.getAttributes(
                        UniqueIdentifier.textString("object-7"), references));
    }

    @Test
    void allowsOmittedUniqueIdentifier() {
        try (ClientBatchItem item = ClientBatchItem.getAttributes(
                null, List.of(AttributeReference.tag(0x0042_002f)))) {
            assertNotNull(item);
        }
    }

    @Test
    void rejectsTooManyReferencesBeforeIteratingOrCopyingTheList() {
        List<AttributeReference> oversized = new AbstractList<>() {
            @Override
            public AttributeReference get(int index) {
                throw new AssertionError("oversized input must be rejected before iteration");
            }

            @Override
            public int size() {
                return 100_001;
            }
        };

        assertThrows(ResourceLimitException.class,
                () -> ClientBatchItem.getAttributes(null, oversized));
    }

    @Test
    void responsePreservesRepeatedAttributeOrderAndRedactsItsString() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure attributes = append(TtlvStructure.create(), 0x0042_002f,
                TtlvValue.LongInteger(7), limits);
        attributes = append(attributes, 0x0042_002f, TtlvValue.LongInteger(19), limits);
        long handle = attributes.nativeHandleOwner().transfer();
        ClientBatchItemResult.GetAttributesResponse response =
                ClientBatchItemResult.parseGetAttributesResponse(new Object[] {
                    new long[] {1}, "object-7", new long[] {handle}
                }).orElseThrow();

        assertEquals("object-7", response.uniqueIdentifier());
        assertTrue(response.toString().contains("uniqueIdentifier=redacted"));
        assertTrue(response.toString().contains("attributes=redacted"));
        try (response; TtlvStructureView view = response.attributes().view()) {
            assertEquals(2, view.itemCount());
            assertEquals(7, longIntegerAt(view, 0));
            assertEquals(19, longIntegerAt(view, 1));
        }
    }

    @Test
    void absentResponseHasNoOwnedAttributes() {
        assertTrue(ClientBatchItemResult.parseGetAttributesResponse(new Object[] {
            new long[] {0}, null, new long[] {0}
        }).isEmpty());
    }

    @Test
    void successfulNoMatchResponseAcceptsTheUniqueIdentifierOnlyShape() {
        assertDoesNotThrow(() -> ClientBatchItemResult.parseGetAttributesResponse(new Object[] {
            new long[] {1}, "object-7", new long[] {0}
        }));
    }

    @Test
    void malformedSuccessReleasesItsNativeAttributesHandle() {
        TtlvStructure attributes = TtlvStructure.create();
        long handle = attributes.nativeHandleOwner().transfer();

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseGetAttributesResponse(new Object[] {
                    new long[] {1}, null, new long[] {handle}
                }));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvStructureView(handle));
    }

    @Test
    void absentResponseWithNativeFieldsIsRejectedAndReleased() {
        TtlvStructure attributes = TtlvStructure.create();
        long handle = attributes.nativeHandleOwner().transfer();

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseGetAttributesResponse(new Object[] {
                    new long[] {0}, "unexpected", new long[] {handle}
                }));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvStructureView(handle));
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        try (Tag tag = RawTag.fromRaw(rawTag).checked();
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, item, limits);
        }
    }

    private static long longIntegerAt(TtlvStructureView structure, long index) {
        try (TtlvItemView item = structure.itemAt(index);
                TtlvValueView value = item.value()) {
            return value.longInteger();
        }
    }
}
