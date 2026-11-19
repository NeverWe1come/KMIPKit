package org.kmipkit;

/** A sanitized transport failure with the native request delivery state. */
public final class TransportException extends ClientOperationException {
    private static final long serialVersionUID = 1L;

    TransportException(int statusCode, int categoryCode, int causeCode, int deliveryStateCode) {
        super(statusCode, categoryCode, causeCode, deliveryStateCode);
    }
}
