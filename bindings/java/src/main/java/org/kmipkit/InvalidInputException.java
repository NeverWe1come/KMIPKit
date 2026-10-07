package org.kmipkit;

/** The input is invalid or a native handle has already been closed. */
public final class InvalidInputException extends KmipException {
    private static final long serialVersionUID = 1L;

    public InvalidInputException(String message) {
        super(message);
    }
}
