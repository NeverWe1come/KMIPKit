package org.kmipkit;

/** Base type for stable KMIPKit Java error categories. */
public class KmipException extends RuntimeException {
    private static final long serialVersionUID = 1L;

    public KmipException(String message) {
        super(message);
    }
}
