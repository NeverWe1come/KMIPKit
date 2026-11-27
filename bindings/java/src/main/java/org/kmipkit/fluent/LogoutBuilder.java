package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP Logout request (§6.1.31). */
public final class LogoutBuilder extends TypedOperationBuilder<LogoutBuilder> {
    private TtlvStructure ticket;

    LogoutBuilder(KmipClient client, TtlvStructure ticket) {
        super(client);
        if (ticket != null) {
            this.ticket = replace(null, ticket);
        }
    }

    @Override
    protected LogoutBuilder self() { return this; }

    public LogoutBuilder ticket(TtlvStructure value) {
        if (value == null) throw new InvalidInputException("ticket is required");
        ticket = replace(ticket, value);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (ticket == null) throw new InvalidInputException("Logout ticket is required");
        return validated(ClientBatchItem.logout(ticket));
    }
}
