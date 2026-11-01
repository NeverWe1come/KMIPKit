package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Owned KMIP 2.1 Process request model; creating or encoding it performs no network I/O. */
public final class ProcessRequest implements AutoCloseable {
    private final NativeHandle handle;

    private ProcessRequest(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.PROCESS_REQUEST);
    }

    /** Creates a request from the exact opaque correlation bytes. */
    public static ProcessRequest create(byte[] asynchronousCorrelationValue) {
        NativeExtensionRegistry.ensureLoaded();
        if (asynchronousCorrelationValue == null) {
            throw new InvalidInputException("Process request correlation value is invalid");
        }
        return new ProcessRequest(NativeExtensionRegistry.processRequestCreate(
                asynchronousCorrelationValue));
    }

    /** Returns an owned generic TTLV copy of the Process Request Payload. */
    public TtlvStructure toTtlvPayload() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.processRequestToTtlvPayload(value), this));
    }

    @Override
    public void close() {
        handle.close();
    }
}
