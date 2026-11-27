package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;

/** Fluent builder for a typed KMIP Set Endpoint Role request (§6.1.54). */
public final class SetEndpointRoleBuilder extends TypedOperationBuilder<SetEndpointRoleBuilder> {
    private Long endpointRole;
    private String vendorIdentification;

    SetEndpointRoleBuilder(KmipClient client, Long endpointRole, String vendorIdentification) {
        super(client);
        this.endpointRole = endpointRole == null ? null : unsigned32(endpointRole, "Endpoint Role");
        this.vendorIdentification = vendorIdentification;
    }

    @Override
    protected SetEndpointRoleBuilder self() { return this; }

    public SetEndpointRoleBuilder endpointRole(long role) {
        ensureOpen();
        endpointRole = unsigned32(role, "Endpoint Role");
        return this;
    }

    public SetEndpointRoleBuilder vendorIdentification(String vendorId) {
        ensureOpen();
        vendorIdentification = vendorId;
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (endpointRole == null) {
            throw new InvalidInputException("Set Endpoint Role requires endpointRole");
        }
        return validated(ClientBatchItem.setEndpointRole(endpointRole, vendorIdentification));
    }
}
