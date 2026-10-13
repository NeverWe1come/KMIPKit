package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** One ordered, single-use collection of typed request batch items. */
public final class ClientBatch implements AutoCloseable {
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
        if (indicator < 0 || indicator > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        NativeExtensionRegistry.ensureLoaded();
        handle.withValue(batch -> {
            NativeExtensionRegistry.clientBatchSetAsynchronousIndicator(batch, indicator);
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

    @Override
    public void close() {
        handle.close();
    }
}
