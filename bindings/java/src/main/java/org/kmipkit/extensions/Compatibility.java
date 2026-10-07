package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** KMIP and KMIPKit version ranges supported by a definition. */
public final class Compatibility {
    private final NativeHandle handle;

    private Compatibility(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.COMPATIBILITY);
    }

    public static Compatibility create(int kmipMinMajor, int kmipMinMinor,
            int kmipMaxMajor, int kmipMaxMinor, String kmipkitMinimum, String kmipkitMaximum) {
        NativeExtensionRegistry.ensureLoaded();
        return new Compatibility(NativeExtensionRegistry.compatibilityCreate(
                kmipMinMajor, kmipMinMinor, kmipMaxMajor, kmipMaxMinor,
                kmipkitMinimum, kmipkitMaximum));
    }

    long handle() {
        return handle.get();
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }
}
