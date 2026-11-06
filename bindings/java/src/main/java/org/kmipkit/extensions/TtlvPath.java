package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.Tag;

/** Immutable finite path to a discriminator value in a vendor payload. */
public final class TtlvPath implements AutoCloseable {
    private final NativeHandle handle;

    private TtlvPath(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_PATH);
    }

    public static TtlvPath create(Tag firstTag) {
        if (firstTag == null) {
            throw new InvalidInputException("TTLV path tag is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() ->
                new TtlvPath(NativeExtensionRegistry.ttlvPathCreate(firstTag.handle())));
    }

    public static TtlvPath withChildTag(TtlvPath path, Tag tag) {
        if (path == null || tag == null) {
            throw new InvalidInputException("TTLV path inputs are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> {
            long tagHandle = tag.handle();
            long pathHandle = path.handle.transfer();
            return new TtlvPath(NativeExtensionRegistry.ttlvPathWithChildTag(pathHandle, tagHandle));
        });
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
    @Override
    public void close() {
        handle.close();
    }
}
