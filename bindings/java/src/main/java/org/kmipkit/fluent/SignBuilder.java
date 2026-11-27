package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP Sign request. */
public final class SignBuilder extends CryptoOperationBuilder<SignBuilder>
        implements SignInput<SignBuilder> {
    SignBuilder(KmipClient client) {
        super(client, NATIVE_OPERATION_SIGN);
    }

    /** Configures signature parameters from a curated {@link SignatureSpec}. */
    public SignBuilder withSignatureSpec(SignatureSpec spec) {
        if (spec == null) throw new org.kmipkit.InvalidInputException("SignatureSpec is required");
        try (org.kmipkit.protocol.CryptographicParameters params = spec.toCryptographicParameters()) {
            return cryptographicParameters(params);
        }
    }
}
