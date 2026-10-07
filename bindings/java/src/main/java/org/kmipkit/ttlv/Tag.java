package org.kmipkit.ttlv;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** A validated KMIP TTLV tag. */
public final class Tag {
    private final NativeHandle handle;

    private Tag(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TAG);
    }

    static Tag fromNative(long nativeHandle) {
        return new Tag(nativeHandle);
    }

    public int raw() {
        return NativeExtensionRegistry.tagValue(handle.get());
    }

    public long handle() {
        return handle.get();
    }
}
