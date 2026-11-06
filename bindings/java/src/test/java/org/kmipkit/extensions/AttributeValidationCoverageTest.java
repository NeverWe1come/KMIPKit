package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.TtlvStructure;

final class AttributeValidationCoverageTest {
    @Test
    void attributeReferencesPreserveBothRepresentationsAndRejectInvalidValues() {
        AttributeReference tag = AttributeReference.tag(0xffff_ffffL);
        AttributeReference name = AttributeReference.name("Vendor.2", "Extended Name");

        assertTrue(tag.isTag());
        assertEquals(0xffff_ffffL, tag.tagValue().orElseThrow());
        assertTrue(tag.vendorIdentification().isEmpty());
        assertFalse(name.isTag());
        assertTrue(name.tagValue().isEmpty());
        assertEquals("Vendor.2", name.vendorIdentification().orElseThrow());
        assertEquals("Extended Name", name.attributeName().orElseThrow());
        assertEquals(tag, AttributeReference.tag(0xffff_ffffL));
        assertEquals(name, AttributeReference.name("Vendor.2", "Extended Name"));
        assertEquals(tag.hashCode(), AttributeReference.tag(0xffff_ffffL).hashCode());
        assertTrue(tag.toString().contains("ffffffff"));
        assertTrue(name.toString().contains("Vendor.2:Extended Name"));

        assertThrows(InvalidInputException.class, () -> AttributeReference.tag(-1));
        assertThrows(InvalidInputException.class, () -> AttributeReference.tag(0x1_0000_0000L));
        assertThrows(InvalidInputException.class, () -> AttributeReference.name("Vendor!", "Name"));
        assertThrows(NullPointerException.class, () -> AttributeReference.name(null, "Name"));
        assertThrows(NullPointerException.class, () -> AttributeReference.name("Vendor", null));
    }

    @Test
    void attributeRequestValidationRejectsWrongIdentifiersAndMissingReferencesBeforeNativeCalls() {
        UniqueIdentifier integerIdentifier = UniqueIdentifier.integer(7);

        assertThrows(InvalidInputException.class,
                () -> AttributeRequest.createNative(5, integerIdentifier,
                        null, null, null, 0, null));
        assertThrows(InvalidInputException.class,
                () -> AttributeRequest.createNative(5, null,
                        null, null, null, -1, null));
        assertThrows(InvalidInputException.class,
                () -> AttributeRequest.createNative(5, null,
                        null, null, null, 0x1_0000_0000L, null));
        assertThrows(InvalidInputException.class,
                () -> AttributeRequest.createReadNative(5, integerIdentifier, List.of()));
        assertThrows(InvalidInputException.class,
                () -> AttributeRequest.createReadNative(5, null, null));
        assertThrows(InvalidInputException.class,
                () -> AttributeRequest.createReadNative(5, null, Arrays.asList((AttributeReference) null)));
        assertThrows(InvalidInputException.class,
                () -> GetAttributesRequest.create(null, null));
        assertThrows(InvalidInputException.class,
                () -> AdjustAttributeRequest.create(null, null, 0, null));
    }

    @Test
    void attributeSetRejectsNullInputsAndCanReportAnEmptyCollection() {
        assertThrows(InvalidInputException.class, () -> AttributeSet.tryNew(null));
        try (TtlvStructure empty = TtlvStructure.create();
                AttributeSet set = AttributeSet.tryNew(empty)) {
            assertTrue(set.isEmpty());
            assertEquals(0, set.len());
            assertThrows(InvalidInputException.class, () -> set.tryPush(null));
            try (TtlvStructure items = set.asItems()) {
                try (var view = items.view()) {
                    assertEquals(0, view.itemCount());
                }
            }
        }
    }
}
