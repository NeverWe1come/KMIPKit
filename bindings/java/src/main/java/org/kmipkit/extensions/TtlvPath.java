package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.Tag;

/** Immutable finite path to a discriminator value in a vendor payload. */
public final class TtlvPath {
    private final NativeHandle handle;

    private TtlvPath(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_PATH);
    }

    public static TtlvPath create(Tag firstTag) {
        NativeExtensionRegistry.ensureLoaded();
        return new TtlvPath(NativeExtensionRegistry.ttlvPathCreate(firstTag.handle()));
    }

    public static TtlvPath withChildTag(TtlvPath path, Tag tag) {
        NativeExtensionRegistry.ensureLoaded();
        long pathHandle = path.handle.transfer();
        return new TtlvPath(NativeExtensionRegistry.ttlvPathWithChildTag(pathHandle, tag.handle()));
    }

    long handle() {
        return handle.get();
    }

    NativeHandle handleOwner() {
        return handle;
    }

    /** Internal owner token used when a consuming call takes this path with another handle. */
    public NativeHandle nativeHandleOwner() {
        return handle;
    }
}
