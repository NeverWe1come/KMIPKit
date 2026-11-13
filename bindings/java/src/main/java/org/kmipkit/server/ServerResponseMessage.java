package org.kmipkit.server;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Owned batch response ready for one explicit send on its accepted connection. */
public final class ServerResponseMessage implements AutoCloseable {
    private final NativeHandle handle;
    @SuppressWarnings("unused")
    private final ServerRequestBatch owner;

    ServerResponseMessage(long nativeHandle, ServerRequestBatch owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SERVER_RESPONSE_MESSAGE);
        this.owner = owner;
    }

    NativeHandle nativeHandle() { return handle; }
    boolean belongsTo(ServerConnection connection) { return owner.belongsTo(connection); }
    @Override public void close() { handle.close(); }
}
