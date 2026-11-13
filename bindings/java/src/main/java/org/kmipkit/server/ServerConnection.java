package org.kmipkit.server;

import java.time.Duration;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;

/** An accepted listener connection with explicit bounded receive and response calls. */
public final class ServerConnection implements AutoCloseable {
    private final NativeHandle handle;
    @SuppressWarnings("unused")
    private final ServerListener owner;

    ServerConnection(long nativeHandle, ServerListener owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SERVER_CONNECTION);
        this.owner = owner;
    }

    public ServerRequestBatch receive(CodecLimits limits, Duration timeout) {
        CodecLimits selected = limits == null ? CodecLimits.defaults() : limits;
        long[] time = ServerTimeout.nativeValues(timeout);
        long[] codec = {selected.maxMessageBytes(), selected.maxStructureDepth(), selected.maxElements()};
        long batch = handle.withValue(raw -> ServerNative.connectionReceive(
                raw, codec, time[0], (int) time[1]));
        return new ServerRequestBatch(batch, this);
    }

    public void send(ServerResponseMessage response, Duration timeout) {
        if (response == null || !response.belongsTo(this)) {
            throw new InvalidInputException("response was built for another connection");
        }
        long[] time = ServerTimeout.nativeValues(timeout);
        NativeHandle.withValues(new NativeHandle[] {handle, response.nativeHandle()}, handles -> {
            ServerNative.connectionSend(handles[0], handles[1], time[0], (int) time[1]);
            return null;
        });
    }

    @Override
    public void close() { handle.close(); }
}
