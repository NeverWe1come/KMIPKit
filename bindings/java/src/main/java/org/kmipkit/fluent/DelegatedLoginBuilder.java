package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP Delegated Login request (§6.1.12). */
public final class DelegatedLoginBuilder extends TypedOperationBuilder<DelegatedLoginBuilder> {
    private TtlvStructure rights;
    private Long leaseTimeInterval;
    private Long leaseTimeDateTime;
    private Integer requestCount;
    private TtlvStructure usageLimits;

    DelegatedLoginBuilder(KmipClient client, TtlvStructure rights) {
        super(client);
        if (rights != null) {
            this.rights = replace(null, rights);
        }
    }

    @Override
    protected DelegatedLoginBuilder self() { return this; }

    public DelegatedLoginBuilder rights(TtlvStructure value) {
        if (value == null) throw new InvalidInputException("rights is required");
        rights = replace(rights, value);
        return this;
    }

    public DelegatedLoginBuilder leaseTimeInterval(long seconds) {
        ensureOpen();
        leaseTimeInterval = unsigned32(seconds, "Lease Time Interval");
        leaseTimeDateTime = null;
        return this;
    }

    public DelegatedLoginBuilder leaseTimeDateTime(long unixSeconds) {
        ensureOpen();
        leaseTimeDateTime = unixSeconds;
        leaseTimeInterval = null;
        return this;
    }

    public DelegatedLoginBuilder requestCount(int count) {
        ensureOpen();
        requestCount = count;
        return this;
    }

    public DelegatedLoginBuilder usageLimits(TtlvStructure value) {
        usageLimits = replace(usageLimits, value);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (rights == null) throw new InvalidInputException("Delegated Login rights is required");
        return validated(ClientBatchItem.delegatedLogin(
                rights, leaseTimeInterval, leaseTimeDateTime, requestCount, usageLimits));
    }
}
