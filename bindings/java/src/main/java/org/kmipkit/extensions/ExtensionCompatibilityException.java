package org.kmipkit.extensions;

import org.kmipkit.KmipException;

/** The extension definition is outside the configured compatibility range. */
public final class ExtensionCompatibilityException extends KmipException {
    private static final long serialVersionUID = 1L;

    public ExtensionCompatibilityException(String message) {
        super(message);
    }
}
