package org.kmipkit.server;

import org.kmipkit.InvalidInputException;

/** Typed Set Endpoint Role response. */
public final class SetEndpointRoleResponse extends ServerResponse {
    private SetEndpointRoleResponse(long handle, SetEndpointRoleRequest request) { super(handle, request); }

    public static SetEndpointRoleResponse forRequest(
            SetEndpointRoleRequest request, int endpointRole, String vendorIdentification) {
        if (request == null || endpointRole < 0) throw new InvalidInputException("response fields are invalid");
        return new SetEndpointRoleResponse(ServerNative.responseEndpointRole(
                request.nativeHandle(), endpointRole, vendorIdentification), request);
    }
}
