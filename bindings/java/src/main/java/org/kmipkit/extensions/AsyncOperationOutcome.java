package org.kmipkit.extensions;

import java.nio.ByteBuffer;
import java.util.Optional;
import java.util.function.Consumer;

import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Caller-owned result of exactly one explicit asynchronous KMIP exchange. */
public final class AsyncOperationOutcome implements AutoCloseable {
    private final NativeHandle handle;

    AsyncOperationOutcome(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CLIENT_ASYNC_OPERATION_OUTCOME);
    }

    /** Copies common result metadata; result message contents are redacted from formatting. */
    public AsyncOperationMetadataV1 metadataV1() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] result = handle.withValue(NativeExtensionRegistry::clientAsyncOperationOutcomeMetadataV1);
        if (result == null || result.length != 2 || !(result[0] instanceof long[] values)
                || (result[1] != null && !(result[1] instanceof byte[]))) {
            throw new InvalidInputException("native asynchronous result metadata is invalid");
        }
        return new AsyncOperationMetadataV1(values, (byte[]) result[1]);
    }

    /**
     * Borrows the exact correlation bytes through a read-only direct buffer for the callback.
     *
     * <p>The temporary buffer is zeroized before this method returns, including when the callback
     * throws. A buffer retained by the callback observes only zeroed bytes afterward. Copies made
     * by application code are outside KMIPKit's zeroization control. Returns {@code false} when
     * no correlation value is present.
     */
    public boolean withAsynchronousCorrelationValue(Consumer<ByteBuffer> callback) {
        if (callback == null) {
            throw new InvalidInputException("asynchronous correlation callback is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value ->
                NativeExtensionRegistry.withClientAsyncOperationOutcomeCorrelationValue(
                        value, callback));
    }

    /** Returns the number of accepted response Message Extensions in wire order. */
    public long extensionCount() {
        NativeExtensionRegistry.ensureLoaded();
        long count = handle.withValue(
                NativeExtensionRegistry::clientAsyncOperationOutcomeExtensionCount);
        if (count < 0) {
            throw new ResourceLimitException("response extension count exceeds the Java range");
        }
        return count;
    }

    /**
     * Returns a caller-owned generic TTLV copy of the response Message Extension at the given
     * index. Close the returned structure when it is no longer needed.
     */
    public TtlvStructure extensionTtlvAt(long index) {
        if (index < 0 || index >= extensionCount()) {
            throw new InvalidInputException("response extension index is outside the outcome");
        }
        NativeExtensionRegistry.ensureLoaded();
        long structure = handle.withValue(value ->
                NativeExtensionRegistry.clientAsyncOperationOutcomeExtensionTtlvAt(value, index));
        if (structure <= 0) {
            throw new InvalidInputException("native response extension is invalid");
        }
        return TtlvStructure.fromNative(structure, null);
    }

    /** Returns a caller-owned generic TTLV copy of a successful operation response payload. */
    public Optional<TtlvStructure> responsePayloadTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long payload = handle.withValue(
                NativeExtensionRegistry::clientAsyncOperationOutcomeResponsePayloadTtlv);
        return payload == 0 ? Optional.empty() : Optional.of(TtlvStructure.fromNative(payload, null));
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "AsyncOperationOutcome[metadata=available, correlationValue=redacted, responsePayload=preserved]";
    }
}
