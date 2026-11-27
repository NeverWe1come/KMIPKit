package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP Decrypt request. */
public final class DecryptBuilder extends CryptoOperationBuilder<DecryptBuilder>
        implements DecryptInput<DecryptBuilder> {
    DecryptBuilder(KmipClient client) {
        super(client, NATIVE_OPERATION_DECRYPT);
    }

    /** Configures cipher parameters from a curated {@link CipherSpec}. */
    public DecryptBuilder withCipherSpec(CipherSpec spec) {
        if (spec == null) throw new org.kmipkit.InvalidInputException("CipherSpec is required");
        try (org.kmipkit.protocol.CryptographicParameters params = spec.toCryptographicParameters()) {
            return cryptographicParameters(params);
        }
    }
}
