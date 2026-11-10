package org.kmipkit.extensions;

import java.nio.ByteBuffer;
import java.util.OptionalLong;
import java.util.function.Consumer;

import org.kmipkit.InvalidInputException;

/** Typed Cancel response fields backed by the originating asynchronous outcome. */
public final class AsyncCancelResponse extends AsyncOperationResponseV1 {
    private final OptionalLong cancellationResult;

    AsyncCancelResponse(AsyncOperationOutcome owner, long[] values) {
        super(owner);
        cancellationResult = values[4] == 0
                ? OptionalLong.empty() : OptionalLong.of(values[5]);
    }

    /** Returns the optional raw unsigned KMIP Cancellation Result value. */
    public OptionalLong cancellationResult() {
        ensureOpen();
        return cancellationResult;
    }

    /** Borrows the echoed correlation value for the callback and wipes its temporary buffer. */
    public boolean withEchoedCorrelationValue(Consumer<ByteBuffer> callback) {
        if (callback == null) {
            throw new InvalidInputException("asynchronous correlation callback is invalid");
        }
        ensureOpen();
        return withCancelCorrelationValue(callback);
    }
}
