package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;

/** Typed KMIP 2.1 Locate request with an explicit validated Attributes filter. */
public final class LocateBuilder implements AutoCloseable {
    private final KmipClient client;
    private TtlvStructure attributes;
    private Integer maximumItems;
    private Integer offsetItems;
    private Long storageStatusMask;
    private Long objectGroupMember;
    private RequestOptions options;
    private Long asynchronousIndicator;

    LocateBuilder(KmipClient client, AttributeSet attributes) {
        if (attributes == null) throw new InvalidInputException("Locate Attributes are required");
        this.client = client;
        this.attributes = attributes.asItems();
    }

    /** Limits the number of located objects returned by the server. */
    public LocateBuilder maximumItems(int value) {
        maximumItems = value;
        return this;
    }

    /** Selects the zero-based server-side result offset. */
    public LocateBuilder offsetItems(int value) {
        offsetItems = value;
        return this;
    }

    /** Sets the raw-preserving Storage Status Mask filter. */
    public LocateBuilder storageStatusMask(long value) {
        storageStatusMask = unsigned32(value, "Storage Status Mask");
        return this;
    }

    /** Sets the raw-preserving Object Group Member filter. */
    public LocateBuilder objectGroupMember(long value) {
        objectGroupMember = unsigned32(value, "Object Group Member");
        return this;
    }

    /** Sets exchange-local timeout options. */
    public LocateBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are required");
        options = value;
        return this;
    }

    /** Sets the Request Header Asynchronous Indicator independently of exchange options. */
    public LocateBuilder asynchronousIndicator(long value) {
        asynchronousIndicator = unsigned32(value, "asynchronous indicator");
        return this;
    }

    /** Validates the typed request without sending it. */
    public void validate() throws org.kmipkit.ClientOperationException {
        try (ClientBatchItem ignored = build()) {
            // Typed construction runs the shared Rust Locate validator.
        }
    }

    /** Builds the existing typed batch item without sending it. */
    public ClientBatchItem build() {
        if (attributes == null) throw new InvalidInputException("Locate builder is closed");
        return ClientBatchItem.locate(
                maximumItems, offsetItems, storageStatusMask, objectGroupMember, attributes);
    }

    /** Alias for {@link #build()}. */
    public ClientBatchItem toBatchItem() { return build(); }

    /** Executes Locate with client limits and throws on server error or Pending. */
    public ClientBatchItemResult execute() throws org.kmipkit.ClientOperationException {
        return BatchExecution.executeResult(client, build(), null, options, asynchronousIndicator);
    }

    /** Executes Locate with client limits, returning raw outcomes. */
    public ClientBatchItemResult executeOutcome() throws org.kmipkit.ClientOperationException {
        return BatchExecution.executeOutcome(client, build(), null, options, asynchronousIndicator);
    }

    /** Executes Locate with explicit limits, returning raw outcomes. */
    public ClientBatchItemResult executeOutcome(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are required");
        return BatchExecution.executeOutcome(client, build(), limits, options, asynchronousIndicator);
    }

    /** Executes one typed exchange; this method never retries or polls. */
    public ClientBatchResponse execute(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        return BatchExecution.execute(client, build(), limits, options, asynchronousIndicator);
    }

    /** Executes one exchange with explicit options supplied at the call site. */
    public ClientBatchResponse execute(CodecLimits limits, RequestOptions value)
            throws org.kmipkit.ClientOperationException {
        if (value == null) throw new InvalidInputException("request options are required");
        options = value;
        return execute(limits);
    }

    /** Executes one exchange with explicit options supplied at the call site. */
    public ClientBatchResponse executeWithOptions(CodecLimits limits, RequestOptions value)
            throws org.kmipkit.ClientOperationException {
        return execute(limits, value);
    }

    @Override
    public void close() {
        if (attributes != null) {
            attributes.close();
            attributes = null;
        }
    }

    private static long unsigned32(long value, String field) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException(field + " is outside uint32");
        }
        return value;
    }
}
