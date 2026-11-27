package org.kmipkit.fluent;

import java.util.Arrays;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;

/** Fluent builder for a typed KMIP RNG Seed request. */
public final class RngSeedBuilder extends TypedOperationBuilder<RngSeedBuilder> {
    private byte[] data;

    RngSeedBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected RngSeedBuilder self() { return this; }

    public RngSeedBuilder data(byte[] seed) {
        ensureOpen();
        if (seed == null) throw new InvalidInputException("RNG Seed data is required");
        data = Arrays.copyOf(seed, seed.length);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (data == null) throw new InvalidInputException("RNG Seed data is required");
        return validated(ClientBatchItem.rngSeed(data));
    }
}
