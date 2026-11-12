package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;

/** Typed Get Attribute List request preserving an omitted identifier. */
public final class GetAttributeListBuilder {
    private final KmipClient client;
    private final UniqueIdentifier uniqueIdentifier;
    private RequestOptions options;
    private Long asynchronousIndicator;

    GetAttributeListBuilder(KmipClient client, UniqueIdentifier uniqueIdentifier) {
        if (uniqueIdentifier != null
                && uniqueIdentifier.kind() != UniqueIdentifier.Kind.TEXT_STRING) {
            throw new InvalidInputException(
                    "Get Attribute List Unique Identifier must be a Text String");
        }
        this.client = client;
        this.uniqueIdentifier = uniqueIdentifier;
    }

    /** Validates the typed request without sending it. */
    public void validate() {
        try (ClientBatchItem ignored = build()) {
            // The typed constructor validates while preserving optional fields.
        }
    }

    /** Builds the existing typed batch item without sending it. */
    public ClientBatchItem build() {
        return ClientBatchItem.getAttributeList(uniqueIdentifier);
    }

    /** Alias for {@link #build()}. */
    public ClientBatchItem toBatchItem() { return build(); }

    /** Sets exchange-local timeout options. */
    public GetAttributeListBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are required");
        options = value;
        return this;
    }

    /** Sets the explicit Request Header Asynchronous Indicator. */
    public GetAttributeListBuilder asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        asynchronousIndicator = value;
        return this;
    }

    /** Executes one typed exchange; this method never retries or polls. */
    public ClientBatchResponse execute(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are required");
        if (client == null) throw new InvalidInputException("client is required to execute");
        try (ClientBatch batch = ClientBatch.create()) {
            batch.add(build());
            if (asynchronousIndicator != null) {
                batch.asynchronousIndicator(asynchronousIndicator);
            }
            return options == null ? client.execute(batch, limits)
                    : client.executeWithOptions(batch, limits, options);
        }
    }
}
