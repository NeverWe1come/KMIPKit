package org.kmipkit.server;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Common typed view of a server-originated request; it has no client send method. */
public class ServerRequest implements AutoCloseable {
    private final NativeHandle handle;
    private final ServerRequestBatch owner;
    private final int operation;

    ServerRequest(long nativeHandle, ServerRequestBatch owner, int operation) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SERVER_REQUEST);
        this.owner = owner;
        this.operation = operation;
    }

    public int operation() { return operation; }
    public byte[] uniqueIdentifier() { return handle.withValue(ServerNative::requestUniqueIdentifier); }

    long nativeHandle() { return handle.get(); }
    NativeHandle handleForTransfer() { return handle; }
    ServerRequestBatch owner() { return owner; }

    @Override
    public void close() { handle.close(); }
}
