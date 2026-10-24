package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;

/** One ordered, single-use collection of typed request batch items. */
public final class ClientBatch implements AutoCloseable {
    private static final long MAX_UINT32_VALUE = 0xffff_ffffL;
    private static final int MAX_CORRELATION_BYTES = 16 * 1024 * 1024;

    private final NativeHandle handle;
    private long itemCount;

    private ClientBatch(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CLIENT_BATCH);
    }

    /** Creates an empty batch. */
    public static ClientBatch create() {
        NativeExtensionRegistry.ensureLoaded();
        return new ClientBatch(NativeExtensionRegistry.clientBatchCreate());
    }

    /** Adds and consumes an already typed request item in caller order. */
    public ClientBatch add(ClientBatchItem item) {
        if (item == null) {
            throw new InvalidInputException("client batch item is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        NativeHandle.withNativeHandles(() -> {
            long batch = handle.get();
            if (itemCount == Long.MAX_VALUE) {
                throw new InvalidInputException("client batch item count is invalid");
            }
            long itemHandle = item.nativeHandleOwner().transfer();
            NativeExtensionRegistry.clientBatchAddItem(batch, itemHandle);
            itemCount++;
            return null;
        });
        return this;
    }

    /** Sets the raw forward-compatible KMIP Asynchronous Indicator value. */
    public ClientBatch asynchronousIndicator(long indicator) {
        validateUint32(indicator, "asynchronous indicator");
        NativeExtensionRegistry.ensureLoaded();
        handle.withValue(batch -> {
            NativeExtensionRegistry.clientBatchSetAsynchronousIndicator(batch, indicator);
            return null;
        });
        return this;
    }

    /** Sets the caller-selected request timestamp in signed Unix seconds. */
    public ClientBatch requestTimestamp(long timestamp) {
        NativeExtensionRegistry.ensureLoaded();
        handle.withValue(batch -> {
            NativeExtensionRegistry.clientBatchSetRequestTimestamp(batch, timestamp);
            return null;
        });
        return this;
    }

    /** Sets the KMIP Batch Order Option explicitly. */
    public ClientBatch ordered(boolean enabled) {
        NativeExtensionRegistry.ensureLoaded();
        handle.withValue(batch -> {
            NativeExtensionRegistry.clientBatchSetOrderOption(batch, enabled);
            return null;
        });
        return this;
    }

    /**
     * Appends a caller-selected KMIP Batch Error Continuation Option value.
     * Duplicate or unassigned values are rejected when the batch is executed.
     */
    public ClientBatch addErrorContinuationValue(long value) {
        validateUint32(value, "error continuation value");
        NativeExtensionRegistry.ensureLoaded();
        handle.withValue(batch -> {
            NativeExtensionRegistry.clientBatchAddErrorContinuationValue(batch, value);
            return null;
        });
        return this;
    }

    /**
     * Sets exact UTF-8 client correlation bytes, preserving their byte sequence.
     * The byte array must contain valid UTF-8 and may not exceed 16 MiB.
     */
    public ClientBatch clientCorrelationValue(byte[] value) {
        if (value == null) {
            throw new InvalidInputException("client correlation value is invalid");
        }
        if (value.length > MAX_CORRELATION_BYTES) {
            throw new ResourceLimitException("client correlation value exceeds the configured limit");
        }
        byte[] copy = value.clone();
        NativeExtensionRegistry.ensureLoaded();
        handle.withValue(batch -> {
            NativeExtensionRegistry.clientBatchSetClientCorrelationValue(batch, copy);
            return null;
        });
        return this;
    }

    /** Returns the number of items added in caller order. */
    public long itemCount() {
        handle.get();
        return itemCount;
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    private static void validateUint32(long value, String field) {
        if (value < 0 || value > MAX_UINT32_VALUE) {
            throw new InvalidInputException(field + " is outside uint32");
        }
    }

    @Override
    public void close() {
        handle.close();
    }
}
