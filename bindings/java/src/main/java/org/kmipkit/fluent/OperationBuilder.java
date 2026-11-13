package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;

/** Typed Get or Destroy request for one explicit Unique Identifier. */
public final class OperationBuilder {
    private final KmipClient client;
    private final UniqueIdentifier id;
    private final boolean destroy;
    private RequestOptions options;
    private Long asynchronousIndicator;
    OperationBuilder(KmipClient client, UniqueIdentifier id, boolean destroy) {
        if (client == null || id == null) throw new InvalidInputException("operation inputs are invalid");
        this.client = client; this.id = id; this.destroy = destroy;
    }
    public OperationBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are invalid");
        options = value; return this;
    }
    /** Sets the explicit Request Header Asynchronous Indicator on the containing batch. */
    public OperationBuilder asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        asynchronousIndicator = value;
        return this;
    }
    public ClientBatchItem build() {
        return destroy ? ClientBatchItem.destroy(id) : ClientBatchItem.get(id, null, null, null, null);
    }
    public ClientBatchItem toBatchItem() { return build(); }
    public ClientBatchResponse execute(CodecLimits limits) throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are invalid");
        try (ClientBatch batch = ClientBatch.create()) {
            batch.add(build());
            if (asynchronousIndicator != null) batch.asynchronousIndicator(asynchronousIndicator);
            return options == null ? client.execute(batch, limits)
                    : client.executeWithOptions(batch, limits, options);
        }
    }
}
