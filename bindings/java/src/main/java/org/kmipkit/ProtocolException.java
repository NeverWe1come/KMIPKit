package org.kmipkit;

/** A sanitized response or TTLV protocol error from typed client execution. */
public final class ProtocolException extends ClientOperationException {
    private static final long serialVersionUID = 1L;

    ProtocolException(int statusCode, int categoryCode, int causeCode, int deliveryStateCode) {
        super(statusCode, categoryCode, causeCode, deliveryStateCode);
    }
}
