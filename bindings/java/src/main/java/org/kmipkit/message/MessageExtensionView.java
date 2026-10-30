package org.kmipkit.message;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Read-only opaque Message Extension view that retains its batch item owner. */
public final class MessageExtensionView implements AutoCloseable {
    private final NativeHandle handle;
    private final Object owner;

    MessageExtensionView(long nativeHandle, Object owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.MESSAGE_EXTENSION_VIEW);
        this.owner = owner;
    }

    /** Returns the full generic extension value in source order. */
    public TtlvStructure ttlv() {
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.messageExtensionViewTtlv(value), this));
    }

    @Override
    public void close() {
        handle.close();
    }
}
