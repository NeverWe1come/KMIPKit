package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;

/** Fluent typed Activate request with optional identifier and exchange options. */
public final class ActivateBuilder extends TypedOperationBuilder<ActivateBuilder> {
    private UniqueIdentifier uniqueIdentifier;

    ActivateBuilder(KmipClient client) {
        super(client);
    }
    @Override
    protected ActivateBuilder self() {
        return this;
    }

    public ActivateBuilder uniqueIdentifier(UniqueIdentifier value) {
        ensureOpen();
        uniqueIdentifier = value;
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.activate(uniqueIdentifier));
    }
}
