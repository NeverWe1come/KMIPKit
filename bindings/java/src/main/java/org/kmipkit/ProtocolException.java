package org.kmipkit;

import java.util.Optional;

import org.kmipkit.ttlv.TtlvStructure;

/** A sanitized protocol error with generic response access only for approved UID shape failures. */
public final class ProtocolException extends ClientOperationException {
    private static final long serialVersionUID = 1L;
    private final TtlvStructure responseTtlv;

    ProtocolException(int statusCode, int categoryCode, int causeCode, int deliveryStateCode) {
        this(statusCode, categoryCode, causeCode, deliveryStateCode, 0);
    }

    ProtocolException(
            int statusCode,
            int categoryCode,
            int causeCode,
            int deliveryStateCode,
            long responseTtlvHandle) {
        super(statusCode, categoryCode, causeCode, deliveryStateCode);
        responseTtlv = responseTtlvHandle == 0
                ? null : TtlvStructure.fromNative(responseTtlvHandle, this);
    }

    /**
     * Returns the caller-owned generic response for an approved missing, repeated, or malformed
     * Unique Identifier in a successful MAC, MAC Verify, Sign, or Signature Verify response.
     * Close the returned structure when it is no longer needed.
     */
    public Optional<TtlvStructure> responseTtlv() {
        return Optional.ofNullable(responseTtlv);
    }
}
