package org.kmipkit.ttlv;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertSame;

import java.lang.reflect.Field;
import java.util.Objects;

import org.junit.jupiter.api.Test;

final class TtlvViewOwnerRetentionTest {
    private static final int PRIVATE_TAG = 0x420001;

    @Test
    void liveValueViewRetainsEveryOwnerInItsBorrowedViewChain() throws ReflectiveOperationException {
        ViewOwnerChain chain = createBorrowedValueView();

        TtlvStructure structure = chain.structure();
        TtlvStructureView structureView = chain.structureView();
        TtlvItemView itemView = chain.itemView();
        try (structure;
                structureView;
                itemView;
                TtlvValueView valueView = chain.valueView()) {
            assertSame(structure, retainedOwner(structureView));
            assertSame(structureView, retainedOwner(itemView));
            assertSame(itemView, retainedOwner(valueView));
            assertEquals(47, valueView.integer());
        }
    }

    private static Object retainedOwner(Object view) throws ReflectiveOperationException {
        Field owner = view.getClass().getDeclaredField("owner");
        if (!owner.trySetAccessible()) {
            throw new AssertionError("test must inspect the private view owner contract");
        }
        return Objects.requireNonNull(owner.get(view));
    }

    private static ViewOwnerChain createBorrowedValueView() {
        TtlvStructure structure;
        try (RawTag rawTag = RawTag.fromRaw(PRIVATE_TAG);
                Tag tag = rawTag.checked();
                TtlvStructure empty = TtlvStructure.create();
                TtlvValue value = TtlvValue.Integer(47);
                TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults())) {
            structure = TtlvStructure.withItem(empty, item, CodecLimits.defaults());
        }

        TtlvStructureView structureView = structure.view();
        TtlvItemView itemView = structureView.itemAt(0);
        TtlvValueView valueView = itemView.value();
        return new ViewOwnerChain(
                structure,
                structureView,
                itemView,
                valueView);
    }

    private record ViewOwnerChain(
            TtlvStructure structure,
            TtlvStructureView structureView,
            TtlvItemView itemView,
            TtlvValueView valueView) { }
}
