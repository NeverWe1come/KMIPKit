package org.kmipkit.server;

import org.kmipkit.InvalidInputException;

/** Typed empty success response to Notify. */
public final class NotifyResponse extends ServerResponse {
    private NotifyResponse(long handle, NotifyRequest request) { super(handle, request); }

    public static NotifyResponse forRequest(NotifyRequest request) {
        if (request == null) throw new InvalidInputException("request is required");
        return new NotifyResponse(ServerNative.responseNotify(request.nativeHandle()), request);
    }
}
