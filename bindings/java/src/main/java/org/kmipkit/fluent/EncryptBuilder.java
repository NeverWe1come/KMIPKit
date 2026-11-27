package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP Encrypt request. */
public final class EncryptBuilder extends CryptoOperationBuilder<EncryptBuilder>
        implements CipherInput<EncryptBuilder> {
    EncryptBuilder(KmipClient client) {
        super(client, NATIVE_OPERATION_ENCRYPT);
    }

    /** Configures cipher parameters from a curated {@link CipherSpec}. */
    public EncryptBuilder withCipherSpec(CipherSpec spec) {
        if (spec == null) throw new org.kmipkit.InvalidInputException("CipherSpec is required");
        try (org.kmipkit.protocol.CryptographicParameters params = spec.toCryptographicParameters()) {
            return cryptographicParameters(params);
        }
    }
}
