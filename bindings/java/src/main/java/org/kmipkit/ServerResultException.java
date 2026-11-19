package org.kmipkit;

/** A sanitized server-result error from typed client execution. */
public final class ServerResultException extends ClientOperationException {
    private static final long serialVersionUID = 1L;

    ServerResultException(int statusCode, int categoryCode, int causeCode, int deliveryStateCode) {
        super(statusCode, categoryCode, causeCode, deliveryStateCode);
    }
}
