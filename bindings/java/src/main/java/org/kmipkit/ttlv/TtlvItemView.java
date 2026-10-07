package org.kmipkit.ttlv;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Read-only view of a TTLV item in a borrowed Structure. */
public final class TtlvItemView {
    private final NativeHandle handle;
    private final Object owner;

    private TtlvItemView(long nativeHandle, Object owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_ITEM_VIEW);
        this.owner = owner;
    }

    public static TtlvItemView fromNative(long nativeHandle, Object owner) {
        return new TtlvItemView(nativeHandle, owner);
    }

    public Tag tag() {
        return RawTag.fromRaw(NativeExtensionRegistry.ttlvItemViewTag(handle.get())).checked();
    }

    public TtlvItemType itemType() {
        return TtlvItemType.fromCode(NativeExtensionRegistry.ttlvItemViewType(handle.get()));
    }

    public TtlvValueView value() {
        long nativeValueView = NativeExtensionRegistry.ttlvItemViewValue(handle.get());
        return TtlvValueView.fromNative(nativeValueView, this);
    }

    long handle() {
        return handle.get();
    }
}
