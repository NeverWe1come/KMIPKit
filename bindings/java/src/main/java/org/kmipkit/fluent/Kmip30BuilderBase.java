package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.ttlv.CodecLimits;

/** Shared lifecycle for one explicitly version-gated KMIP 3.0 fluent operation. */
public abstract class Kmip30BuilderBase<T extends Kmip30BuilderBase<T>> implements AutoCloseable {
    private final KmipClient client;
    private final Kmip30OperationHandle handle;
    private RequestOptions options;
    private Long asynchronousIndicator;

    Kmip30BuilderBase(KmipClient client, Kmip30OperationHandle handle) {
        if (client == null || handle == null) {
            throw new InvalidInputException("KMIP 3.0 builder inputs are invalid");
        }
        this.client = client;
        this.handle = handle;
    }

    static KmipClient requireClient(KmipClient client) {
        if (client == null) {
            throw new org.kmipkit.InvalidInputException("KMIP client is required");
        }
        return client;
    }

    protected abstract T self();

    public T withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are invalid");
        options = value;
        return self();
    }

    public T asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        handle.setAsynchronousIndicator(value);
        asynchronousIndicator = value;
        return self();
    }

    public final void validate() {
        handle.validate();
    }

    public final ClientBatchItem build() {
        return handle.build();
    }

    public final ClientBatchItem toBatchItem() {
        return build();
    }

    public final ClientBatchResponse execute(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are invalid");
        return BatchExecution.execute(client, build(), limits, options, asynchronousIndicator);
    }

    @Override public final void close() { handle.close(); }
}
