package org.kmipkit;

/** A configured KMIPKit resource limit was exceeded. */
public final class ResourceLimitException extends KmipException {
    private static final long serialVersionUID = 1L;

    public ResourceLimitException(String message) {
        super(message);
    }
}
