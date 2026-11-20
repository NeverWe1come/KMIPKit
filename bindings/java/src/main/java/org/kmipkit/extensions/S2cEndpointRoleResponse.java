package org.kmipkit.extensions;

import org.jspecify.annotations.Nullable;
import org.kmipkit.InvalidInputException;

/** Mutable successful Set Endpoint Role response populated by a server request handler. */
public final class S2cEndpointRoleResponse {
    private long endpointRole;
    private @Nullable String vendorIdentification;

    public S2cEndpointRoleResponse(long endpointRole) {
        endpointRole(endpointRole);
    }

    public long endpointRole() { return endpointRole; }

    public S2cEndpointRoleResponse endpointRole(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("Endpoint Role is outside uint32");
        }
        endpointRole = value;
        return this;
    }

    public @Nullable String vendorIdentification() { return vendorIdentification; }

    public S2cEndpointRoleResponse vendorIdentification(@Nullable String value) {
        vendorIdentification = value;
        return this;
    }
}
