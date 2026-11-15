package org.kmipkit.extensions;

import org.kmipkit.KmipException;

/** The extension identity is invalid. */
public final class InvalidExtensionIdentityException extends KmipException {
    private static final long serialVersionUID = 1L;

    public InvalidExtensionIdentityException(String message) {
        super(message);
    }
}
