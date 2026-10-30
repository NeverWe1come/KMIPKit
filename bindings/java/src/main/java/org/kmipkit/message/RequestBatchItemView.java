package org.kmipkit.message;

import java.util.Optional;
import java.util.OptionalLong;
import java.util.function.Function;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/** Read-only request batch item view that retains its message owner. */
public final class RequestBatchItemView implements AutoCloseable {
    private static final int EPHEMERAL = 0x420154;
    private static final int REQUEST_PAYLOAD = 0x420079;
    private static final int UNIQUE_BATCH_ITEM_ID = 0x420093;

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

    /** Returns the optional Ephemeral field. */
    public Optional<Boolean> ephemeral() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.booleanValue(value, EPHEMERAL);
        }
    }

    /** Returns the optional per-item Unique Batch Item ID as an owned byte copy. */
    public Optional<byte[]> uniqueBatchItemId() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.byteString(value, UNIQUE_BATCH_ITEM_ID);
        }
    }

    /** Invokes the callback with the required Request Payload Structure. */
    public <T> Optional<T> withRequestPayload(Function<TtlvStructureView, T> callback) {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.withStructure(value, REQUEST_PAYLOAD, callback);
        }
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
