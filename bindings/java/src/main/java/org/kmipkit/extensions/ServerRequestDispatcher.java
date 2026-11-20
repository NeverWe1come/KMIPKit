package org.kmipkit.extensions;

import java.util.Objects;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.message.ResponseMessage;

/** Dispatches IncomingServerBatch items to a ServerRequestHandler using C ABI dispatch_v4. */
public final class ServerRequestDispatcher implements AutoCloseable {
    private final NativeHandle handle;

    private ServerRequestDispatcher(long nativeHandle) {
        this.handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SERVER_REQUEST_DISPATCHER);
    }

    public static ServerRequestDispatcher create() {
        NativeExtensionRegistry.ensureLoaded();
        return new ServerRequestDispatcher(NativeExtensionRegistry.serverRequestDispatcherCreate());
    }

    public ResponseMessage dispatch(IncomingServerBatch batch, ServerRequestHandler handler) {
        Objects.requireNonNull(batch, "batch");
        Objects.requireNonNull(handler, "handler");
        NativeExtensionRegistry.ensureLoaded();
        long responseHandle = handle.withValue(dispatcher ->
                batch.nativeHandleOwner().withValue(rawBatch ->
                        NativeExtensionRegistry.serverRequestDispatcherDispatchV3(dispatcher, rawBatch, handler)));
        return ResponseMessage.fromNative(responseHandle);
    }

    @Override
    public void close() { handle.close(); }
}
