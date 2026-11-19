package org.kmipkit.credentials;

/** Stable credential validation categories. Unknown numeric values remain representable. */
public final class CredentialValidationErrorKind {
    public static final int EMPTY_AUTHENTICATION = 1;
    public static final int DUPLICATE_FIELD = 2;
    public static final int MISSING_FIELD = 3;
    public static final int WRONG_FIELD_TYPE = 4;
    public static final int FIELD_OUT_OF_ORDER = 5;
    public static final int INVALID_TTLV_STRUCTURE = 6;

    private CredentialValidationErrorKind() {}
}
