package org.kmipkit;

import java.util.Optional;

/** Base exception for sanitized errors reported by typed client execution. */
public class ClientOperationException extends KmipException {
    private static final long serialVersionUID = 1L;

    private final int statusCode;
    private final int categoryCode;
    private final int causeCode;
    private final int deliveryStateCode;
    private final ProfileValidationDetails profileValidation;

    ClientOperationException(int statusCode, int categoryCode, int causeCode, int deliveryStateCode) {
        this(statusCode, categoryCode, causeCode, deliveryStateCode, null);
    }

    ClientOperationException(
            int statusCode,
            int categoryCode,
            int causeCode,
            int deliveryStateCode,
            ProfileValidationDetails profileValidation) {
        super("KMIP client operation failed; native diagnostic details are redacted");
        this.statusCode = statusCode;
        this.categoryCode = categoryCode;
        this.causeCode = causeCode;
        this.deliveryStateCode = deliveryStateCode;
        this.profileValidation = profileValidation;
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

    /** Returns safe source-backed profile validation details, when this was such a failure. */
    public Optional<ProfileValidationDetails> profileValidation() {
        return Optional.ofNullable(profileValidation);
    }
}
