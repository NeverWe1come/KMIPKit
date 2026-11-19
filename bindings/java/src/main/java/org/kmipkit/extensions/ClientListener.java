package org.kmipkit.extensions;

import java.time.Duration;
import java.util.Objects;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Server-to-Client listener bound to TLS or HTTPS. */
public final class ClientListener implements AutoCloseable {
    private final NativeHandle handle;

    private ClientListener(long nativeHandle) {
        this.handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CLIENT_LISTENER);
    }

    public static ClientListener bindRawTls(ListenerTransportConfiguration config) {
        Objects.requireNonNull(config, "config");
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = NativeExtensionRegistry.clientListenerBindRawTls(
                config.bindHost, config.bindPort, config.flags,
                config.serverCertificates, config.serverCertificatesEncoding,
                config.serverPrivateKey, config.serverPrivateKeyEncoding,
                config.trustCertificates, config.trustCertificatesEncoding,
                config.readTimeoutMs, config.writeTimeoutMs, config.maxRequestBytes);
        return new ClientListener(nativeHandle);
    }

    public static ClientListener bindHttps(ListenerTransportConfiguration config) {
        Objects.requireNonNull(config, "config");
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = NativeExtensionRegistry.clientListenerBindHttps(
                config.bindHost, config.bindPort, config.flags,
                config.serverCertificates, config.serverCertificatesEncoding,
                config.serverPrivateKey, config.serverPrivateKeyEncoding,
                config.trustCertificates, config.trustCertificatesEncoding,
                config.readTimeoutMs, config.writeTimeoutMs, config.maxRequestBytes);
        return new ClientListener(nativeHandle);
    }

    public int port() {
        return handle.withValue(NativeExtensionRegistry::clientListenerGetPort);
    }

    public ClientListenerConnection accept(Duration timeout) {
        Objects.requireNonNull(timeout, "timeout");
        long connHandle = handle.withValue(raw ->
                NativeExtensionRegistry.clientListenerAccept(raw, timeout.toMillis()));
        return ClientListenerConnection.fromNativeHandle(connHandle);
    }

    @Override
    public void close() { handle.close(); }
}
