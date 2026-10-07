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
        return handle.withValue(value -> TtlvItemType.fromCode(
                NativeExtensionRegistry.ttlvValueViewType(value)));
    }

    public TtlvStructureView structure() {
        return handle.withValue(value -> TtlvStructureView.fromNative(
                NativeExtensionRegistry.ttlvValueViewStructure(value), this));
    }

    public int integer() {
        return handle.withValue(NativeExtensionRegistry::ttlvValueViewInteger);
    }

    public long longInteger() {
        return handle.withValue(NativeExtensionRegistry::ttlvValueViewLongInteger);
    }

    public long enumeration() {
        return handle.withValue(NativeExtensionRegistry::ttlvValueViewEnumeration);
    }

    public boolean booleanValue() {
        return handle.withValue(NativeExtensionRegistry::ttlvValueViewBoolean);
    }

    public long dateTime() {
        return handle.withValue(NativeExtensionRegistry::ttlvValueViewDateTime);
    }

    public long interval() {
        return handle.withValue(NativeExtensionRegistry::ttlvValueViewInterval);
    }

    public long dateTimeExtended() {
        return handle.withValue(NativeExtensionRegistry::ttlvValueViewDateTimeExtended);
    }

    public long byteLength() {
        return handle.withValue(NativeExtensionRegistry::ttlvValueViewByteLength);
    }

    public short byteAt(long index) {
        return handle.withValue(value -> NativeExtensionRegistry.ttlvValueViewByteAt(value, index));
    }

    long handle() {
        return handle.get();
    }
}
