package org.kmipkit.message;

import java.util.OptionalLong;
import java.util.Optional;
import java.util.function.Function;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/** Read-only response batch item view that retains its message owner. */
public final class ResponseBatchItemView implements AutoCloseable {
    private static final int ASYNCHRONOUS_CORRELATION_VALUE = 0x420006;
    private static final int RESPONSE_PAYLOAD = 0x42007C;
    private static final int RESULT_MESSAGE = 0x42007D;
    private static final int RESULT_REASON = 0x42007E;
    private static final int UNIQUE_BATCH_ITEM_ID = 0x420093;

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

    /** Returns the optional echoed Unique Batch Item ID as an owned byte copy. */
    public Optional<byte[]> uniqueBatchItemId() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.byteString(value, UNIQUE_BATCH_ITEM_ID);
        }
    }

    /** Returns the optional raw Result Reason Enumeration. */
    public OptionalLong resultReason() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.enumeration(value, RESULT_REASON);
        }
    }

    /** Returns the optional Result Message text. */
    public Optional<String> resultMessage() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.textString(value, RESULT_MESSAGE);
        }
    }

    /** Returns the optional asynchronous correlation bytes as an owned copy. */
    public Optional<byte[]> asynchronousCorrelationValue() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.byteString(value, ASYNCHRONOUS_CORRELATION_VALUE);
        }
    }

    /** Invokes the callback with the optional Response Payload Structure. */
    public <T> Optional<T> withResponsePayload(Function<TtlvStructureView, T> callback) {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.withStructure(value, RESPONSE_PAYLOAD, callback);
        }
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
