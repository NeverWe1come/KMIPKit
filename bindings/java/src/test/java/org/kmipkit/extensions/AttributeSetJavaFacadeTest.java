package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

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

final class AttributeSetJavaFacadeTest {
    @Test
    void validatesConstructionAndAppendWhilePreservingOrderAndConsumption() {
        // KMIP Specification v2.1 §§4.12–4.13 define Contact Information as Text String and
        // Cryptographic Algorithm as Enumeration; the latter is also used for a type mismatch.
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure source = append(TtlvStructure.create(), 0x0042_0022,
                TtlvValue.TextString("support".getBytes(), limits), limits);

        try (source; AttributeSet attributes = AttributeSet.tryNew(source)) {
            assertEquals(1, attributes.len());
            assertFalse(attributes.isEmpty());

            TtlvItem second = item(0x0042_0028, TtlvValue.Enumeration(0xffff_ffffL), limits);
            try (second) {
                attributes.tryPush(second);
            }
            assertEquals(2, attributes.len());

            TtlvItem invalid = item(0x0042_0028, TtlvValue.Integer(7), limits);
            try (invalid) {
                assertThrows(InvalidInputException.class, () -> attributes.tryPush(invalid));
            }
            assertEquals(2, attributes.len(), "failed validation must leave the set unchanged");

            try (TtlvStructure copy = attributes.asItems();
                    TtlvStructureView view = copy.view()) {
                assertEquals(2, view.itemCount());
                assertEquals(0x0042_0022, tagAt(view, 0));
                assertEquals(0x0042_0028, tagAt(view, 1));
            }

            try (TtlvStructure consumed = attributes.intoItems();
                    TtlvStructureView view = consumed.view()) {
                assertEquals(2, view.itemCount());
            }
            assertThrows(InvalidInputException.class, attributes::len);
        }
    }

    @Test
    void createsAnEmptyValidatedAttributeSet() {
        try (TtlvStructure source = TtlvStructure.create();
                AttributeSet attributes = AttributeSet.tryNew(source)) {
            assertEquals(0, attributes.len());
            assertTrue(attributes.isEmpty());
            try (TtlvStructure consumed = attributes.intoItems();
                    TtlvStructureView view = consumed.view()) {
                assertEquals(0, view.itemCount());
            }
        }
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked();
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, item, limits);
        }
    }

    private static TtlvItem item(int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            return TtlvItem.create(tag, value, limits);
        }
    }

    private static int tagAt(TtlvStructureView structure, long index) {
        try (TtlvItemView item = structure.itemAt(index); Tag tag = item.tag()) {
            return tag.raw();
        }
    }
}
