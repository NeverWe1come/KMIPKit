package org.kmipkit.extensions;

/** Typed Query Asynchronous Requests status with an uninterpreted generic TTLV payload. */
public final class AsyncQueryAsyncRequestsResponse extends AsyncOperationResponseV1 {
    AsyncQueryAsyncRequestsResponse(AsyncOperationOutcome owner, long[] values) {
        super(owner);
    }
}
