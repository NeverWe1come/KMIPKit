package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP MAC Verify request. */
public final class MacVerifyBuilder extends CryptoOperationBuilder<MacVerifyBuilder>
        implements MacVerifyInput<MacVerifyBuilder> {
    MacVerifyBuilder(KmipClient client) {
        super(client, NATIVE_OPERATION_MAC_VERIFY);
    }
}
