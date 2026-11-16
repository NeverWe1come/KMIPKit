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
import org.jspecify.annotations.Nullable;

/** Typed KMIP 2.1 Revoke request preserving raw reason values and optional fields. */
public final class RevokeBuilder {
    private final KmipClient client;
    private final long reasonCode;
    private @Nullable UniqueIdentifier uniqueIdentifier;
    private @Nullable String revocationMessage;
    private @Nullable Long compromiseOccurrenceDate;
    private @Nullable RequestOptions options;
    private @Nullable Long asynchronousIndicator;

    RevokeBuilder(KmipClient client, long reasonCode, @Nullable String revocationMessage) {
        if (reasonCode < 0 || reasonCode > 0xffff_ffffL) {
            throw new InvalidInputException("Revocation Reason Code is outside uint32");
        }
        this.client = client;
        this.reasonCode = reasonCode;
        this.revocationMessage = revocationMessage;
    }

    /** Sets the optional wire-preserving object identifier. */
    public RevokeBuilder uniqueIdentifier(@Nullable UniqueIdentifier value) {
        uniqueIdentifier = value;
        return this;
    }

    /** Sets the optional audit-only Revocation Message. */
    public RevokeBuilder revocationMessage(@Nullable String value) {
        revocationMessage = value;
        return this;
    }

    /** Sets the optional KMIP Date-Time compromise occurrence value. */
    public RevokeBuilder compromiseOccurrenceDate(@Nullable Long value) {
        compromiseOccurrenceDate = value;
        return this;
    }

    /** Sets exchange-local timeout options. */
    public RevokeBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are required");
        options = value;
        return this;
    }

    /** Sets the Request Header Asynchronous Indicator independently of exchange options. */
    public RevokeBuilder asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        asynchronousIndicator = value;
        return this;
    }

    /** Validates the typed request without sending it. */
    public void validate() {
        try (ClientBatchItem ignored = build()) {
            // Typed construction runs the shared Rust Revoke validator.
        }
    }

    /** Builds the existing typed batch item without sending it. */
    public ClientBatchItem build() {
        try (RevocationReason reason = RevocationReason.create(reasonCode, revocationMessage);
                RevokeRequest request = RevokeRequest.create(
                        uniqueIdentifier, reason, compromiseOccurrenceDate)) {
            return ClientBatchItem.revoke(request);
        }
    }

    /** Alias for {@link #build()}. */
    public ClientBatchItem toBatchItem() { return build(); }

    /** Executes one typed exchange; this method never retries or polls. */
    public ClientBatchResponse execute(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are required");
        if (client == null) throw new InvalidInputException("client is required to execute");
        try (ClientBatch batch = ClientBatch.create()) {
            batch.add(build());
            if (asynchronousIndicator != null) batch.asynchronousIndicator(asynchronousIndicator);
            return options == null ? client.execute(batch, limits)
                    : client.executeWithOptions(batch, limits, options);
        }
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
}
