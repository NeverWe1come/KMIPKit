package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;

/** Fluent typed Archive request with optional identifier and exchange options. */
public final class ArchiveBuilder extends TypedOperationBuilder<ArchiveBuilder> {
    private UniqueIdentifier uniqueIdentifier;

    ArchiveBuilder(KmipClient client) {
        super(client);
    }
    @Override
    protected ArchiveBuilder self() {
        return this;
    }

    public ArchiveBuilder uniqueIdentifier(UniqueIdentifier value) {
        ensureOpen();
        uniqueIdentifier = value;
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.archive(uniqueIdentifier));
    }
}
