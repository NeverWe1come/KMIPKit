package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP Hash request. */
public final class HashBuilder extends CryptoOperationBuilder<HashBuilder>
        implements HashInput<HashBuilder> {
    HashBuilder(KmipClient client) {
        super(client, NATIVE_OPERATION_HASH);
    }

    /** Configures hash algorithm parameters from a curated {@link HashAlgorithm}. */
    public HashBuilder withHashAlgorithm(HashAlgorithm alg) {
        if (alg == null) throw new org.kmipkit.InvalidInputException("HashAlgorithm is required");
        try (org.kmipkit.protocol.CryptographicParameters params = alg.toCryptographicParameters()) {
            return cryptographicParameters(params);
        }
    }
}
