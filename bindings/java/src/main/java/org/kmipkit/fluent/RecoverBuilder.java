package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;

/** Fluent typed Recover request with optional identifier and exchange options. */
public final class RecoverBuilder extends TypedOperationBuilder<RecoverBuilder> {
    private UniqueIdentifier uniqueIdentifier;

    RecoverBuilder(KmipClient client) {
        super(client);
    }
    @Override
    protected RecoverBuilder self() {
        return this;
    }

    public RecoverBuilder uniqueIdentifier(UniqueIdentifier value) {
        ensureOpen();
        uniqueIdentifier = value;
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.recover(uniqueIdentifier));
    }
}
