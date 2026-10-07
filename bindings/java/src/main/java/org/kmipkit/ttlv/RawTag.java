package org.kmipkit.ttlv;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** A 24-bit TTLV tag value that may be checked for use as a KMIP tag. */
public final class RawTag {
    private final NativeHandle handle;

    private RawTag(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.RAW_TAG);
    }

    public static RawTag fromRaw(int raw) {
        NativeExtensionRegistry.ensureLoaded();
        return new RawTag(NativeExtensionRegistry.rawTagCreate(raw));
    }

    public int raw() {
        return handle.withValue(NativeExtensionRegistry::rawTagValue);
    }

    public Tag checked() {
        return handle.withValue(value -> Tag.fromNative(NativeExtensionRegistry.rawTagChecked(value)));
    }

    long handle() {
        return handle.get();
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }
}
