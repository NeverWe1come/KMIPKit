package org.kmipkit.ttlv;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Read-only typed access to a generic TTLV value. */
public final class TtlvValueView {
    private final NativeHandle handle;
    private final Object owner;

    private TtlvValueView(long nativeHandle, Object owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_VALUE_VIEW);
        this.owner = owner;
    }

    public static TtlvValueView fromNative(long nativeHandle, Object owner) {
        return new TtlvValueView(nativeHandle, owner);
    }

    public TtlvItemType itemType() {
        return TtlvItemType.fromCode(NativeExtensionRegistry.ttlvValueViewType(handle.get()));
    }

    public TtlvStructureView structure() {
        long nativeView = NativeExtensionRegistry.ttlvValueViewStructure(handle.get());
        return TtlvStructureView.fromNative(nativeView, this);
    }

    public int integer() {
        return NativeExtensionRegistry.ttlvValueViewInteger(handle.get());
    }

    public long longInteger() {
        return NativeExtensionRegistry.ttlvValueViewLongInteger(handle.get());
    }

    public long enumeration() {
        return NativeExtensionRegistry.ttlvValueViewEnumeration(handle.get());
    }

    public boolean booleanValue() {
        return NativeExtensionRegistry.ttlvValueViewBoolean(handle.get());
    }

    public long dateTime() {
        return NativeExtensionRegistry.ttlvValueViewDateTime(handle.get());
    }

    public long interval() {
        return NativeExtensionRegistry.ttlvValueViewInterval(handle.get());
    }

    public long dateTimeExtended() {
        return NativeExtensionRegistry.ttlvValueViewDateTimeExtended(handle.get());
    }

    public long byteLength() {
        return NativeExtensionRegistry.ttlvValueViewByteLength(handle.get());
    }

    public short byteAt(long index) {
        return NativeExtensionRegistry.ttlvValueViewByteAt(handle.get(), index);
    }

    long handle() {
        return handle.get();
    }
}
