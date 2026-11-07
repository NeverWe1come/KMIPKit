package org.kmipkit.extensions;

import java.util.Optional;

import org.kmipkit.ttlv.TtlvStructure;

/** Owner-backed common view of one typed asynchronous operation response. */
public abstract class AsyncOperationResponseV1 {
    private final AsyncOperationOutcome owner;

    AsyncOperationResponseV1(AsyncOperationOutcome owner) {
        this.owner = owner;
    }

    /** Returns common result metadata while the originating outcome remains open. */
    public AsyncOperationMetadataV1 result() {
        return owner.metadataV1();
    }

    /** Returns an owned generic TTLV copy of the optional response payload. */
    public Optional<TtlvStructure> raw() {
        return owner.responsePayloadTtlv();
    }

    final void ensureOpen() {
        owner.ensureOpen();
    }

    final boolean withCancelCorrelationValue(java.util.function.Consumer<java.nio.ByteBuffer> callback) {
        return owner.withCancelCorrelationValue(callback);
    }

    final boolean withOwnerAsynchronousCorrelationValue(
            java.util.function.Consumer<java.nio.ByteBuffer> callback) {
        return owner.withAsynchronousCorrelationValue(callback);
    }
}
