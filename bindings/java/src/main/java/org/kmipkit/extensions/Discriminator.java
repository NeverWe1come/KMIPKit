package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvValue;

/** Exact path and scalar value used for conservative extension recognition. */
public final class Discriminator {
    private final NativeHandle handle;

    private Discriminator(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.DISCRIMINATOR);
    }

    public static Discriminator create(TtlvPath path, TtlvValue scalarValue) {
        NativeExtensionRegistry.ensureLoaded();
        long[] consumed = NativeHandle.transferAll(path.nativeHandleOwner(), scalarValue.nativeHandleOwner());
        return new Discriminator(NativeExtensionRegistry.discriminatorCreate(consumed[0], consumed[1]));
    }

    long handle() {
        return handle.get();
    }
}
