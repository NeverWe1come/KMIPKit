package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.ttlv.CodecLimits;
import org.jspecify.annotations.Nullable;

/** Builds and executes the empty client-initiated KMIP 2.1 §6.1.36 Ping request. */
public final class PingBuilder {
    private final KmipClient client;
    private @Nullable RequestOptions options;
    private @Nullable Long asynchronousIndicator;

    PingBuilder(KmipClient client) {
        this.client = client;
    }

    /** Sets exchange-local timeout options without changing client policy. */
    public PingBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are required");
        options = value;
        return this;
    }

    /** Sets the raw-preserving Request Header Asynchronous Indicator. */
    public PingBuilder asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        asynchronousIndicator = value;
        return this;
    }

    /** Validates the empty typed Ping request without sending it. */
    public void validate() {
        try (ClientBatchItem ignored = build()) {
            // Construction yields the existing typed Ping request.
        }
    }

    /** Builds the existing typed Ping batch item without sending it. */
    public ClientBatchItem build() {
        return ClientBatchItem.ping();
    }

    /** Alias for {@link #build()}. */
    public ClientBatchItem toBatchItem() {
        return build();
    }

    /** Executes one typed exchange with the client's configured limits and throws on error/Pending. */
    public ClientBatchItemResult execute() throws org.kmipkit.ClientOperationException {
        return BatchExecution.executeResult(client, build(), null, options, asynchronousIndicator);
    }

    /** Executes one typed exchange with the client's configured limits, returning raw outcomes. */
    public ClientBatchItemResult executeOutcome() throws org.kmipkit.ClientOperationException {
        return BatchExecution.executeOutcome(client, build(), null, options, asynchronousIndicator);
    }

    /** Executes one typed exchange with explicit limits, returning raw outcomes. */
    public ClientBatchItemResult executeOutcome(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are required");
        return BatchExecution.executeOutcome(client, build(), limits, options, asynchronousIndicator);
    }

    /** Executes one typed exchange; this builder never retries or polls. */
    public ClientBatchResponse execute(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        return BatchExecution.execute(client, build(), limits, options, asynchronousIndicator);
    }

    /** Executes one exchange with explicit call-local options. */
    public ClientBatchResponse execute(CodecLimits limits, RequestOptions value)
            throws org.kmipkit.ClientOperationException {
        if (value == null) throw new InvalidInputException("request options are required");
        options = value;
        return execute(limits);
    }

    /** Executes one exchange with explicit call-local options. */
    public ClientBatchResponse executeWithOptions(CodecLimits limits, RequestOptions value)
            throws org.kmipkit.ClientOperationException {
        return execute(limits, value);
    }
}
