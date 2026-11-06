package org.kmipkit.credentials;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** A redacted credential validation failure with a stable, raw-preserving category. */
public final class CredentialValidationException extends InvalidInputException {
    private static final long serialVersionUID = 1L;
    private final int kind;

    CredentialValidationException(int kind) {
        super("credential structure is invalid (validation category " + kind + ")");
        this.kind = kind;
    }

    /** Return the stable category code, preserving unknown future values. */
    public int kind() {
        return kind;
    }

    static int lastKindFromNative() {
        NativeExtensionRegistry.ensureLoaded();
        return nativeLastKind();
    }

    private static native int nativeLastKind();
}
