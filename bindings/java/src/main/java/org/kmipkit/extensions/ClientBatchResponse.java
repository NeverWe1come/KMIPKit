package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Ordered common result metadata for one executed typed batch. */
public final class ClientBatchResponse implements AutoCloseable {
    private final NativeHandle handle;
    private final long maxElements;
    private final long maxMessageBytes;

    ClientBatchResponse(long nativeHandle, long maxElements, long maxMessageBytes) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CLIENT_BATCH_RESPONSE);
        this.maxElements = maxElements;
        this.maxMessageBytes = maxMessageBytes;
    }

    /** Returns the number of response items in server order. */
    public long itemCount() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(NativeExtensionRegistry::clientBatchResponseItemCount);
    }

    /** Returns an independently closable result view at the requested server-order index. */
    public ClientBatchItemResult itemAt(long index) {
        if (index < 0 || index >= itemCount()) {
            throw new InvalidInputException("response item index is outside the batch");
        }
        NativeExtensionRegistry.ensureLoaded();
        long item = handle.withValue(value ->
                NativeExtensionRegistry.clientBatchResponseItemAt(value, index));
        return new ClientBatchItemResult(item, maxElements, maxMessageBytes);
    }

    @Override
    public void close() {
        handle.close();
    }
}
