package org.kmipkit.extensions;

import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;

/** Validated TLS 1.3 transport inputs consumed when constructing a client. */
public final class TransportConfiguration implements AutoCloseable {
    private static final long MAX_INPUT_BYTES = 16 * 1024 * 1024L;
    private static final int RAW_TLS = 1;
    private static final int HTTPS = 2;
    private static final int TRUST_PLATFORM = 1;
    private static final int TRUST_CERTIFICATES = 2;
    private static final int SOURCE_MEMORY = 1;
    private static final int SOURCE_FILE = 2;

    /** Encoding used for certificate, key, and revocation-list material. */
    public enum Encoding {
        PEM(1),
        DER(2);

        private final int nativeValue;

        Encoding(int nativeValue) {
            this.nativeValue = nativeValue;
        }
    }

    /** A timeout may inherit the transport default, be bounded, or be unbounded. */
    public record Timeout(int kind, long seconds, int nanoseconds) {
        public Timeout {
            if (kind < 0 || kind > 2 || seconds < 0 || nanoseconds < 0
                    || nanoseconds >= 1_000_000_000 || (kind == 2 && (seconds != 0 || nanoseconds != 0))) {
                throw new InvalidInputException("transport timeout is invalid");
            }
        }

        /** Inherits the timeout selected by the accepted transport policy. */
        public static Timeout defaultValue() {
            return new Timeout(0, 0, 0);
        }

        /** Uses the specified non-negative timeout. */
        public static Timeout bounded(Duration duration) {
            if (duration == null || duration.isNegative()) {
                throw new InvalidInputException("transport timeout is invalid");
            }
            return new Timeout(1, duration.getSeconds(), duration.getNano());
        }

        /** Disables the specified timeout. */
        public static Timeout unbounded() {
            return new Timeout(2, 0, 0);
        }
    }

    private final NativeHandle handle;

    private TransportConfiguration(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TRANSPORT_CONFIGURATION);
    }

    /** Starts a builder for a mutually authenticated raw TTLV-over-TLS endpoint. */
    public static Builder builder() {
        return new Builder();
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    @Override
    public void close() {
        handle.close();
    }

    /** Builder for the accepted KMIPKIT-0013 endpoint, trust, identity, and timeout inputs. */
    public static final class Builder {
        private int endpointKind;
        private String endpoint;
        private int endpointPort;
        private int trustKind;
        private List<Input> clientCertificates = List.of();
        private Input privateKey;
        private List<Input> trustCertificates = List.of();
        private List<Input> revocationLists = List.of();
        private String tlsServerName;
        private String targetUri;
        private Timeout connectTimeout = Timeout.defaultValue();
        private Timeout writeTimeout = Timeout.defaultValue();
        private Timeout readTimeout = Timeout.defaultValue();
        private Timeout totalTimeout = Timeout.defaultValue();
        private long maxRequestBytes = 16 * 1024 * 1024L;
        private boolean built;

        private Builder() {}

        /** Selects a raw TTLV-over-TLS endpoint. */
        public Builder rawTls(String host, int port) {
            ensureMutable();
            if (host == null || host.isBlank() || port < 1 || port > 65_535) {
                throw new InvalidInputException("raw TLS endpoint is invalid");
            }
            endpointKind = RAW_TLS;
            endpoint = host;
            endpointPort = port;
            return this;
        }

        /** Selects an HTTPS endpoint; redirects and proxies remain disabled. */
        public Builder https(String target) {
            ensureMutable();
            if (target == null || target.isBlank()) {
                throw new InvalidInputException("HTTPS endpoint is invalid");
            }
            endpointKind = HTTPS;
            endpoint = target;
            endpointPort = 0;
            return this;
        }

        /** Supplies a PEM client-certificate chain and an in-memory PEM private key. */
        public Builder clientIdentityPem(byte[][] certificateChain, byte[] privateKeyPem) {
            return clientIdentity(certificateChain, privateKeyPem, Encoding.PEM);
        }

        /** Supplies a DER client-certificate chain and an in-memory DER private key. */
        public Builder clientIdentityDer(byte[][] certificateChain, byte[] privateKeyDer) {
            return clientIdentity(certificateChain, privateKeyDer, Encoding.DER);
        }

        /** Supplies client identity material from files using the selected encoding. */
        public Builder clientIdentityFiles(List<Path> certificateChain, Path privateKeyFile,
                Encoding encoding) {
            ensureMutable();
            clientCertificates = fileInputs(certificateChain, encoding);
            privateKey = fileInput(privateKeyFile, encoding, true);
            return this;
        }

        /** Uses platform trust roots. */
        public Builder trustPlatform() {
            ensureMutable();
            trustKind = TRUST_PLATFORM;
            trustCertificates = List.of();
            return this;
        }

        /** Supplies in-memory PEM trust certificates. */
        public Builder trustCertificatesPem(byte[][] certificates) {
            return trustCertificates(certificates, Encoding.PEM);
        }

        /** Supplies in-memory DER trust certificates. */
        public Builder trustCertificatesDer(byte[][] certificates) {
            return trustCertificates(certificates, Encoding.DER);
        }

        /** Supplies trust certificates from files using the selected encoding. */
        public Builder trustCertificateFiles(List<Path> certificates, Encoding encoding) {
            ensureMutable();
            trustKind = TRUST_CERTIFICATES;
            trustCertificates = fileInputs(certificates, encoding);
            return this;
        }

        /** Supplies in-memory PEM revocation lists. */
        public Builder revocationListsPem(byte[][] lists) {
            return revocationLists(lists, Encoding.PEM);
        }

        /** Supplies in-memory DER revocation lists. */
        public Builder revocationListsDer(byte[][] lists) {
            return revocationLists(lists, Encoding.DER);
        }

        /** Sets the TLS server name for raw TLS when it differs from the endpoint host. */
        public Builder tlsServerName(String value) {
            ensureMutable();
            tlsServerName = requireText(value, "TLS server name");
            return this;
        }

        /** Sets the HTTP request target URI required by an HTTPS endpoint. */
        public Builder targetUri(String value) {
            ensureMutable();
            targetUri = requireText(value, "HTTP target URI");
            return this;
        }

        /** Configures connect, write, read, and total request timeouts. */
        public Builder timeouts(Timeout connect, Timeout write, Timeout read, Timeout total) {
            ensureMutable();
            if (connect == null || write == null || read == null || total == null) {
                throw new InvalidInputException("transport timeouts are invalid");
            }
            connectTimeout = connect;
            writeTimeout = write;
            readTimeout = read;
            totalTimeout = total;
            return this;
        }

        /** Sets the maximum encoded KMIP request size. */
        public Builder maxRequestBytes(long value) {
            ensureMutable();
            if (value <= 0) {
                throw new InvalidInputException("maximum request size must be positive");
            }
            maxRequestBytes = value;
            return this;
        }

        /** Validates inputs and creates an opaque native configuration. */
        public TransportConfiguration build() {
            ensureMutable();
            built = true;
            try {
                validate();
                NativeExtensionRegistry.ensureLoaded();
                byte[][] clientData = data(clientCertificates);
                byte[][] trustData = data(trustCertificates);
                byte[][] revocationData = data(revocationLists);
                long nativeHandle = NativeExtensionRegistry.transportConfigCreate(
                        endpointKind, endpoint, endpointPort, trustKind,
                        sources(clientCertificates), encodings(clientCertificates), clientData,
                        source(privateKey), privateKey.encoding.nativeValue, privateKey.data,
                        sources(trustCertificates), encodings(trustCertificates), trustData,
                        sources(revocationLists), encodings(revocationLists), revocationData,
                        tlsServerName, targetUri, timeoutValues(), maxRequestBytes);
                return new TransportConfiguration(nativeHandle);
            } finally {
                if (privateKey != null && privateKey.secret) {
                    Arrays.fill(privateKey.data, (byte) 0);
                }
                wipeSecretInputReferences();
            }
        }

        private Builder clientIdentity(byte[][] chain, byte[] key, Encoding encoding) {
            ensureMutable();
            clientCertificates = memoryInputs(chain, encoding, false);
            if (key == null || encoding == null) {
                throw new InvalidInputException("client identity is incomplete");
            }
            if (privateKey != null && privateKey.secret) {
                Arrays.fill(privateKey.data, (byte) 0);
            }
            privateKey = new Input(SOURCE_MEMORY, encoding, key.clone(), true);
            return this;
        }

        private Builder trustCertificates(byte[][] certificates, Encoding encoding) {
            ensureMutable();
            trustKind = TRUST_CERTIFICATES;
            trustCertificates = memoryInputs(certificates, encoding, false);
            return this;
        }

        private Builder revocationLists(byte[][] lists, Encoding encoding) {
            ensureMutable();
            revocationLists = memoryInputs(lists, encoding, false);
            return this;
        }

        private void validate() {
            if (endpointKind == 0 || endpoint == null || endpoint.isBlank()) {
                throw new InvalidInputException("transport endpoint is missing");
            }
            if (clientCertificates.isEmpty() || privateKey == null) {
                throw new InvalidInputException("mutual TLS client identity is required");
            }
            if (trustKind == 0 || (trustKind == TRUST_CERTIFICATES && trustCertificates.isEmpty())) {
                throw new InvalidInputException("TLS trust source is required");
            }
            if (endpointKind == RAW_TLS && targetUri != null) {
                throw new InvalidInputException("HTTP target URI requires HTTPS");
            }
            if (endpointKind == HTTPS && tlsServerName != null) {
                throw new InvalidInputException("TLS server name override requires raw TLS");
            }
            validateText(endpoint, "transport endpoint");
            validateText(tlsServerName, "TLS server name");
            validateText(targetUri, "HTTP target URI");
            long total = totalBytes(clientCertificates, privateKey, trustCertificates, revocationLists);
            if (total > MAX_INPUT_BYTES) {
                throw new ResourceLimitException("transport material exceeds the local input limit");
            }
        }

        private long[] timeoutValues() {
            Timeout[] values = {connectTimeout, writeTimeout, readTimeout, totalTimeout};
            long[] encoded = new long[12];
            for (int index = 0; index < values.length; index++) {
                Timeout timeout = values[index];
                encoded[index * 3] = timeout.kind();
                encoded[index * 3 + 1] = timeout.seconds();
                encoded[index * 3 + 2] = timeout.nanoseconds();
            }
            return encoded;
        }

        private void wipeSecretInputReferences() {
            if (privateKey != null && privateKey.secret) {
                Arrays.fill(privateKey.data, (byte) 0);
            }
        }

        private void ensureMutable() {
            if (built) {
                throw new InvalidInputException("transport configuration builder is already consumed");
            }
        }
    }

    private static List<Input> memoryInputs(byte[][] values, Encoding encoding, boolean secret) {
        if (values == null || values.length == 0 || encoding == null) {
            throw new InvalidInputException("transport material is incomplete");
        }
        if (values.length > 4096) {
            throw new ResourceLimitException("transport material count exceeds the local limit");
        }
        List<Input> result = new ArrayList<>(values.length);
        for (byte[] value : values) {
            if (value == null || value.length == 0) {
                throw new InvalidInputException("transport material is empty");
            }
            result.add(new Input(SOURCE_MEMORY, encoding, value.clone(), secret));
        }
        return List.copyOf(result);
    }

    private static List<Input> fileInputs(List<Path> paths, Encoding encoding) {
        if (paths == null || paths.isEmpty() || paths.size() > 4096 || encoding == null) {
            throw new InvalidInputException("transport file inputs are invalid");
        }
        List<Input> result = new ArrayList<>(paths.size());
        for (Path path : paths) {
            result.add(fileInput(path, encoding, false));
        }
        return List.copyOf(result);
    }

    private static Input fileInput(Path path, Encoding encoding, boolean secret) {
        if (path == null || encoding == null || path.toString().isBlank()) {
            throw new InvalidInputException("transport file input is invalid");
        }
        byte[] bytes = path.toString().getBytes(StandardCharsets.UTF_8);
        if (bytes.length > 4096) {
            throw new ResourceLimitException("transport file path exceeds the local text limit");
        }
        return new Input(SOURCE_FILE, encoding, bytes, secret);
    }

    private static byte[][] data(List<Input> inputs) {
        return inputs.stream().map(input -> input.data).toArray(byte[][]::new);
    }

    private static int[] sources(List<Input> inputs) {
        return inputs.stream().mapToInt(input -> input.source).toArray();
    }

    private static int[] encodings(List<Input> inputs) {
        return inputs.stream().mapToInt(input -> input.encoding.nativeValue).toArray();
    }

    private static int source(Input input) {
        return input.source;
    }

    private static long totalBytes(List<Input> certificates, Input key, List<Input> trust,
            List<Input> revocation) {
        long total = key.data.length;
        for (List<Input> group : List.of(certificates, trust, revocation)) {
            for (Input input : group) {
                total = Math.addExact(total, input.data.length);
            }
        }
        return total;
    }

    private static String requireText(String value, String field) {
        if (value == null || value.isBlank()) {
            throw new InvalidInputException(field + " is invalid");
        }
        validateText(value, field);
        return value;
    }

    private static void validateText(String value, String field) {
        if (value != null && value.getBytes(StandardCharsets.UTF_8).length > 4096) {
            throw new ResourceLimitException(field + " exceeds the local text limit");
        }
    }

    private static final class Input {
        private final int source;
        private final Encoding encoding;
        private final byte[] data;
        private final boolean secret;

        private Input(int source, Encoding encoding, byte[] data, boolean secret) {
            this.source = source;
            this.encoding = encoding;
            this.data = data;
            this.secret = secret;
        }
    }
}
