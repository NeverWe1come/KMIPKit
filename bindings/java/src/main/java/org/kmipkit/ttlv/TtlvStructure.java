package org.kmipkit.ttlv;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** An owned generic TTLV Structure that preserves child order and unknown tags. */
public final class TtlvStructure {
    private final NativeHandle handle;

    private TtlvStructure(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_STRUCTURE);
    }

    public static TtlvStructure create() {
        return new TtlvStructure(NativeExtensionRegistry.ttlvStructureCreate());
    }

    public static TtlvStructure withItem(TtlvStructure structure, TtlvItem item, CodecLimits limits) {
        NativeExtensionRegistry.ensureLoaded();
        long[] limitValues = NativeExtensionRegistry.codecLimitValues(limits);
        long[] consumed = NativeHandle.transferAll(structure.handle, item.nativeHandleOwner());
        return new TtlvStructure(NativeExtensionRegistry.ttlvStructureWithItem(
                consumed[0], consumed[1], limitValues));
    }

    public TtlvStructureView view() {
        return handle.withValue(value -> TtlvStructureView.fromNative(
                NativeExtensionRegistry.ttlvStructureView(value), this));
    }

    public static TtlvStructure fromNative(long nativeHandle, Object owner) {
        return new TtlvStructure(nativeHandle);
    }

    public long handle() {
        return handle.get();
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }
}
