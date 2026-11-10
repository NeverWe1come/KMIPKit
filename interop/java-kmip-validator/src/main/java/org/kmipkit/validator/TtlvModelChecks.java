package org.kmipkit.validator;

import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

/** Exercises public generic TTLV constructors and views without sending a message. */
public final class TtlvModelChecks {
    private static final CodecLimits LIMITS = CodecLimits.defaults();

    private TtlvModelChecks() { }

    /** Constructs and reads one raw-preserving enumeration, including an unassigned value. */
    public static long roundTripEnumeration(long value) {
        try (RawTag rawTag = RawTag.fromRaw(0x420001);
                Tag tag = rawTag.checked();
                TtlvValue ttlvValue = TtlvValue.Enumeration(value);
                TtlvItem item = TtlvItem.create(tag, ttlvValue, LIMITS);
                var itemView = item.view();
                var valueView = itemView.value()) {
            return valueView.enumeration();
        }
    }

    /** Constructs and reads an integer used as a KMIP bitmask, preserving unknown bits. */
    public static int roundTripIntegerMask(int value) {
        try (RawTag rawTag = RawTag.fromRaw(0x420002);
                Tag tag = rawTag.checked();
                TtlvValue ttlvValue = TtlvValue.Integer(value);
                TtlvItem item = TtlvItem.create(tag, ttlvValue, LIMITS);
                var itemView = item.view();
                var valueView = itemView.value()) {
            return valueView.integer();
        }
    }

    /** Builds a generic structure and verifies that item insertion order is preserved. */
    public static int[] roundTripTagOrder() {
        TtlvStructure structure = TtlvStructure.create();
        structure = appendEnumeration(structure, 0x420001, 0xf0000001L);
        structure = appendEnumeration(structure, 0x420002, 0x80000002L);
        TtlvStructure ordered = structure;
        try (ordered; var view = ordered.view();
                var first = view.itemAt(0); var firstTag = first.tag();
                var second = view.itemAt(1); var secondTag = second.tag()) {
            return new int[] {firstTag.raw(), secondTag.raw()};
        }
    }

    /** Executes the generic value-preservation check associated with one local catalog case. */
    public static ModelCheck verifyLocalCase(CatalogLoader.CatalogCase testCase) {
        var requirements = testCase.requires();
        if (requirements.hasNonNull("enum_type")) {
            long value = roundTripEnumeration(0xf0000001L);
            return new ModelCheck(value == 0xf0000001L, "generic enumeration constructor/view");
        }
        if (requirements.hasNonNull("bitmask_type")) {
            int value = roundTripIntegerMask(0x80000001);
            return new ModelCheck(value == 0x80000001, "generic integer mask constructor/view");
        }
        return new ModelCheck(false, "local model case has no supported generic value type");
    }

    private static TtlvStructure appendEnumeration(TtlvStructure structure, int tagValue, long value) {
        try (RawTag rawTag = RawTag.fromRaw(tagValue);
                Tag tag = rawTag.checked();
                TtlvValue ttlvValue = TtlvValue.Enumeration(value)) {
            return TtlvStructure.withItem(structure, TtlvItem.create(tag, ttlvValue, LIMITS), LIMITS);
        }
    }

    public record ModelCheck(boolean passed, String diagnostic) { }
}
