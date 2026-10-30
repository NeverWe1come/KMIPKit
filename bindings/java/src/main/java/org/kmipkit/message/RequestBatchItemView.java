package org.kmipkit.message;

import java.util.OptionalLong;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Read-only request batch item view that retains its message owner. */
public final class RequestBatchItemView implements AutoCloseable {
    private final NativeHandle handle;
    private final RequestMessage owner;

    RequestBatchItemView(long nativeHandle, RequestMessage owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.REQUEST_BATCH_ITEM_VIEW);
        this.owner = owner;
    }

    /** Returns the optional raw Operation Enumeration without narrowing unknown values. */
    public OptionalLong operation() {
        return MessageNative.optionalRaw(
                handle.withValue(NativeExtensionRegistry::requestBatchItemViewOperation));
    }

    /** Returns the complete generic batch item in source order. */
    public TtlvStructure ttlv() {
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.requestBatchItemViewTtlv(value), this));
    }

    /** Returns the number of Message Extensions in this item. */
    public long extensionCount() {
        return handle.withValue(NativeExtensionRegistry::requestBatchItemExtensionCount);
    }

    /** Returns a retained view of the indexed Message Extension. */
    public MessageExtensionView extensionAt(long extensionIndex) {
        return handle.withValue(value -> new MessageExtensionView(
                NativeExtensionRegistry.requestBatchItemExtensionAt(value, extensionIndex), this));
    }

    @Override
    public void close() {
        handle.close();
    }
}
