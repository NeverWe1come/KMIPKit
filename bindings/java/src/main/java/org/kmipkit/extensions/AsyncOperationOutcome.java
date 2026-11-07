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

    private static native long[] readCancelResponseV1(long outcome);
    private static native long[] readPollResponseV1(long outcome);
    private static native long[] readProcessResponseV1(long outcome);
    private static native long[] readQueryAsyncRequestsResponseV1(long outcome);
    private static native boolean withCancelResponseCorrelationValueV1(
            long outcome, Consumer<ByteBuffer> callback);

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

    /** Returns a caller-owned generic TTLV copy of the response payload when present. */
    public Optional<TtlvStructure> responsePayloadTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long payload = handle.withValue(
                NativeExtensionRegistry::clientAsyncOperationOutcomeResponsePayloadTtlv);
        return payload == 0 ? Optional.empty() : Optional.of(TtlvStructure.fromNative(payload, null));
    }

    /** Returns the typed Cancel response when this outcome contains a valid Cancel result. */
    public Optional<AsyncCancelResponse> cancelResponse() {
        return readResponse(0x19, NativeReader.CANCEL)
                .map(values -> new AsyncCancelResponse(this, values));
    }

    /** Returns the typed Poll response when this outcome contains a valid Poll result. */
    public Optional<AsyncPollResponse> pollResponse() {
        return readResponse(0x1a, NativeReader.POLL)
                .map(values -> new AsyncPollResponse(this, values));
    }

    /** Returns the typed Process response when this outcome contains a valid Process result. */
    public Optional<AsyncProcessResponse> processResponse() {
        return readResponse(0x3a, NativeReader.PROCESS)
                .map(values -> new AsyncProcessResponse(this, values));
    }

    /** Returns the typed Query Asynchronous Requests status with a generic payload view. */
    public Optional<AsyncQueryAsyncRequestsResponse> queryAsyncRequestsResponse() {
        return readResponse(0x39, NativeReader.QUERY_ASYNC_REQUESTS)
                .map(values -> new AsyncQueryAsyncRequestsResponse(this, values));
    }

    private Optional<long[]> readResponse(long expectedOperation, NativeReader reader) {
        NativeExtensionRegistry.ensureLoaded();
        long[] values = handle.withValue(reader.read);
        if (values == null) {
            return Optional.empty();
        }
        if (values.length != 8 || values[0] != 1 || values[1] != expectedOperation
                || values[2] < 0 || values[2] > 0xffff_ffffL
                || values[3] < 0 || values[3] > 1
                || values[4] < 0 || values[4] > 1
                || values[5] < 0 || values[5] > 0xffff_ffffL
                || values[6] < 0 || values[6] > 1
                || values[7] < 0
                || (values[4] == 0 && values[5] != 0)
                || (values[6] == 0 && values[7] != 0)) {
            throw new InvalidInputException("native asynchronous typed response is invalid");
        }
        return Optional.of(values);
    }

    private enum NativeReader {
        CANCEL(AsyncOperationOutcome::readCancelResponseV1),
        POLL(AsyncOperationOutcome::readPollResponseV1),
        PROCESS(AsyncOperationOutcome::readProcessResponseV1),
        QUERY_ASYNC_REQUESTS(AsyncOperationOutcome::readQueryAsyncRequestsResponseV1);

        private final java.util.function.LongFunction<long[]> read;

        NativeReader(java.util.function.LongFunction<long[]> read) {
            this.read = read;
        }
    }

    boolean withCancelCorrelationValue(Consumer<ByteBuffer> callback) {
        if (callback == null) {
            throw new InvalidInputException("asynchronous correlation callback is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value -> withCancelResponseCorrelationValueV1(value, callback));
    }

    void ensureOpen() {
        handle.get();
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
