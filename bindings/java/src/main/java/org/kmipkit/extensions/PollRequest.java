package org.kmipkit.extensions;

import java.nio.ByteBuffer;
import java.util.function.Consumer;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Owned KMIP 2.1 Poll request model; creating or encoding it performs no network I/O. */
public final class PollRequest implements AutoCloseable {
    private final NativeHandle handle;

    private PollRequest(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.POLL_REQUEST);
    }

    /** Creates a request from the exact opaque correlation bytes. */
    public static PollRequest create(byte[] asynchronousCorrelationValue) {
        NativeExtensionRegistry.ensureLoaded();
        if (asynchronousCorrelationValue == null) {
            throw new InvalidInputException("Poll request correlation value is invalid");
        }
        return new PollRequest(NativeExtensionRegistry.pollRequestCreate(
                asynchronousCorrelationValue));
    }

    /** Lends a read-only correlation buffer to the callback and wipes its native staging copy. */
    public boolean withAsynchronousCorrelationValue(Consumer<ByteBuffer> callback) {
        NativeExtensionRegistry.ensureLoaded();
        if (callback == null) {
            throw new InvalidInputException("Poll request callback is invalid");
        }
        return handle.withValue(value -> NativeExtensionRegistry.withPollRequestCorrelationValue(
                value, callback));
    }

    /** Returns an owned generic TTLV copy of the Poll Request Payload. */
    public TtlvStructure toTtlvPayload() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.pollRequestToTtlvPayload(value), this));
    }

    @Override
    public void close() {
        handle.close();
    }
}
