package org.kmipkit.server;

import org.kmipkit.InvalidInputException;

/** Typed empty success response to Put. */
public final class PutResponse extends ServerResponse {
    private PutResponse(long handle, PutRequest request) { super(handle, request); }

    public static PutResponse forRequest(PutRequest request) {
        if (request == null) throw new InvalidInputException("request is required");
        return new PutResponse(ServerNative.responsePut(request.nativeHandle()), request);
    }
}
