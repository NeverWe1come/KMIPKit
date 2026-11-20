package org.kmipkit.extensions;

import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Listener TLS / mTLS configuration for receiving Server-to-Client connections. */
public final class ListenerTransportConfiguration implements AutoCloseable {
    final String bindHost;
    final int bindPort;
    final int flags;
    final byte[][] serverCertificates;
    final int serverCertificatesEncoding;
    final byte[] serverPrivateKey;
    final int serverPrivateKeyEncoding;
    final byte[][] trustCertificates;
    final int trustCertificatesEncoding;
    final long readTimeoutMs;
    final long writeTimeoutMs;
    final long maxRequestBytes;
    final String targetPath;
    final byte[][] revocationLists;
    final int revocationListsEncoding;
    final WireEncoding wireEncoding;
    private boolean closed;

    private ListenerTransportConfiguration(
            String bindHost,
            int bindPort,
            int flags,
            byte[][] serverCertificates,
            int serverCertificatesEncoding,
            byte[] serverPrivateKey,
            int serverPrivateKeyEncoding,
            byte[][] trustCertificates,
            int trustCertificatesEncoding,
            long readTimeoutMs,
            long writeTimeoutMs,
            long maxRequestBytes, String targetPath, byte[][] revocationLists,
            int revocationListsEncoding, WireEncoding wireEncoding) {
        this.bindHost = bindHost;
        this.bindPort = bindPort;
        this.flags = flags;
        this.serverCertificates = serverCertificates;
        this.serverCertificatesEncoding = serverCertificatesEncoding;
        this.serverPrivateKey = serverPrivateKey;
        this.serverPrivateKeyEncoding = serverPrivateKeyEncoding;
        this.trustCertificates = trustCertificates;
        this.trustCertificatesEncoding = trustCertificatesEncoding;
        this.readTimeoutMs = readTimeoutMs;
        this.writeTimeoutMs = writeTimeoutMs;
        this.maxRequestBytes = maxRequestBytes;
        this.targetPath = targetPath;
        this.revocationLists = revocationLists;
        this.revocationListsEncoding = revocationListsEncoding;
        this.wireEncoding = wireEncoding;
    }

    public String bindHost() { return bindHost; }
    public int bindPort() { return bindPort; }
    public int flags() { return flags; }
    public boolean isTls12Compat() { return (flags & 4) != 0; }

    @Override
    public synchronized void close() {
        if (!closed) {
            closed = true;
            if (serverPrivateKey != null) {
                Arrays.fill(serverPrivateKey, (byte) 0);
            }
        }
    }

    public static Builder builder() { return new Builder(); }

    public static final class Builder implements AutoCloseable {
        private String bindHost = "127.0.0.1";
        private int bindPort = 0;
        private boolean tls12Compat;
        private List<byte[]> serverCertificates = new ArrayList<>();
        private int serverCertificatesEncoding = 1; // PEM
        private byte[] serverPrivateKey;
        private int serverPrivateKeyEncoding = 1; // PEM
        private List<byte[]> trustCertificates = new ArrayList<>();
        private int trustCertificatesEncoding = 1; // PEM
        private Duration readTimeout = Duration.ofSeconds(30);
        private Duration writeTimeout = Duration.ofSeconds(30);
        private long maxRequestBytes = 16 * 1024 * 1024L;
        private String targetPath;
        private byte[][] revocationLists = new byte[0][];
        private int revocationListsEncoding = 1;
        private WireEncoding wireEncoding;
        private boolean built;

        private Builder() {}

        public Builder bind(String host, int port) {
            if (host == null || host.isBlank() || port < 0 || port > 65_535) {
                throw new InvalidInputException("invalid bind address");
            }
            this.bindHost = host;
            this.bindPort = port;
            return this;
        }

        public Builder tls12Compat(boolean enabled) {
            this.tls12Compat = enabled;
            return this;
        }

        public Builder serverIdentityPem(byte[][] certificateChain, byte[] privateKeyPem) {
            return serverIdentity(certificateChain, privateKeyPem, TransportConfiguration.Encoding.PEM);
        }

        public Builder serverIdentityDer(byte[][] certificateChain, byte[] privateKeyDer) {
            return serverIdentity(certificateChain, privateKeyDer, TransportConfiguration.Encoding.DER);
        }

        public Builder serverIdentity(byte[][] certificateChain, byte[] privateKey, TransportConfiguration.Encoding encoding) {
            if (certificateChain == null || certificateChain.length == 0 || privateKey == null) {
                throw new InvalidInputException("server identity is incomplete");
            }
            this.serverCertificates.clear();
            for (byte[] cert : certificateChain) {
                this.serverCertificates.add(cert.clone());
            }
            this.serverCertificatesEncoding = encoding == TransportConfiguration.Encoding.DER ? 2 : 1;
            this.serverPrivateKey = privateKey.clone();
            this.serverPrivateKeyEncoding = encoding == TransportConfiguration.Encoding.DER ? 2 : 1;
            return this;
        }

        public Builder trustCertificatesPem(byte[][] certificates) {
            return trustCertificates(certificates, TransportConfiguration.Encoding.PEM);
        }

        public Builder trustCertificatesDer(byte[][] certificates) {
            return trustCertificates(certificates, TransportConfiguration.Encoding.DER);
        }

        public Builder trustCertificates(byte[][] certificates, TransportConfiguration.Encoding encoding) {
            if (certificates == null) {
                throw new InvalidInputException("trust certificates cannot be null");
            }
            this.trustCertificates.clear();
            for (byte[] cert : certificates) {
                this.trustCertificates.add(cert.clone());
            }
            this.trustCertificatesEncoding = encoding == TransportConfiguration.Encoding.DER ? 2 : 1;
            return this;
        }

        public Builder readTimeout(Duration timeout) {
            if (timeout == null || timeout.isNegative()) {
                throw new InvalidInputException("readTimeout must be non-negative");
            }
            this.readTimeout = timeout;
            return this;
        }

        public Builder writeTimeout(Duration timeout) {
            if (timeout == null || timeout.isNegative()) {
                throw new InvalidInputException("writeTimeout must be non-negative");
            }
            this.writeTimeout = timeout;
            return this;
        }

        public Builder maxRequestBytes(long bytes) {
            if (bytes <= 0) {
                throw new InvalidInputException("maxRequestBytes must be positive");
            }
            this.maxRequestBytes = bytes;
            return this;
        }

        public Builder targetPath(String path) {
            if (path == null || !path.startsWith("/") || !StandardCharsets.UTF_8.newEncoder().canEncode(path)) {
                throw new InvalidInputException("invalid listener path");
            }
            this.targetPath = path;
            return this;
        }

        public Builder revocationLists(byte[][] lists, TransportConfiguration.Encoding encoding) {
            if (lists == null || encoding == null) {
                throw new InvalidInputException("revocation lists and encoding are required");
            }
            byte[][] copy = new byte[lists.length][];
            for (int i = 0; i < lists.length; i++) {
                if (lists[i] == null) throw new InvalidInputException("revocation list is required");
                copy[i] = lists[i].clone();
            }
            this.revocationLists = copy;
            this.revocationListsEncoding = encoding == TransportConfiguration.Encoding.DER ? 2 : 1;
            return this;
        }

        public Builder wireEncoding(WireEncoding encoding) {
            if (encoding == null) throw new InvalidInputException("wire encoding is required");
            this.wireEncoding = encoding;
            return this;
        }

        public ListenerTransportConfiguration build() {
            if (built) throw new InvalidInputException("builder is closed");
            built = true;
            byte[] keyCopy = serverPrivateKey != null ? serverPrivateKey.clone() : null;
            byte[][] certs = serverCertificates.toArray(new byte[0][]);
            byte[][] trust = trustCertificates.toArray(new byte[0][]);
            wipe();
            return new ListenerTransportConfiguration(
                    bindHost,
                    bindPort,
                    tls12Compat ? 4 : 0,
                    certs,
                    serverCertificatesEncoding,
                    keyCopy,
                    serverPrivateKeyEncoding,
                    trust,
                    trustCertificatesEncoding,
                    readTimeout.toMillis(),
                    writeTimeout.toMillis(),
                    maxRequestBytes, targetPath, revocationLists, revocationListsEncoding, wireEncoding);
        }

        @Override
        public void close() {
            built = true;
            wipe();
        }

        private void wipe() {
            if (serverPrivateKey != null) {
                Arrays.fill(serverPrivateKey, (byte) 0);
                serverPrivateKey = null;
            }
            serverCertificates.clear();
            trustCertificates.clear();
        }
    }
}
