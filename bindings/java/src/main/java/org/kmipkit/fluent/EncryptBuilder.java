package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP Encrypt request. */
public final class EncryptBuilder extends CryptoOperationBuilder<EncryptBuilder>
        implements CipherInput<EncryptBuilder> {
    EncryptBuilder(KmipClient client) {
        super(client, 0);
    }
}
