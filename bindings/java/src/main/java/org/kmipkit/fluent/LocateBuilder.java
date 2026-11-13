package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;

/** Builds a typed Locate request while preserving explicit optional filters. */
public final class LocateBuilder {
    private final KmipClient client;
    private final AttributeSet attributes;
    private Integer maximumItems;
    private Integer offsetItems;
    private Long storageStatusMask;
    private Long objectGroupMember;
    private RequestOptions options;
    private Long asynchronousIndicator;

    LocateBuilder(KmipClient client, AttributeSet attributes) {
        if (client == null || attributes == null) {
            throw new InvalidInputException("Locate inputs are invalid");
        }
        this.client = client;
        this.attributes = attributes;
    }

    /** Selects the exact caller-supplied maximum item count. */
    public LocateBuilder maximumItems(int value) { maximumItems = value; return this; }
    /** Selects the exact caller-supplied offset. */
    public LocateBuilder offsetItems(int value) { offsetItems = value; return this; }
    /** Selects the exact caller-supplied storage-status bit mask. */
    public LocateBuilder storageStatusMask(long value) { storageStatusMask = value; return this; }
    /** Selects the exact caller-supplied object-group member value. */
    public LocateBuilder objectGroupMember(long value) { objectGroupMember = value; return this; }
    /** Applies exchange-local timeout overrides. */
    public LocateBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are invalid");
        options = value;
        return this;
    }
    /** Sets the explicit Request Header Asynchronous Indicator. */
    public LocateBuilder asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        asynchronousIndicator = value;
        return this;
    }
    /** Builds the typed Locate batch item without sending it. */
    public ClientBatchItem build() {
        try (TtlvStructure items = attributes.asItems()) {
            return ClientBatchItem.locate(
                    maximumItems, offsetItems, storageStatusMask, objectGroupMember, items);
        }
    }
    /** Alias for {@link #build()}. */
    public ClientBatchItem toBatchItem() { return build(); }
    /** Validates this request locally without sending it. */
    public void validate() { try (ClientBatchItem ignored = build()) { } }
    /** Executes exactly one exchange and returns the typed Locate result. */
    public ClientBatchResponse execute(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are invalid");
        try (ClientBatch batch = ClientBatch.create()) {
            batch.add(build());
            if (asynchronousIndicator != null) batch.asynchronousIndicator(asynchronousIndicator);
            return options == null ? client.execute(batch, limits)
                    : client.executeWithOptions(batch, limits, options);
        }
    }
}
