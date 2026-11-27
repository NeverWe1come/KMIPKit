package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;

/** Fluent builder for a typed KMIP Check request. */
public final class CheckBuilder extends TypedOperationBuilder<CheckBuilder> {
    private UniqueIdentifier uniqueIdentifier;
    private Long usageLimitsCount;
    private Integer cryptographicUsageMask;
    private Long leaseTime;

    CheckBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected CheckBuilder self() { return this; }

    public CheckBuilder uniqueIdentifier(UniqueIdentifier id) {
        ensureOpen();
        uniqueIdentifier = id;
        return this;
    }

    public CheckBuilder usageLimitsCount(long count) {
        ensureOpen();
        usageLimitsCount = count;
        return this;
    }

    public CheckBuilder cryptographicUsageMask(int mask) {
        ensureOpen();
        cryptographicUsageMask = mask;
        return this;
    }

    public CheckBuilder leaseTime(long seconds) {
        ensureOpen();
        leaseTime = unsigned32(seconds, "Lease Time");
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.check(uniqueIdentifier, usageLimitsCount, cryptographicUsageMask, leaseTime));
    }
}
