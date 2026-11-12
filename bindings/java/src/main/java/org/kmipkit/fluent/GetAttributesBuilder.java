package org.kmipkit.fluent;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;

/** Typed Get Attributes request preserving the caller's reference order. */
public final class GetAttributesBuilder {
    private final KmipClient client;
    private final UniqueIdentifier uniqueIdentifier;
    private final List<AttributeReference> attributeReferences;
    private RequestOptions options;
    private Long asynchronousIndicator;

    GetAttributesBuilder(
            KmipClient client, UniqueIdentifier uniqueIdentifier,
            List<AttributeReference> attributeReferences) {
        if (uniqueIdentifier != null
                && uniqueIdentifier.kind() != UniqueIdentifier.Kind.TEXT_STRING) {
            throw new InvalidInputException(
                    "Get Attributes Unique Identifier must be a Text String");
        }
        if (attributeReferences == null || attributeReferences.size() > 100_000) {
            throw new InvalidInputException("Get Attributes references are invalid");
        }
        this.client = client;
        this.uniqueIdentifier = uniqueIdentifier;
        this.attributeReferences = new ArrayList<>(attributeReferences);
        if (this.attributeReferences.stream().anyMatch(java.util.Objects::isNull)) {
            throw new InvalidInputException("Get Attributes reference is required");
        }
    }

    /** Validates the typed request without sending it. */
    public void validate() throws org.kmipkit.ClientOperationException {
        try (ClientBatchItem ignored = build()) {
            // Construction invokes the existing typed request validator.
        }
    }

    /** Builds the existing typed batch item without sending it. */
    public ClientBatchItem build() {
        return ClientBatchItem.getAttributes(uniqueIdentifier, attributeReferences);
    }

    /** Alias for {@link #build()}. */
    public ClientBatchItem toBatchItem() { return build(); }

    /** Sets exchange-local timeout options. */
    public GetAttributesBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are required");
        options = value;
        return this;
    }

    /** Sets the explicit Request Header Asynchronous Indicator. */
    public GetAttributesBuilder asynchronousIndicator(long value) {
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
