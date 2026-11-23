package org.kmipkit.client;

import org.kmipkit.protocol.KeyUsage;

/** Explicit properties for one symmetric-key Create operation. */
public record SymmetricKeyRequest(long algorithm, int length, int usageMask, String name) {
    /** Creates properties with a present KMIP Name. */
    public static SymmetricKeyRequest named(long algorithm, int length, int usageMask, String name) {
        return new SymmetricKeyRequest(algorithm, length, usageMask, name);
    }

    /** Creates properties without a Name attribute. */
    public static SymmetricKeyRequest unnamed(long algorithm, int length, int usageMask) {
        return new SymmetricKeyRequest(algorithm, length, usageMask, null);
    }
    /** Creates properties from named usage flags. */
    public SymmetricKeyRequest(long algorithm, int length, String name, KeyUsage... flags) {
        this(algorithm, length, KeyUsage.mask(flags), name);
    }
}
