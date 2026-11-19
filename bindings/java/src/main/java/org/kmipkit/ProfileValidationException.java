package org.kmipkit;

/** Redacted exception for fail-closed profile resolution or validation failures. */
public final class ProfileValidationException extends KmipException {
    private static final long serialVersionUID = 1L;

    private final ProfileValidationDetails profileValidation;

    /** Creates an exception carrying safe, source-backed details from the native validator. */
    public ProfileValidationException(ProfileValidationDetails profileValidation) {
        super("KMIP profile validation failed; native diagnostic details are redacted");
        if (profileValidation == null) {
            throw new InvalidInputException("profile validation details are invalid");
        }
        this.profileValidation = profileValidation;
    }

    /** Returns the safe source-backed identifiers describing this validation failure. */
    public ProfileValidationDetails profileValidation() {
        return profileValidation;
    }
}
