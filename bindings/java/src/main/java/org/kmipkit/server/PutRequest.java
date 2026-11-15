package org.kmipkit.server;

/** Received Put request carrying an inspectable KMIP managed object. */
public final class PutRequest extends ServerRequest {
    PutRequest(long handle, ServerRequestBatch owner) { super(handle, owner, 28); }

    public int putFunction() { return ServerNative.putFunction(nativeHandle()); }
    public PutObject putObject() { return new PutObject(ServerNative.putObject(nativeHandle()), this); }
}
