package org.kmipkit.extensions;

import org.kmipkit.KmipException;

/** The extension schema or value is invalid. */
public final class InvalidExtensionSchemaException extends KmipException {
    private static final long serialVersionUID = 1L;

    public InvalidExtensionSchemaException(String message) {
        super(message);
    }
}
