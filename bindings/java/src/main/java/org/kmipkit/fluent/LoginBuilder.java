package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP Login request (§6.1.30). */
public final class LoginBuilder extends TypedOperationBuilder<LoginBuilder> {
    private Long leaseTimeInterval;
    private Long leaseTimeDateTime;
    private Integer requestCount;
    private TtlvStructure usageLimits;

    LoginBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected LoginBuilder self() { return this; }

    public LoginBuilder leaseTimeInterval(long seconds) {
        ensureOpen();
        leaseTimeInterval = unsigned32(seconds, "Lease Time Interval");
        leaseTimeDateTime = null;
        return this;
    }

    public LoginBuilder leaseTimeDateTime(long unixSeconds) {
        ensureOpen();
        leaseTimeDateTime = unixSeconds;
        leaseTimeInterval = null;
        return this;
    }

    public LoginBuilder requestCount(int count) {
        ensureOpen();
        requestCount = count;
        return this;
    }

    public LoginBuilder usageLimits(TtlvStructure value) {
        usageLimits = replace(usageLimits, value);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.login(
                leaseTimeInterval, leaseTimeDateTime, requestCount, usageLimits));
    }
}
