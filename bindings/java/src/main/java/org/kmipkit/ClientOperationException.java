package org.kmipkit;

/** Base exception for sanitized errors reported by typed client execution. */
public class ClientOperationException extends KmipException {
    private static final long serialVersionUID = 1L;

    private final int statusCode;
    private final int categoryCode;
    private final int causeCode;
    private final int deliveryStateCode;

    ClientOperationException(int statusCode, int categoryCode, int causeCode, int deliveryStateCode) {
        super("KMIP client operation failed; native diagnostic details are redacted");
        this.statusCode = statusCode;
        this.categoryCode = categoryCode;
        this.causeCode = causeCode;
        this.deliveryStateCode = deliveryStateCode;
    }

    /** Returns the stable native status code. */
    public int statusCode() {
        return statusCode;
    }

    /** Returns the raw forward-compatible error category code. */
    public int categoryCode() {
        return categoryCode;
    }

    /** Returns the raw forward-compatible cause code. */
    public int causeCode() {
        return causeCode;
    }

    /** Returns the raw forward-compatible request delivery state. */
    public int deliveryStateCode() {
        return deliveryStateCode;
    }
}
