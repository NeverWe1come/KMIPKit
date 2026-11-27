package org.kmipkit.client;

import java.nio.file.Path;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.fluent.FluentBatchBuilder;
import org.kmipkit.fluent.FluentRequest;
import org.kmipkit.credentials.Authentication;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.fluent.KeyRef;
import org.kmipkit.fluent.KmipNamespaces;
import org.kmipkit.fluent.StreamsClient;
import org.kmipkit.fluent.WorkflowsClient;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ProtocolVersion;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.extensions.WireEncoding;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.RevocationReason;
import org.kmipkit.ttlv.CodecLimits;

/** Idiomatic task-oriented facade over the existing typed KMIP client. */
public final class KmipClient implements AutoCloseable {
    private final org.kmipkit.extensions.Client client;
    private final CodecLimits codecLimits;
    private final ProtocolVersion protocolVersion;

    private KmipClient(org.kmipkit.extensions.Client client,
            CodecLimits codecLimits, ProtocolVersion protocolVersion) {
        this.client = client;
        this.codecLimits = codecLimits;
        this.protocolVersion = protocolVersion;
    }

    public static KmipClient create(ClientConfiguration configuration, TransportConfiguration transport) {
        if (configuration == null || transport == null) {
            throw new InvalidInputException("client configuration is invalid");
        }
        return new KmipClient(
                org.kmipkit.extensions.Client.create(configuration, transport),
                configuration.codecLimits(),
                configuration.protocolVersion());
    }

    /** Starts a fluent builder for configuring and constructing a {@link KmipClient}. */
    public static Builder builder() {
        return new Builder();
    }

    /** Connects a raw-TLS mutual-TLS client from PEM file paths using KMIP 2.1 defaults. */
    public static KmipClient connectMtls(String host, int port,
            Path caPemFile, Path clientCertPemFile, Path clientKeyPemFile) {
        return builder()
                .rawTls(host, port)
                .caPemFile(caPemFile)
                .clientIdentityPemFiles(clientCertPemFile, clientKeyPemFile)
                .build();
    }

    /** Connects a raw-TLS mutual-TLS client from PEM file paths for a specific KMIP version. */
    public static KmipClient connectMtls(String host, int port,
            Path caPemFile, Path clientCertPemFile, Path clientKeyPemFile, ProtocolVersion version) {
        return builder()
                .rawTls(host, port)
                .caPemFile(caPemFile)
                .clientIdentityPemFiles(clientCertPemFile, clientKeyPemFile)
                .protocolVersion(version)
                .build();
    }

    /** Connects an HTTPS mutual-TLS client from PEM file paths using KMIP 2.1 defaults. */
    public static KmipClient connectHttps(String uri,
            Path caPemFile, Path clientCertPemFile, Path clientKeyPemFile) {
        return builder()
                .https(uri)
                .caPemFile(caPemFile)
                .clientIdentityPemFiles(clientCertPemFile, clientKeyPemFile)
                .build();
    }

    /** Returns the default {@link CodecLimits} embedded in this client's configuration. */
    public CodecLimits codecLimits() {
        return codecLimits;
    }

    /** Returns the default KMIP {@link ProtocolVersion} configured for this client. */
    public ProtocolVersion protocolVersion() {
        return protocolVersion;
    }

    /** Ergonomic builder combining transport and client configuration. */
    public static final class Builder {
        private final TransportConfiguration.Builder transport = TransportConfiguration.builder();
        private final List<Path> caPemFiles = new ArrayList<>();
        private final List<Path> caDerFiles = new ArrayList<>();
        private ClientConfiguration configuration;
        private TransportConfiguration transportConfig;
        private ProtocolVersion protocolVersion;
        private WireEncoding wireEncoding;
        private CodecLimits codecLimits;
        private TransportConfiguration.Timeout connectTimeout = TransportConfiguration.Timeout.defaultValue();
        private TransportConfiguration.Timeout writeTimeout = TransportConfiguration.Timeout.defaultValue();
        private TransportConfiguration.Timeout readTimeout = TransportConfiguration.Timeout.defaultValue();
        private TransportConfiguration.Timeout totalTimeout = TransportConfiguration.Timeout.defaultValue();

        private Builder() {}

        public Builder rawTls(String host, int port) { transport.rawTls(host, port); return this; }
        public Builder https(String uri) { transport.https(uri); return this; }
        public Builder targetUri(String target) { transport.targetUri(target); return this; }
        public Builder tlsServerName(String host) { transport.tlsServerName(host); return this; }
        public Builder tls12Compat(boolean enabled) { transport.tls12Compat(enabled); return this; }
        public Builder platformTrust() { transport.trustPlatform(); return this; }
        public Builder caPemFile(Path path) {
            if (path == null) throw new InvalidInputException("CA file path is invalid");
            caPemFiles.add(path);
            return this;
        }
        public Builder caDerFile(Path path) {
            if (path == null) throw new InvalidInputException("CA DER file path is invalid");
            caDerFiles.add(path);
            return this;
        }
        public Builder caPem(byte[][] certs) { transport.trustCertificatesPem(certs); return this; }
        public Builder caDer(byte[][] certs) { transport.trustCertificatesDer(certs); return this; }
        public Builder clientIdentityPemFiles(Path certPem, Path keyPem) {
            if (certPem == null || keyPem == null) {
                throw new InvalidInputException("client identity file paths are invalid");
            }
            transport.clientIdentityFiles(List.of(certPem), keyPem, TransportConfiguration.Encoding.PEM);
            return this;
        }
        public Builder clientIdentityPem(byte[][] certChain, byte[] keyPem) {
            transport.clientIdentityPem(certChain, keyPem);
            return this;
        }
        public Builder clientIdentityDer(byte[][] certChain, byte[] keyDer) {
            transport.clientIdentityDer(certChain, keyDer);
            return this;
        }
        public Builder connectTimeout(Duration d) {
            this.connectTimeout = TransportConfiguration.Timeout.bounded(d);
            return this;
        }
        public Builder writeTimeout(Duration d) {
            this.writeTimeout = TransportConfiguration.Timeout.bounded(d);
            return this;
        }
        public Builder readTimeout(Duration d) {
            this.readTimeout = TransportConfiguration.Timeout.bounded(d);
            return this;
        }
        public Builder totalTimeout(Duration d) {
            this.totalTimeout = TransportConfiguration.Timeout.bounded(d);
            return this;
        }
        public Builder protocolVersion(ProtocolVersion version) {
            if (version == null) throw new InvalidInputException("protocol version is invalid");
            this.protocolVersion = version;
            return this;
        }
        public Builder wireEncoding(WireEncoding encoding) {
            if (encoding == null) throw new InvalidInputException("wire encoding is invalid");
            this.wireEncoding = encoding;
            return this;
        }
        public Builder codecLimits(CodecLimits limits) {
            if (limits == null) throw new InvalidInputException("codec limits are invalid");
            this.codecLimits = limits;
            return this;
        }
        public Builder configuration(ClientConfiguration configuration) {
            if (configuration == null) throw new InvalidInputException("client configuration is invalid");
            this.configuration = configuration;
            return this;
        }
        public Builder transportConfiguration(TransportConfiguration transportConfiguration) {
            if (transportConfiguration == null) {
                throw new InvalidInputException("transport configuration is invalid");
            }
            this.transportConfig = transportConfiguration;
            return this;
        }

        public KmipClient build() {
            boolean ownsConfig = configuration == null;
            ClientConfiguration cfg = ownsConfig ? ClientConfiguration.defaults() : configuration;
            try {
                if (wireEncoding != null) {
                    ClientConfiguration next = cfg.withWireEncoding(wireEncoding);
                    if (ownsConfig) cfg.close();
                    cfg = next;
                    ownsConfig = true;
                }
                if (protocolVersion != null) {
                    ClientConfiguration next = cfg.withProtocolVersion(protocolVersion);
                    if (ownsConfig) cfg.close();
                    cfg = next;
                    ownsConfig = true;
                }
                if (codecLimits != null) {
                    ClientConfiguration next = cfg.withCodecLimits(codecLimits);
                    if (ownsConfig) cfg.close();
                    cfg = next;
                    ownsConfig = true;
                }
                if (!caPemFiles.isEmpty()) {
                    transport.trustCertificateFiles(caPemFiles, TransportConfiguration.Encoding.PEM);
                } else if (!caDerFiles.isEmpty()) {
                    transport.trustCertificateFiles(caDerFiles, TransportConfiguration.Encoding.DER);
                }
                if (transportConfig != null) {
                    return KmipClient.create(cfg, transportConfig);
                }
                transport.timeouts(connectTimeout, writeTimeout, readTimeout, totalTimeout);
                try (TransportConfiguration builtTransport = transport.build()) {
                    return KmipClient.create(cfg, builtTransport);
                }
            } finally {
                if (ownsConfig && cfg != null) {
                    cfg.close();
                }
            }
        }
    }

    public FluentRequest request() { return new FluentRequest(this); }
    public KmipNamespaces.KeysClient keys() { return new KmipNamespaces.KeysClient(this); }
    public KmipNamespaces.CryptoClient crypto() { return new KmipNamespaces.CryptoClient(this); }
    public KmipNamespaces.CertificatesClient certificates() { return new KmipNamespaces.CertificatesClient(this); }
    public KmipNamespaces.SecretsClient secrets() { return new KmipNamespaces.SecretsClient(this); }
    public KmipNamespaces.AttributesClient attributes() { return new KmipNamespaces.AttributesClient(this); }
    public KmipNamespaces.SessionsClient sessions() { return new KmipNamespaces.SessionsClient(this); }
    public KmipNamespaces.AdminClient admin() { return new KmipNamespaces.AdminClient(this); }
    public KmipNamespaces.AsyncOpsClient asyncOps() { return new KmipNamespaces.AsyncOpsClient(this); }
    public KeyRef key(String id) { return new KeyRef(this, UniqueIdentifier.textString(id)); }
    public KeyRef key(UniqueIdentifier id) { return new KeyRef(this, id); }
    public FluentBatchBuilder batch() { return new FluentBatchBuilder(this); }
    public WorkflowsClient workflows() { return new WorkflowsClient(this); }
    public StreamsClient streams() { return new StreamsClient(this); }
    /** Executes a typed Create Symmetric Key request with the client's inherited timeout policy. */
    public ClientBatchResponse createSymmetricKey(SymmetricKeyRequest request, CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        return createSymmetricKey(request, limits, null);
    }
    /** Executes a typed Create Symmetric Key request with exchange-local timeout overrides. */
    public ClientBatchResponse createSymmetricKey(
            SymmetricKeyRequest request, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        if (request == null || limits == null) {
            throw new org.kmipkit.InvalidInputException("symmetric key request is invalid");
        }
        org.kmipkit.fluent.SymmetricKeyBuilder builder = request().createSymmetricKey()
                .algorithm(request.algorithm()).length(request.length()).usageMask(request.usageMask());
        if (request.name() != null) builder.name(request.name());
        if (options != null) builder.withOptions(options);
        try (builder) { return builder.execute(limits); }
    }
    /** Executes a typed Get request for an explicit identifier. */
    public ClientBatchResponse getById(UniqueIdentifier id, CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        return getById(id, limits, null);
    }
    /** Executes a typed Get request with exchange-local timeout overrides. */
    public ClientBatchResponse getById(UniqueIdentifier id, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        org.kmipkit.fluent.OperationBuilder builder = request().get(id);
        if (options != null) builder.withOptions(options);
        return builder.execute(limits);
    }
    /** Executes a typed Destroy request for an explicit identifier. */
    public ClientBatchResponse destroyById(UniqueIdentifier id, CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        return destroyById(id, limits, null);
    }
    /** Executes a typed Destroy request with exchange-local timeout overrides. */
    public ClientBatchResponse destroyById(UniqueIdentifier id, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        org.kmipkit.fluent.OperationBuilder builder = request().destroy(id);
        if (options != null) builder.withOptions(options);
        return builder.execute(limits);
    }
    /** Executes one Activate request for an explicit identifier. */
    public ClientBatchResponse activateById(UniqueIdentifier id, CodecLimits limits)
            throws org.kmipkit.ClientOperationException { return activateById(id, limits, null); }
    /** Executes one Activate request with exchange-local timeout overrides. */
    public ClientBatchResponse activateById(
            UniqueIdentifier id, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        org.kmipkit.fluent.OperationBuilder builder = request().activate(id);
        if (options != null) builder.withOptions(options);
        return builder.execute(limits);
    }
    /** Executes one Archive request for an explicit identifier. */
    public ClientBatchResponse archiveById(UniqueIdentifier id, CodecLimits limits)
            throws org.kmipkit.ClientOperationException { return archiveById(id, limits, null); }
    /** Executes one Archive request with exchange-local timeout overrides. */
    public ClientBatchResponse archiveById(
            UniqueIdentifier id, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        org.kmipkit.fluent.OperationBuilder builder = request().archive(id);
        if (options != null) builder.withOptions(options);
        return builder.execute(limits);
    }
    /** Executes one Recover request for an explicit identifier. */
    public ClientBatchResponse recoverById(UniqueIdentifier id, CodecLimits limits)
            throws org.kmipkit.ClientOperationException { return recoverById(id, limits, null); }
    /** Executes one Recover request with exchange-local timeout overrides. */
    public ClientBatchResponse recoverById(
            UniqueIdentifier id, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        org.kmipkit.fluent.OperationBuilder builder = request().recover(id);
        if (options != null) builder.withOptions(options);
        return builder.execute(limits);
    }
    /** Executes one Locate request with an explicit Attribute collection. */
    public ClientBatchResponse locate(AttributeSet attributes, CodecLimits limits)
            throws org.kmipkit.ClientOperationException { return locate(attributes, limits, null); }
    /** Executes Locate with exchange-local timeout overrides. */
    public ClientBatchResponse locate(
            AttributeSet attributes, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        org.kmipkit.fluent.LocateBuilder builder = request().locate(attributes);
        if (options != null) builder.withOptions(options);
        return builder.execute(limits);
    }
    /** Executes one Revoke request with an explicit reason and optional date. */
    public ClientBatchResponse revokeById(
            UniqueIdentifier id, RevocationReason reason, Long compromiseOccurrenceDate,
            CodecLimits limits) throws org.kmipkit.ClientOperationException {
        return revokeById(id, reason, compromiseOccurrenceDate, limits, null);
    }
    /** Executes Revoke with exchange-local timeout overrides. */
    public ClientBatchResponse revokeById(
            UniqueIdentifier id, RevocationReason reason, Long compromiseOccurrenceDate,
            CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        org.kmipkit.fluent.RevokeBuilder builder = request().revoke(id, reason);
        if (compromiseOccurrenceDate != null) builder.compromiseOccurrenceDate(compromiseOccurrenceDate);
        if (options != null) builder.withOptions(options);
        return builder.execute(limits);
    }
    /** Executes a typed Discover Versions request offering the given protocol versions. */
    public ClientBatchResponse discoverVersions(
            java.util.List<org.kmipkit.extensions.ProtocolVersion> protocolVersions,
            CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        try (var builder = request().discoverVersions()) {
            if (protocolVersions != null && !protocolVersions.isEmpty()) {
                builder.protocolVersions(protocolVersions);
            }
            if (options != null) builder.withOptions(options);
            return builder.execute(limits);
        }
    }
    /** Executes a typed Query request for one or more KMIP Query Functions. */
    public ClientBatchResponse query(int[] queryFunctions, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        try (var builder = request().query()) {
            if (queryFunctions != null && queryFunctions.length > 0) builder.functions(queryFunctions);
            if (options != null) builder.withOptions(options);
            return builder.execute(limits);
        }
    }
    /**
     * Executes one Query with Authentication in the Request Header.
     * The batch consumes Authentication when attached. It applies only to this exchange.
     */
    public ClientBatchResponse query(
            int[] queryFunctions, Authentication authentication, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        try (var builder = request().query(); var batch = ClientBatch.create()) {
            if (queryFunctions != null && queryFunctions.length > 0) builder.functions(queryFunctions);
            try (var item = builder.build()) {
                batch.add(item);
            }
            batch.setAuthentication(authentication);
            return options == null ? client.execute(batch, limits) : client.executeWithOptions(batch, limits, options);
        }
    }

    /** Executes a typed Ping request. */
    public ClientBatchResponse ping(CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        var builder = request().ping();
        if (options != null) builder.withOptions(options);
        return builder.execute(limits);
    }
    /** Executes a typed RNG Retrieve request. */
    public ClientBatchResponse rngRetrieve(int dataLength, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        try (var builder = request().rngRetrieve().dataLength(dataLength)) {
            if (options != null) builder.withOptions(options);
            return builder.execute(limits);
        }
    }
    /** Executes a typed RNG Seed request. */
    public ClientBatchResponse rngSeed(byte[] data, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        try (var builder = request().rngSeed().data(data)) {
            if (options != null) builder.withOptions(options);
            return builder.execute(limits);
        }
    }
    /** Executes a typed Check request for an explicit Unique Identifier. */
    public ClientBatchResponse checkById(
            UniqueIdentifier id, Long usageLimitsCount, Integer cryptographicUsageMask, Long leaseTime,
            CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        try (var builder = request().check().uniqueIdentifier(id)) {
            if (usageLimitsCount != null) builder.usageLimitsCount(usageLimitsCount);
            if (cryptographicUsageMask != null) builder.cryptographicUsageMask(cryptographicUsageMask);
            if (leaseTime != null) builder.leaseTime(leaseTime);
            if (options != null) builder.withOptions(options);
            return builder.execute(limits);
        }
    }
    /** Executes a typed Export request for an explicit Unique Identifier. */
    public ClientBatchResponse exportById(
            UniqueIdentifier id, Long keyFormatType, Long keyWrapType, Long keyCompressionType,
            CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        try (var builder = request().export().uniqueIdentifier(id)) {
            if (keyFormatType != null) builder.keyFormatType(keyFormatType);
            if (keyWrapType != null) builder.keyWrapType(keyWrapType);
            if (keyCompressionType != null) builder.keyCompressionType(keyCompressionType);
            if (options != null) builder.withOptions(options);
            return builder.execute(limits);
        }
    }
    /** Executes a typed Obtain Lease request for an explicit Unique Identifier. */
    public ClientBatchResponse obtainLeaseById(
            UniqueIdentifier id, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        try (var builder = request().obtainLease().uniqueIdentifier(id)) {
            if (options != null) builder.withOptions(options);
            return builder.execute(limits);
        }
    }
    /** Executes a typed Get Attribute List request for an optional Unique Identifier. */
    public ClientBatchResponse getAttributeList(
            UniqueIdentifier id, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        var builder = request().getAttributeList(id);
        if (options != null) builder.withOptions(options);
        return builder.execute(limits);
    }
    /** Executes any single typed ClientBatchItem directly. */
    public ClientBatchResponse executeItem(
            org.kmipkit.extensions.ClientBatchItem item, CodecLimits limits, RequestOptions options)
            throws org.kmipkit.ClientOperationException {
        try (item; var batch = org.kmipkit.extensions.ClientBatch.create()) {
            batch.add(item);
            return options == null ? client.execute(batch, limits) : client.executeWithOptions(batch, limits, options);
        }
    }
    public org.kmipkit.extensions.ClientBatchResponse execute(
            org.kmipkit.extensions.ClientBatch batch, org.kmipkit.ttlv.CodecLimits limits)
            throws org.kmipkit.ClientOperationException { return client.execute(batch, limits); }
    public org.kmipkit.extensions.ClientBatchResponse executeWithOptions(
            org.kmipkit.extensions.ClientBatch batch, org.kmipkit.ttlv.CodecLimits limits,
            org.kmipkit.extensions.RequestOptions options) throws org.kmipkit.ClientOperationException {
        return client.executeWithOptions(batch, limits, options);
    }
    @Override public void close() { client.close(); }
}
