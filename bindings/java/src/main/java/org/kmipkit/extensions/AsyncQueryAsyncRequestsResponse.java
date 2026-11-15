package org.kmipkit.extensions;

import java.nio.ByteBuffer;
import java.util.function.Consumer;

import org.kmipkit.InvalidInputException;

/** Typed Query Asynchronous Requests status with an uninterpreted generic TTLV payload. */
public final class AsyncQueryAsyncRequestsResponse extends AsyncOperationResponseV1 {
    AsyncQueryAsyncRequestsResponse(AsyncOperationOutcome owner, long[] values) {
        super(owner);
    }

    /** Returns whether this Query Asynchronous Requests response is Pending. */
    public boolean isPending() {
        return result().resultStatus() == 2;
    }

    /** Borrows the Pending correlation value for the callback and wipes its temporary buffer. */
    public boolean withAsynchronousCorrelationValue(Consumer<ByteBuffer> callback) {
        if (callback == null) {
            throw new InvalidInputException("asynchronous correlation callback is invalid");
        }
        ensureOpen();
        return withOwnerAsynchronousCorrelationValue(callback);
    }
}
