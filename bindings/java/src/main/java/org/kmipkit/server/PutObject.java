package org.kmipkit.server;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Owned view of the managed object in a received Put request. */
public final class PutObject implements AutoCloseable {
    private final NativeHandle handle;
    @SuppressWarnings("unused")
    private final PutRequest owner;

    PutObject(long nativeHandle, PutRequest owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SERVER_PUT_OBJECT);
        this.owner = owner;
    }

    public int objectType() { return handle.withValue(ServerNative::putObjectType); }
    public int tag() { return handle.withValue(ServerNative::putObjectTag); }
    @Override public void close() { handle.close(); }
}
