package org.kmipkit.ttlv;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Read-only view of a generic TTLV Structure. The source owner is retained. */
public final class TtlvStructureView {
    private final NativeHandle handle;
    private final Object owner;

    private TtlvStructureView(long nativeHandle, Object owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_STRUCTURE_VIEW);
        this.owner = owner;
    }

    public static TtlvStructureView fromNative(long nativeHandle, Object owner) {
        return new TtlvStructureView(nativeHandle, owner);
    }

    public long itemCount() {
        return NativeExtensionRegistry.ttlvStructureViewItemCount(handle.get());
    }

    public TtlvItemView itemAt(long index) {
        long itemView = NativeExtensionRegistry.ttlvStructureViewItemAt(handle.get(), index);
        return TtlvItemView.fromNative(itemView, this);
    }

    long handle() {
        return handle.get();
    }
}
