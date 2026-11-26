package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;

/** Fluent builder for a typed KMIP Get Usage Allocation request. */
public final class GetUsageAllocationBuilder extends TypedOperationBuilder<GetUsageAllocationBuilder> {
    private UniqueIdentifier uniqueIdentifier;
    private Long usageLimitsCount;

    GetUsageAllocationBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected GetUsageAllocationBuilder self() { return this; }

    public GetUsageAllocationBuilder uniqueIdentifier(UniqueIdentifier id) {
        ensureOpen();
        uniqueIdentifier = id;
        return this;
    }

    public GetUsageAllocationBuilder usageLimitsCount(long count) {
        ensureOpen();
        usageLimitsCount = count;
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (usageLimitsCount == null) throw new InvalidInputException("usageLimitsCount is required");
        return validated(ClientBatchItem.getUsageAllocation(uniqueIdentifier, usageLimitsCount));
    }
}
