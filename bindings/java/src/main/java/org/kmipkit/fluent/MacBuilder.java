package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP MAC request. */
public final class MacBuilder extends CryptoOperationBuilder<MacBuilder>
        implements MacInput<MacBuilder> {
    MacBuilder(KmipClient client) {
        super(client, NATIVE_OPERATION_MAC);
    }
}
