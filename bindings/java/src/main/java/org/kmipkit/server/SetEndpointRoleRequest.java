package org.kmipkit.server;

/** Received Set Endpoint Role request. */
public final class SetEndpointRoleRequest extends ServerRequest {
    SetEndpointRoleRequest(long handle, ServerRequestBatch owner) { super(handle, owner, 50); }
    public int endpointRole() { return ServerNative.endpointRole(nativeHandle()); }
}
