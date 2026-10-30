package org.kmipkit.message;

import java.util.OptionalLong;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Read-only response batch item view that retains its message owner. */
public final class ResponseBatchItemView implements AutoCloseable {
    private final NativeHandle handle;
    private final ResponseMessage owner;

    ResponseBatchItemView(long nativeHandle, ResponseMessage owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.RESPONSE_BATCH_ITEM_VIEW);
        this.owner = owner;
    }

    /** Returns the optional raw Operation Enumeration without narrowing unknown values. */
    public OptionalLong operation() {
        return MessageNative.optionalRaw(
                handle.withValue(NativeExtensionRegistry::responseBatchItemViewOperation));
    }

    /** Returns the optional raw Result Status Enumeration without narrowing unknown values. */
    public OptionalLong resultStatus() {
        return MessageNative.optionalRaw(
                handle.withValue(NativeExtensionRegistry::responseBatchItemViewResultStatus));
    }

    /** Returns the complete generic batch item in source order. */
    public TtlvStructure ttlv() {
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.responseBatchItemViewTtlv(value), this));
    }

    /** Returns the number of Message Extensions in this item. */
    public long extensionCount() {
        return handle.withValue(NativeExtensionRegistry::responseBatchItemExtensionCount);
    }

    /** Returns a retained view of the indexed Message Extension. */
    public MessageExtensionView extensionAt(long extensionIndex) {
        return handle.withValue(value -> new MessageExtensionView(
                NativeExtensionRegistry.responseBatchItemExtensionAt(value, extensionIndex), this));
    }

    @Override
    public void close() {
        handle.close();
    }
}
