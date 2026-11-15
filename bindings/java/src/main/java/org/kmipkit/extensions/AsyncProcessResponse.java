package org.kmipkit.extensions;

import java.nio.ByteBuffer;
import java.util.OptionalLong;
import java.util.function.Consumer;

import org.kmipkit.InvalidInputException;

/** Typed Process status and payload shape backed by its asynchronous outcome. */
public final class AsyncProcessResponse extends AsyncOperationResponseV1 {
    private final boolean pending;
    private final OptionalLong payloadMemberCount;

    AsyncProcessResponse(AsyncOperationOutcome owner, long[] values) {
        super(owner);
        pending = values[3] != 0;
        payloadMemberCount = values[6] == 0
                ? OptionalLong.empty() : OptionalLong.of(values[7]);
    }

    /** Returns whether the Process response remains Pending. */
    public boolean isPending() {
        ensureOpen();
        return pending;
    }

    /** Returns the number of members in a present Process payload structure. */
    public OptionalLong payloadMemberCount() {
        ensureOpen();
        return payloadMemberCount;
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
