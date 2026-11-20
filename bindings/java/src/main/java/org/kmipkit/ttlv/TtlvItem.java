package org.kmipkit.ttlv;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** An owned tag/value pair for insertion into a generic TTLV Structure. */
public final class TtlvItem implements AutoCloseable {
    private final NativeHandle handle;

    private TtlvItem(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_ITEM);
    }

    static TtlvItem fromNative(long nativeHandle) {
        return new TtlvItem(nativeHandle);
    }

    public static TtlvItem fromNativeHandle(long nativeHandle) {
        return new TtlvItem(nativeHandle);
    }

    public static TtlvItem create(Tag tag, TtlvValue value, CodecLimits limits) {
        if (tag == null || value == null || limits == null) {
            throw new InvalidInputException("TTLV item inputs are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long[] limitValues = NativeExtensionRegistry.codecLimitValues(limits);
        return NativeHandle.withNativeHandles(() -> {
            long tagHandle = tag.handle();
            long valueHandle = value.nativeHandleOwner().transfer();
            return new TtlvItem(NativeExtensionRegistry.ttlvItemCreate(
                    tagHandle, valueHandle, limitValues));
        });
    }

    /** Creates a read-only view that retains this item as its native owner. */
    public TtlvItemView view() {
        return TtlvItemView.fromNative(NativeTtlvCodec.itemViewFromItem(handle()), this);
    }

    long handle() {
        return handle.get();
    }

    /** Internal lifecycle bridge for KMIPKit's typed request constructors. */
    public NativeHandle nativeHandleOwner() {
        return handle;
    }
    /** Returns an independent owned copy of this value. */
    public TtlvItem copy() {
        return handle.withValue(value -> TtlvItem.fromNativeHandle(
                NativeExtensionRegistry.ttlvItemCopy(value)));
    }

    /** Returns an independently owned copy of this item's value. */
    public TtlvValue valueCopy() {
        return handle.withValue(value -> TtlvValue.fromNative(
                NativeExtensionRegistry.ttlvItemValueCopy(value)));
    }

    @Override
    public void close() {
        handle.close();
    }
}
