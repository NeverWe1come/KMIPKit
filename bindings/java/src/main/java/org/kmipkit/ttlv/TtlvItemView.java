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
        return handle.withValue(value -> RawTag.fromRaw(
                NativeExtensionRegistry.ttlvItemViewTag(value)).checked());
    }

    public TtlvItemType itemType() {
        return handle.withValue(value -> TtlvItemType.fromCode(
                NativeExtensionRegistry.ttlvItemViewType(value)));
    }

    public TtlvValueView value() {
        return handle.withValue(value -> TtlvValueView.fromNative(
                NativeExtensionRegistry.ttlvItemViewValue(value), this));
    }

    long handle() {
        return handle.get();
    }
}
