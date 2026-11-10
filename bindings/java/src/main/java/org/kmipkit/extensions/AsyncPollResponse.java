package org.kmipkit.extensions;

import java.nio.ByteBuffer;
import java.util.function.Consumer;

import org.kmipkit.InvalidInputException;

/** Typed Poll response status backed by the originating asynchronous outcome. */
public final class AsyncPollResponse extends AsyncOperationResponseV1 {
    private final boolean pending;

    AsyncPollResponse(AsyncOperationOutcome owner, long[] values) {
        super(owner);
        pending = values[3] != 0;
    }

    /** Returns whether the Poll response remains Pending. */
    public boolean isPending() {
        ensureOpen();
        return pending;
    }

    /** Borrows the correlation value for the callback and wipes its temporary buffer. */
    public boolean withAsynchronousCorrelationValue(Consumer<ByteBuffer> callback) {
        if (callback == null) {
            throw new InvalidInputException("asynchronous correlation callback is invalid");
        }
        ensureOpen();
        return withOwnerAsynchronousCorrelationValue(callback);
    }
}
