package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.Tag;

/** A deterministic before/after child-tag constraint. */
public final class ExtensionOrderConstraint {
    private final NativeHandle handle;

    private ExtensionOrderConstraint(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.ORDER_CONSTRAINT);
    }

    public static ExtensionOrderConstraint create(Tag beforeTag, Tag afterTag) {
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> new ExtensionOrderConstraint(
                NativeExtensionRegistry.extensionOrderConstraintCreate(beforeTag.handle(), afterTag.handle())));
    }

    long handle() {
        return handle.get();
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }
}
