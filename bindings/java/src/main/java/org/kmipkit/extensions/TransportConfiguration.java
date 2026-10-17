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
    private static final int MAX_MATERIAL_ITEMS = 4_096;
    private static final int MAX_TEXT_BYTES = 4_096;
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

    /** One certificate, key, trust certificate, or revocation list input.
     *
     * <p>Memory inputs own a defensive copy. Close them after configuring a builder to zeroize that
     * copy. Client identity configuration consumes and closes its private-key input immediately.
     * Java runtime copies outside KMIPKit's control cannot be zeroized by this type.
     */
    public static final class TransportInput implements AutoCloseable {
        private final int source;
        private final Encoding encoding;
        private final byte[] data;
        private boolean closed;

        private TransportInput(int source, Encoding encoding, byte[] data) {
            this.source = source;
            this.encoding = encoding;
            this.data = data;
        }

        /** Copies one in-memory credential or certificate value. */
        public static TransportInput memory(byte[] value, Encoding encoding) {
            if (value == null || value.length == 0 || encoding == null) {
                throw new InvalidInputException("transport material is incomplete");
            }
            if (value.length > MAX_INPUT_BYTES) {
                throw new ResourceLimitException("transport material exceeds the local input limit");
            }
            return new TransportInput(SOURCE_MEMORY, encoding, value.clone());
        }

        /** Copies a file path used by the native transport to load one input. */
        public static TransportInput file(Path path, Encoding encoding) {
            String text = filePathText(path, encoding);
            return new TransportInput(SOURCE_FILE, encoding, text.getBytes(StandardCharsets.UTF_8));
        }

        /** Returns the caller-selected encoding without exposing the material bytes or path. */
        public Encoding encoding() {
            return encoding;
        }

        /** Zeroizes this object's copied bytes and prevents later use. */
        @Override
        public synchronized void close() {
            Arrays.fill(data, (byte) 0);
            closed = true;
        }

        private synchronized Input snapshot(boolean secret) {
            ensureOpen();
            return new Input(source, encoding, data.clone(), secret);
        }

        private synchronized long byteLength() {
            ensureOpen();
            return data.length;
        }

        private synchronized void ensureOpen() {
            if (closed) {
                throw new InvalidInputException("transport input is already closed");
            }
        }

        @Override
        public String toString() {
            return "TransportInput[source=" + source + ", encoding=" + encoding
                    + ", data=[REDACTED], closed=" + closed + "]";
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
    public static final class Builder implements AutoCloseable {
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

        /** Zeroizes the copied private key and consumes this builder.
         *
         * <p>Close a builder that is discarded before {@link #build()}. Building also consumes the
         * builder and clears its private-key copy, including when validation or native creation fails.
         */
        @Override
        public void close() {
            if (!built) {
                built = true;
                wipeSecretInputReferences();
            }
        }

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
            List<String> certificatePaths = filePathTexts(certificateChain, encoding);
            String keyPath = filePathText(privateKeyFile, encoding);
            long replacementBytes = Math.addExact(utf8Bytes(certificatePaths), utf8Length(keyPath));
            ensureAggregateLimit(replacementBytes, clientIdentityBytes());

            List<Input> replacementCertificates = fileInputs(certificatePaths, encoding);
            Input replacementKey = fileInput(keyPath, encoding, true);
            replacePrivateKey(replacementKey);
            clientCertificates = replacementCertificates;
            return this;
        }

        /** Supplies each certificate and the private key with independent source and encoding selections. */
        public Builder clientIdentity(List<TransportInput> certificateChain, TransportInput privateKeyInput) {
            ensureMutable();
            List<TransportInput> checkedCertificates = checkedTransportInputs(certificateChain);
            if (privateKeyInput == null) {
                throw new InvalidInputException("client identity is incomplete");
            }
            long replacementBytes = Math.addExact(transportInputBytes(checkedCertificates),
                    privateKeyInput.byteLength());
            ensureAggregateLimit(replacementBytes, clientIdentityBytes());

            List<Input> replacementCertificates = snapshotTransportInputs(checkedCertificates, false);
            Input replacementKey = privateKeyInput.snapshot(true);
            replacePrivateKey(replacementKey);
            clientCertificates = replacementCertificates;
            privateKeyInput.close();
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
            List<String> certificatePaths = filePathTexts(certificates, encoding);
            ensureAggregateLimit(utf8Bytes(certificatePaths), inputBytes(trustCertificates));
            List<Input> replacement = fileInputs(certificatePaths, encoding);
            trustKind = TRUST_CERTIFICATES;
            trustCertificates = replacement;
            return this;
        }

        /** Supplies trust certificates with independent source and encoding selections. */
        public Builder trustCertificates(List<TransportInput> certificates) {
            ensureMutable();
            List<TransportInput> checkedCertificates = checkedTransportInputs(certificates);
            ensureAggregateLimit(transportInputBytes(checkedCertificates), inputBytes(trustCertificates));
            List<Input> replacement = snapshotTransportInputs(checkedCertificates, false);
            trustKind = TRUST_CERTIFICATES;
            trustCertificates = replacement;
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

        /** Supplies in-memory revocation lists with independent encoding selections. */
        public Builder revocationLists(List<TransportInput> lists) {
            ensureMutable();
            List<TransportInput> checkedLists = checkedTransportInputs(lists);
            for (TransportInput input : checkedLists) {
                if (input.source != SOURCE_MEMORY) {
                    throw new InvalidInputException("revocation-list files are not supported");
                }
            }
            ensureAggregateLimit(transportInputBytes(checkedLists), inputBytes(revocationLists));
            revocationLists = snapshotTransportInputs(checkedLists, false);
            return this;
        }

        /** Sets the TLS verification name for an IP endpoint; the HTTPS authority is unchanged. */
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
                wipeSecretInputReferences();
            }
        }

        private Builder clientIdentity(byte[][] chain, byte[] key, Encoding encoding) {
            ensureMutable();
            if (key == null || key.length == 0 || encoding == null) {
                throw new InvalidInputException("client identity is incomplete");
            }
            long replacementBytes = Math.addExact(memoryInputBytes(chain, encoding), key.length);
            ensureAggregateLimit(replacementBytes, clientIdentityBytes());

            List<Input> replacementCertificates = memoryInputs(chain, encoding, false);
            Input replacementKey = new Input(SOURCE_MEMORY, encoding, key.clone(), true);
            replacePrivateKey(replacementKey);
            clientCertificates = replacementCertificates;
            return this;
        }

        private Builder trustCertificates(byte[][] certificates, Encoding encoding) {
            ensureMutable();
            long replacementBytes = memoryInputBytes(certificates, encoding);
            ensureAggregateLimit(replacementBytes, inputBytes(trustCertificates));
            List<Input> replacement = memoryInputs(certificates, encoding, false);
            trustKind = TRUST_CERTIFICATES;
            trustCertificates = replacement;
            return this;
        }

        private Builder revocationLists(byte[][] lists, Encoding encoding) {
            ensureMutable();
            long replacementBytes = memoryInputBytes(lists, encoding);
            ensureAggregateLimit(replacementBytes, inputBytes(revocationLists));
            List<Input> replacement = memoryInputs(lists, encoding, false);
            revocationLists = replacement;
            return this;
        }

        private long clientIdentityBytes() {
            return Math.addExact(inputBytes(clientCertificates), inputBytes(privateKey));
        }

        private void ensureAggregateLimit(long replacementBytes, long replacedBytes) {
            long projectedBytes = Math.addExact(
                    Math.subtractExact(totalInputBytes(clientCertificates, privateKey, trustCertificates,
                            revocationLists), replacedBytes),
                    replacementBytes);
            if (projectedBytes > MAX_INPUT_BYTES) {
                throw new ResourceLimitException("transport material exceeds the local input limit");
            }
        }

        private void replacePrivateKey(Input replacement) {
            if (privateKey != null && privateKey.secret) {
                Arrays.fill(privateKey.data, (byte) 0);
            }
            privateKey = replacement;
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

            validateText(endpoint, "transport endpoint");
            validateText(tlsServerName, "TLS server name");
            validateText(targetUri, "HTTP target URI");
            long total = totalInputBytes(clientCertificates, privateKey, trustCertificates, revocationLists);
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
        if (values.length > MAX_MATERIAL_ITEMS) {
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

    private static List<TransportInput> checkedTransportInputs(List<TransportInput> values) {
        if (values == null || values.isEmpty()) {
            throw new InvalidInputException("transport material is incomplete");
        }
        if (values.size() > MAX_MATERIAL_ITEMS) {
            throw new ResourceLimitException("transport material count exceeds the local limit");
        }
        List<TransportInput> result = new ArrayList<>(values.size());
        for (TransportInput value : values) {
            if (value == null) {
                throw new InvalidInputException("transport material is incomplete");
            }
            value.byteLength();
            result.add(value);
        }
        return List.copyOf(result);
    }

    private static List<Input> snapshotTransportInputs(List<TransportInput> values, boolean secret) {
        List<Input> result = new ArrayList<>(values.size());
        for (TransportInput value : values) {
            result.add(value.snapshot(secret));
        }
        return List.copyOf(result);
    }

    private static long transportInputBytes(List<TransportInput> values) {
        long total = 0;
        for (TransportInput value : values) {
            total = Math.addExact(total, value.byteLength());
        }
        if (total > MAX_INPUT_BYTES) {
            throw new ResourceLimitException("transport material exceeds the local input limit");
        }
        return total;
    }

    private static long memoryInputBytes(byte[][] values, Encoding encoding) {
        if (values == null || values.length == 0 || encoding == null) {
            throw new InvalidInputException("transport material is incomplete");
        }
        if (values.length > MAX_MATERIAL_ITEMS) {
            throw new ResourceLimitException("transport material count exceeds the local limit");
        }
        long total = 0;
        for (byte[] value : values) {
            if (value == null || value.length == 0) {
                throw new InvalidInputException("transport material is empty");
            }
            total = Math.addExact(total, value.length);
        }
        if (total > MAX_INPUT_BYTES) {
            throw new ResourceLimitException("transport material exceeds the local input limit");
        }
        return total;
    }

    private static List<String> filePathTexts(List<Path> paths, Encoding encoding) {
        if (paths == null || paths.isEmpty() || paths.size() > MAX_MATERIAL_ITEMS || encoding == null) {
            throw new InvalidInputException("transport file inputs are invalid");
        }
        List<String> result = new ArrayList<>(paths.size());
        for (Path path : paths) {
            result.add(filePathText(path, encoding));
        }
        return List.copyOf(result);
    }

    private static String filePathText(Path path, Encoding encoding) {
        if (path == null || encoding == null) {
            throw new InvalidInputException("transport file input is invalid");
        }
        String text = path.toString();
        if (text.isBlank()) {
            throw new InvalidInputException("transport file input is invalid");
        }
        if (utf8Length(text) > MAX_TEXT_BYTES) {
            throw new ResourceLimitException("transport file path exceeds the local text limit");
        }
        return text;
    }

    private static List<Input> fileInputs(List<String> paths, Encoding encoding) {
        List<Input> result = new ArrayList<>(paths.size());
        for (String path : paths) {
            result.add(fileInput(path, encoding, false));
        }
        return List.copyOf(result);
    }

    private static Input fileInput(String path, Encoding encoding, boolean secret) {
        byte[] bytes = path.getBytes(StandardCharsets.UTF_8);
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

    private static long totalInputBytes(List<Input> certificates, Input key, List<Input> trust,
            List<Input> revocation) {
        long total = Math.addExact(inputBytes(certificates), inputBytes(key));
        total = Math.addExact(total, inputBytes(trust));
        total = Math.addExact(total, inputBytes(revocation));
        return total;
    }

    private static long inputBytes(List<Input> inputs) {
        long total = 0;
        for (Input input : inputs) {
            total = Math.addExact(total, input.data.length);
        }
        return total;
    }

    private static long inputBytes(Input input) {
        return input == null ? 0 : input.data.length;
    }

    private static long utf8Bytes(List<String> values) {
        long total = 0;
        for (String value : values) {
            total = Math.addExact(total, utf8Length(value));
        }
        return total;
    }

    private static long utf8Length(String value) {
        long length = 0;
        for (int index = 0; index < value.length(); index++) {
            char current = value.charAt(index);
            if (current <= 0x7f) {
                length++;
            } else if (current <= 0x7ff) {
                length += 2;
            } else if (Character.isHighSurrogate(current)
                    && index + 1 < value.length() && Character.isLowSurrogate(value.charAt(index + 1))) {
                length += 4;
                index++;
            } else if (Character.isSurrogate(current)) {
                length++;
            } else {
                length += 3;
            }
        }
        return length;
    }

    private static String requireText(String value, String field) {
        if (value == null || value.isBlank()) {
            throw new InvalidInputException(field + " is invalid");
        }
        validateText(value, field);
        return value;
    }

    private static void validateText(String value, String field) {
        if (value != null && utf8Length(value) > MAX_TEXT_BYTES) {
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
