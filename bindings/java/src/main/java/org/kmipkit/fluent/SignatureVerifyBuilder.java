package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP Signature Verify request. */
public final class SignatureVerifyBuilder extends CryptoOperationBuilder<SignatureVerifyBuilder>
        implements SignatureVerifyInput<SignatureVerifyBuilder> {
    SignatureVerifyBuilder(KmipClient client) {
        super(client, 3);
    }
}
