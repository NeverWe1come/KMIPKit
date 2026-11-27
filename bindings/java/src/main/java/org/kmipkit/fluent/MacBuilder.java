package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;

/** Rust-backed fluent builder for a typed KMIP MAC request. */
public final class MacBuilder extends CryptoOperationBuilder<MacBuilder>
        implements MacInput<MacBuilder> {
    MacBuilder(KmipClient client) {
        super(client, NATIVE_OPERATION_MAC);
    }

    /** Configures MAC parameters from a curated {@link MacSpec}. */
    public MacBuilder withMacSpec(MacSpec spec) {
        if (spec == null) throw new org.kmipkit.InvalidInputException("MacSpec is required");
        try (org.kmipkit.protocol.CryptographicParameters params = spec.toCryptographicParameters()) {
            return cryptographicParameters(params);
        }
    }
}
