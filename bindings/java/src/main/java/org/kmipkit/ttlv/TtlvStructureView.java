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
        return handle.withValue(NativeExtensionRegistry::ttlvStructureViewItemCount);
    }

    public TtlvItemView itemAt(long index) {
        return handle.withValue(value -> TtlvItemView.fromNative(
                NativeExtensionRegistry.ttlvStructureViewItemAt(value, index), this));
    }

    long handle() {
        return handle.get();
    }
}
