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
        return bind(config, false);
    }

    public static ClientListener bindHttps(ListenerTransportConfiguration config) {
        return bind(config, true);
    }

    private static ClientListener bind(ListenerTransportConfiguration config, boolean https) {
        Objects.requireNonNull(config, "config");
        NativeExtensionRegistry.ensureLoaded();
        return new ClientListener(NativeExtensionRegistry.clientListenerBindWithOptions(
                https, config.bindHost, config.bindPort, config.flags,
                config.serverCertificates, config.serverCertificatesEncoding,
                config.serverPrivateKey, config.serverPrivateKeyEncoding,
                config.trustCertificates, config.trustCertificatesEncoding,
                config.readTimeoutMs, config.writeTimeoutMs, config.maxRequestBytes,
                config.targetPath, config.revocationLists, config.revocationListsEncoding,
                config.wireEncoding == null ? 0 : config.wireEncoding.nativeValue() + 1));
    }

    public int port() {
        return handle.withValue(NativeExtensionRegistry::clientListenerGetPort);
    }

    public ClientListenerConnection accept(Duration timeout) {
        if (timeout == null || timeout.isNegative()) {
            throw new InvalidInputException("timeout must be non-negative");
        }
        long connHandle = handle.withValue(raw ->
                NativeExtensionRegistry.clientListenerAccept(raw, timeout.toMillis()));
        return ClientListenerConnection.fromNativeHandle(connHandle);
    }

    @Override
    public void close() { handle.close(); }
}
