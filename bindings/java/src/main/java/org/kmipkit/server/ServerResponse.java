package org.kmipkit.server;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Typed success payload paired with a request of the matching operation. */
public class ServerResponse implements AutoCloseable {
    private final NativeHandle handle;
    private final ServerRequest request;

    ServerResponse(long nativeHandle, ServerRequest request) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SERVER_RESPONSE_ITEM);
        this.request = request;
    }

    long nativeHandle() { return handle.get(); }
    NativeHandle handleForTransfer() { return handle; }
    ServerRequest request() { return request; }
    public int operation() { return request.operation(); }
    @Override public void close() { handle.close(); }
}
