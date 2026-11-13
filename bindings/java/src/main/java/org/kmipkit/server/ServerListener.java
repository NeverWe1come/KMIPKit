package org.kmipkit.server;

import java.time.Duration;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Bound KMIP listener. Accepts only authenticated TLS connections. */
public final class ServerListener implements AutoCloseable {
    private final NativeHandle handle;
    private final ServerListenerConfiguration configuration;

    private ServerListener(long nativeHandle, ServerListenerConfiguration configuration) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SERVER_LISTENER);
        this.configuration = configuration;
    }

    public static ServerListener bindRawTls(ServerListenerConfiguration configuration) {
        if (configuration == null) throw new InvalidInputException("listener configuration is required");
        if (configuration.transport() != ListenerTransport.RAW_TLS) {
            throw new IllegalArgumentException("configuration transport must be RAW_TLS");
        }
        return new ServerListener(
                ServerNative.listenerBind(configuration.nativeHandle(), false), configuration);
    }

    public static ServerListener bindHttps(ServerListenerConfiguration configuration) {
        if (configuration == null) throw new InvalidInputException("listener configuration is required");
        if (configuration.transport() != ListenerTransport.HTTPS) {
            throw new IllegalArgumentException("configuration transport must be HTTPS");
        }
        return new ServerListener(
                ServerNative.listenerBind(configuration.nativeHandle(), true), configuration);
    }

    public int port() { return handle.withValue(ServerNative::listenerPort); }

    public ServerConnection accept(Duration timeout) {
        long[] value = ServerTimeout.nativeValues(timeout);
        long connection = handle.withValue(raw -> ServerNative.listenerAccept(raw, value[0], (int) value[1]));
        return new ServerConnection(connection, this);
    }

    @Override
    public void close() { handle.close(); }
}
