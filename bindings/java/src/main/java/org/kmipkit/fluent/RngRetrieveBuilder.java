package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;

/** Fluent builder for a typed KMIP RNG Retrieve request. */
public final class RngRetrieveBuilder extends TypedOperationBuilder<RngRetrieveBuilder> {
    private Integer dataLength;

    RngRetrieveBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected RngRetrieveBuilder self() { return this; }

    public RngRetrieveBuilder dataLength(int length) {
        ensureOpen();
        dataLength = length;
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (dataLength == null) throw new InvalidInputException("RNG Retrieve dataLength is required");
        return validated(ClientBatchItem.rngRetrieve(dataLength));
    }
}
