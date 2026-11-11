package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP Hash request. */
public final class HashBuilder extends CryptoOperationBuilder<HashBuilder>
        implements HashInput<HashBuilder> {
    HashBuilder(KmipClient client) {
        super(client, NATIVE_OPERATION_HASH);
    }
}
