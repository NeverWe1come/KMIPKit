package org.kmipkit.ttlv;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Shared owned TTLV lifecycle for typed managed object and key wrappers. */
abstract class ManagedTtlvValue implements AutoCloseable {
    private final NativeHandle handle;

    ManagedTtlvValue(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_ITEM);
    }

    /** Lends the complete TTLV node through a view retaining this owner. */
    public final TtlvItemView view() {
        return handle.withValue(value -> TtlvItemView.fromNative(
                NativeTtlvCodec.itemViewFromItem(value), this));
    }

    @Override
    public final void close() {
        handle.close();
    }

    @Override
    public final String toString() {
        return getClass().getSimpleName() + "([REDACTED])";
    }
}
