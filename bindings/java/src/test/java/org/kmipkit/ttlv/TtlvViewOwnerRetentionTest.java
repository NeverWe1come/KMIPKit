package org.kmipkit.ttlv;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import java.lang.ref.WeakReference;
import java.util.Objects;

import org.junit.jupiter.api.Test;

final class TtlvViewOwnerRetentionTest {
    private static final int PRIVATE_TAG = 0x420001;

    @Test
    void liveValueViewRetainsEveryOwnerInItsBorrowedViewChain() {
        ViewOwnerChain chain = createBorrowedValueView();

        System.gc();

        assertNotNull(chain.structure().get());
        assertNotNull(chain.structureView().get());
        assertNotNull(chain.itemView().get());
        TtlvStructure structure = Objects.requireNonNull(chain.structure().get());
        TtlvStructureView structureView = Objects.requireNonNull(chain.structureView().get());
        TtlvItemView itemView = Objects.requireNonNull(chain.itemView().get());
        try (structure;
                structureView;
                itemView;
                TtlvValueView valueView = chain.valueView()) {
            assertEquals(47, valueView.integer());
        }
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
                new WeakReference<>(structure),
                new WeakReference<>(structureView),
                new WeakReference<>(itemView),
                valueView);
    }

    private record ViewOwnerChain(
            WeakReference<TtlvStructure> structure,
            WeakReference<TtlvStructureView> structureView,
            WeakReference<TtlvItemView> itemView,
            TtlvValueView valueView) { }
}
