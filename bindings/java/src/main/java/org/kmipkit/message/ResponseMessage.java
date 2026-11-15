package org.kmipkit.message;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** An owned, validated response envelope. Parsing never executes another request. */
public final class ResponseMessage implements AutoCloseable {
    private final NativeHandle handle;

    private ResponseMessage(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.RESPONSE_MESSAGE);
    }

    /** Consumes and validates an owned generic TTLV Structure as a response message. */
    public static ResponseMessage fromTtlv(TtlvStructure structure) {
        return new ResponseMessage(MessageNative.parse(structure, false));
    }

    /** Returns a retained view of the response header. */
    public ResponseHeaderView header() {
        return handle.withValue(value -> new ResponseHeaderView(
                NativeExtensionRegistry.responseMessageHeader(value), this));
    }

    /** Returns the number of response batch items in source order. */
    public long batchItemCount() {
        return handle.withValue(NativeExtensionRegistry::responseMessageBatchItemCount);
    }

    /** Returns a retained view of the indexed response batch item. */
    public ResponseBatchItemView batchItemAt(long index) {
        return handle.withValue(value -> new ResponseBatchItemView(
                NativeExtensionRegistry.responseMessageBatchItemAt(value, index), this));
    }

    /** Returns an owned generic TTLV copy in the original source order. */
    public TtlvStructure ttlv() {
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.responseMessageTtlv(value), this));
    }

    @Override
    public void close() {
        handle.close();
    }
}
