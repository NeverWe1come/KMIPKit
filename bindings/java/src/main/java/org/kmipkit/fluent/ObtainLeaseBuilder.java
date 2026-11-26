package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;

/** Fluent builder for a typed KMIP Obtain Lease request. */
public final class ObtainLeaseBuilder extends TypedOperationBuilder<ObtainLeaseBuilder> {
    private UniqueIdentifier uniqueIdentifier;

    ObtainLeaseBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected ObtainLeaseBuilder self() { return this; }

    public ObtainLeaseBuilder uniqueIdentifier(UniqueIdentifier id) {
        ensureOpen();
        uniqueIdentifier = id;
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.obtainLease(uniqueIdentifier));
    }
}
