package org.kmipkit.ttlv;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** An owned tag/value pair for insertion into a generic TTLV Structure. */
public final class TtlvItem implements AutoCloseable {
    private final NativeHandle handle;

    private TtlvItem(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_ITEM);
    }

    public static TtlvItem create(Tag tag, TtlvValue value, CodecLimits limits) {
        NativeExtensionRegistry.ensureLoaded();
        long[] limitValues = NativeExtensionRegistry.codecLimitValues(limits);
        return NativeHandle.withNativeHandles(() -> {
            long valueHandle = value.nativeHandleOwner().transfer();
            return new TtlvItem(NativeExtensionRegistry.ttlvItemCreate(
                    tag.handle(), valueHandle, limitValues));
        });
    }

    long handle() {
        return handle.get();
    }

    /** Internal lifecycle bridge for KMIPKit's typed request constructors. */
    public NativeHandle nativeHandleOwner() {
        return handle;
    }
    @Override
    public void close() {
        handle.close();
    }
}
