package org.kmipkit.message;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** An owned, validated request envelope. Parsing never executes a request. */
public final class RequestMessage implements AutoCloseable {
    private final NativeHandle handle;

    private RequestMessage(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.REQUEST_MESSAGE);
    }

    /** Consumes and validates an owned generic TTLV Structure as a request message. */
    public static RequestMessage fromTtlv(TtlvStructure structure) {
        return new RequestMessage(MessageNative.parse(structure, true));
    }

    /** Returns a retained view of the request header. */
    public RequestHeaderView header() {
        return handle.withValue(value -> new RequestHeaderView(
                NativeExtensionRegistry.requestMessageHeader(value), this));
    }

    /** Returns the number of request batch items in source order. */
    public long batchItemCount() {
        return handle.withValue(NativeExtensionRegistry::requestMessageBatchItemCount);
    }

    /** Returns a retained view of the indexed request batch item. */
    public RequestBatchItemView batchItemAt(long index) {
        return handle.withValue(value -> new RequestBatchItemView(
                NativeExtensionRegistry.requestMessageBatchItemAt(value, index), this));
    }

    /** Returns an owned generic TTLV copy in the original source order. */
    public TtlvStructure ttlv() {
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.requestMessageTtlv(value), this));
    }

    @Override
    public void close() {
        handle.close();
    }
}
