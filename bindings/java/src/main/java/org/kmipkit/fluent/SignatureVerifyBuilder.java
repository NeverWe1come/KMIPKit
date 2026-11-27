package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP Signature Verify request. */
public final class SignatureVerifyBuilder extends CryptoOperationBuilder<SignatureVerifyBuilder>
        implements SignatureVerifyInput<SignatureVerifyBuilder> {
    SignatureVerifyBuilder(KmipClient client) {
        super(client, NATIVE_OPERATION_SIGNATURE_VERIFY);
    }

    /** Configures verification parameters from a curated {@link SignatureSpec}. */
    public SignatureVerifyBuilder withSignatureSpec(SignatureSpec spec) {
        if (spec == null) throw new org.kmipkit.InvalidInputException("SignatureSpec is required");
        try (org.kmipkit.protocol.CryptographicParameters params = spec.toCryptographicParameters()) {
            return cryptographicParameters(params);
        }
    }
}
