package org.kmipkit.message;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Read-only response header view that retains its message owner. */
public final class ResponseHeaderView implements AutoCloseable {
    private final NativeHandle handle;
    private final ResponseMessage owner;

    ResponseHeaderView(long nativeHandle, ResponseMessage owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.RESPONSE_HEADER_VIEW);
        this.owner = owner;
    }

    /** Returns the raw signed Protocol Version pair. */
    public ProtocolVersion protocolVersion() {
        int[] values = handle.withValue(NativeExtensionRegistry::responseHeaderViewProtocolVersion);
        if (values == null || values.length != 2) {
            throw new IllegalStateException("native protocol version result is invalid");
        }
        return new ProtocolVersion(values[0], values[1]);
    }

    /** Returns the required response Date-Time unchanged. */
    public long timeStamp() {
        return handle.withValue(NativeExtensionRegistry::responseHeaderViewTimeStamp);
    }

    /** Returns the validated batch count. */
    public int batchCount() {
        return handle.withValue(NativeExtensionRegistry::responseHeaderViewBatchCount);
    }

    /** Returns an owned copy of the complete header Structure in source order. */
    public TtlvStructure ttlv() {
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.responseHeaderViewTtlv(value), this));
    }

    @Override
    public void close() {
        handle.close();
    }
}
