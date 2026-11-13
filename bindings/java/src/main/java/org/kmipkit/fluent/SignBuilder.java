package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP Sign request. */
public final class SignBuilder extends CryptoOperationBuilder<SignBuilder>
        implements SignInput<SignBuilder> {
    SignBuilder(KmipClient client) {
        super(client, NATIVE_OPERATION_SIGN);
    }
}
