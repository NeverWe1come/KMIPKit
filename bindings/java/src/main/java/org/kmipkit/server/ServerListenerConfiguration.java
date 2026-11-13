package org.kmipkit.server;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Owned TLS and wire configuration for a KMIP server listener. */
public final class ServerListenerConfiguration implements AutoCloseable {
    private final NativeHandle handle;
    private final ListenerTransport transport;

    private ServerListenerConfiguration(long nativeHandle, ListenerTransport transport) {
        handle = NativeHandle.owned(nativeHandle,
                NativeExtensionRegistry.SERVER_LISTENER_CONFIGURATION);
        this.transport = transport;
    }

    public static Builder builder() { return new Builder(); }

    public ListenerTransport transport() { return transport; }

    long nativeHandle() { return handle.get(); }

    @Override
    public void close() { handle.close(); }

    /** Collects validated listener settings and creates an owned native configuration. */
    public static final class Builder {
        private ListenerTransport transport = ListenerTransport.RAW_TLS;
        private WireEncoding wireEncoding = WireEncoding.TTLV;
        private CertificateEncoding certificateEncoding = CertificateEncoding.PEM;
        private String host = "127.0.0.1";
        private int port;
        private byte[][] certificateChain;
        private byte[] privateKey;
        private byte[][] trustCertificates;
        private byte[][] revocationLists = new byte[0][];
        private String targetPath = "";
        private long maxRequestBytes;

        private Builder() {}

        public Builder transport(ListenerTransport value) {
            transport = require(value, "transport");
            return this;
        }

        public Builder wireEncoding(WireEncoding value) {
            wireEncoding = require(value, "wire encoding");
            return this;
        }

        public Builder certificateEncoding(CertificateEncoding value) {
            certificateEncoding = require(value, "certificate encoding");
            return this;
        }

        public Builder bindHost(String value) {
            if (value == null || value.isBlank() || value.indexOf('\0') >= 0) {
                throw new InvalidInputException("listener bind host is invalid");
            }
            host = value;
            return this;
        }

        public Builder bindPort(int value) {
            if (value < 0 || value > 65535) {
                throw new InvalidInputException("listener port is outside the valid range");
            }
            port = value;
            return this;
        }

        public Builder listenerCertificateChain(List<byte[]> value) {
            certificateChain = copyInputs(value, false, "listener certificate chain");
            return this;
        }

        public Builder listenerPrivateKey(byte[] value) {
            if (value == null || value.length == 0) {
                throw new InvalidInputException("listener private key is empty");
            }
            privateKey = value.clone();
            return this;
        }

        public Builder trustCertificates(List<byte[]> value) {
            trustCertificates = copyInputs(value, false, "trust certificates");
            return this;
        }

        public Builder revocationLists(List<byte[]> value) {
            revocationLists = copyInputs(value, true, "revocation lists");
            return this;
        }

        public Builder targetPath(String value) {
            if (value == null || value.indexOf('\0') >= 0) {
                throw new InvalidInputException("listener target path is invalid");
            }
            targetPath = value;
            return this;
        }

        public Builder maxRequestBytes(long value) {
            if (value < 0) throw new InvalidInputException("request limit cannot be negative");
            maxRequestBytes = value;
            return this;
        }

        public ServerListenerConfiguration build() {
            if (certificateChain == null || certificateChain.length == 0
                    || trustCertificates == null || trustCertificates.length == 0
                    || privateKey == null
                    || (transport == ListenerTransport.RAW_TLS && !targetPath.isEmpty())) {
                throw new InvalidInputException("listener TLS configuration is incomplete");
            }
            byte[] stagedKey = privateKey.clone();
            try {
                long nativeHandle = ServerNative.configurationCreate(
                        transport.nativeValue, wireEncoding.nativeValue, host, port,
                        certificateEncoding.nativeValue, certificateChain, stagedKey,
                        trustCertificates, revocationLists, targetPath, maxRequestBytes);
                return new ServerListenerConfiguration(nativeHandle, transport);
            } finally {
                java.util.Arrays.fill(stagedKey, (byte) 0);
                java.util.Arrays.fill(privateKey, (byte) 0);
                privateKey = null;
            }
        }

        private static byte[][] copyInputs(List<byte[]> values, boolean allowEmpty, String name) {
            if (values == null || values.size() > 4096 || (!allowEmpty && values.isEmpty())) {
                throw new InvalidInputException(name + " are invalid");
            }
            List<byte[]> copies = new ArrayList<>(values.size());
            for (byte[] value : values) {
                if (value == null || value.length == 0) {
                    throw new InvalidInputException(name + " contain an empty entry");
                }
                copies.add(value.clone());
            }
            return copies.toArray(byte[][]::new);
        }

        private static <T> T require(T value, String name) {
            if (value == null) throw new InvalidInputException(name + " is required");
            return value;
        }
    }
}
