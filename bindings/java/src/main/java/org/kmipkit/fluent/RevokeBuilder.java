package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.RevocationReason;
import org.kmipkit.extensions.RevokeRequest;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;

/** Builds a typed Revoke request without exposing its optional audit message. */
public final class RevokeBuilder {
    private final KmipClient client;
    private final UniqueIdentifier identifier;
    private final long reasonCode;
    private final String reasonMessage;
    private Long compromiseOccurrenceDate;
    private RequestOptions options;
    private Long asynchronousIndicator;

    RevokeBuilder(KmipClient client, UniqueIdentifier identifier, RevocationReason reason) {
        if (client == null || identifier == null || reason == null) {
            throw new InvalidInputException("Revoke inputs are invalid");
        }
        this.client = client;
        this.identifier = identifier;
        this.reasonCode = reason.rawCode();
        this.reasonMessage = reason.message().orElse(null);
    }

    /** Sets the exact signed KMIP Date-Time compromise occurrence value. */
    public RevokeBuilder compromiseOccurrenceDate(long value) {
        compromiseOccurrenceDate = value;
        return this;
    }
    /** Applies exchange-local timeout overrides. */
    public RevokeBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are invalid");
        options = value;
        return this;
    }
    /** Sets the explicit Request Header Asynchronous Indicator. */
    public RevokeBuilder asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        asynchronousIndicator = value;
        return this;
    }
    /** Builds a fresh typed Revoke item, leaving this builder reusable. */
    public ClientBatchItem build() {
        try (RevocationReason reason = RevocationReason.create(reasonCode, reasonMessage);
                RevokeRequest request = RevokeRequest.create(
                        identifier, reason, compromiseOccurrenceDate)) {
            return ClientBatchItem.revoke(request);
        }
    }
    /** Alias for {@link #build()}. */
    public ClientBatchItem toBatchItem() { return build(); }
    /** Validates this request locally without sending it. */
    public void validate() { try (ClientBatchItem ignored = build()) { } }
    /** Executes exactly one exchange and returns the typed Revoke result. */
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
