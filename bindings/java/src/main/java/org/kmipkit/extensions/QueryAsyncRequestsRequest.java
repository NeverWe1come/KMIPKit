package org.kmipkit.extensions;

import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Owned Query Asynchronous Requests model with lossless optional filters. */
public final class QueryAsyncRequestsRequest implements AutoCloseable {
    private static final int MAX_FILTER_COUNT = 100_000;
    private final NativeHandle handle;

    private QueryAsyncRequestsRequest(long nativeHandle) {
        handle = NativeHandle.owned(
                nativeHandle, NativeExtensionRegistry.QUERY_ASYNC_REQUESTS_REQUEST);
    }

    /** Creates an unfiltered query request model. */
    public static QueryAsyncRequestsRequest create() {
        NativeExtensionRegistry.ensureLoaded();
        return new QueryAsyncRequestsRequest(
                NativeExtensionRegistry.queryAsyncRequestsRequestCreate());
    }

    /** Returns a copy with an ordered correlation filter; an empty list is present-empty. */
    public QueryAsyncRequestsRequest withCorrelationValues(List<byte[]> values) {
        NativeExtensionRegistry.ensureLoaded();
        if (values == null) {
            throw new InvalidInputException("Query correlation filter is invalid");
        }
        if (values.size() > MAX_FILTER_COUNT) {
            throw new ResourceLimitException("Query correlation filter exceeds the item limit");
        }
        byte[][] copied = new byte[values.size()][];
        for (int index = 0; index < values.size(); index++) {
            byte[] value = values.get(index);
            if (value == null) {
                throw new InvalidInputException("Query correlation filter contains a null value");
            }
            copied[index] = value;
        }
        return handle.withValue(value -> new QueryAsyncRequestsRequest(
                NativeExtensionRegistry.queryAsyncRequestsRequestWithCorrelationValues(
                        value, copied)));
    }

    /** Returns a copy with ordered raw Operation values, including future values. */
    public QueryAsyncRequestsRequest withOperations(List<Long> operations) {
        NativeExtensionRegistry.ensureLoaded();
        if (operations == null) {
            throw new InvalidInputException("Query operation filter is invalid");
        }
        if (operations.size() > MAX_FILTER_COUNT) {
            throw new ResourceLimitException("Query operation filter exceeds the item limit");
        }
        long[] copied = new long[operations.size()];
        for (int index = 0; index < operations.size(); index++) {
            Long operation = operations.get(index);
            if (operation == null || operation < 0 || operation > 0xffff_ffffL) {
                throw new InvalidInputException("Query operation value is invalid");
            }
            copied[index] = operation;
        }
        return handle.withValue(value -> new QueryAsyncRequestsRequest(
                NativeExtensionRegistry.queryAsyncRequestsRequestWithOperations(value, copied)));
    }

    /** Returns an owned generic TTLV copy of the query Request Payload. */
    public TtlvStructure toTtlvPayload() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.queryAsyncRequestsRequestToTtlvPayload(value), this));
    }

    @Override
    public void close() {
        handle.close();
    }
}
