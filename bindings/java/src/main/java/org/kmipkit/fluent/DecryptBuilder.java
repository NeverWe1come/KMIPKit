package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP Decrypt request. */
public final class DecryptBuilder extends CryptoOperationBuilder<DecryptBuilder>
        implements DecryptInput<DecryptBuilder> {
    DecryptBuilder(KmipClient client) {
        super(client, 1);
    }
}
