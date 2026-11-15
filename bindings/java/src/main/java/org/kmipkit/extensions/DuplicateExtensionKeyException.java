package org.kmipkit.extensions;

import org.kmipkit.KmipException;

/** A registration duplicates an identity or discriminator key. */
public final class DuplicateExtensionKeyException extends KmipException {
    private static final long serialVersionUID = 1L;

    public DuplicateExtensionKeyException(String message) {
        super(message);
    }
}
