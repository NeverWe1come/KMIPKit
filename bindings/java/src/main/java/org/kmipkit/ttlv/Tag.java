package org.kmipkit.ttlv;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** A validated KMIP TTLV tag. */
public final class Tag implements AutoCloseable {
    private final NativeHandle handle;

    private Tag(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TAG);
    }

    static Tag fromNative(long nativeHandle) {
        return new Tag(nativeHandle);
    }

    public int raw() {
        return handle.withValue(NativeExtensionRegistry::tagValue);
    }

    public long handle() {
        return handle.get();
    }

    @Override
    public void close() {
        handle.close();
    }
}
