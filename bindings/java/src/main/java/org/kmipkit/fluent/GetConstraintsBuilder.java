package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;

/** Fluent builder for a typed KMIP Get Constraints request (§6.1.22). */
public final class GetConstraintsBuilder extends TypedOperationBuilder<GetConstraintsBuilder> {
    GetConstraintsBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected GetConstraintsBuilder self() { return this; }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.getConstraints());
    }
}
