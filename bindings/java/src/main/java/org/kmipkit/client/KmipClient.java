package org.kmipkit.client;

import org.kmipkit.fluent.FluentRequest;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.RevocationReason;
import org.kmipkit.ttlv.CodecLimits;

/** Idiomatic task-oriented facade over the existing typed KMIP client. */
public final class KmipClient implements AutoCloseable {
    private final org.kmipkit.extensions.Client client;
    private KmipClient(org.kmipkit.extensions.Client client) { this.client = client; }
    public static KmipClient create(ClientConfiguration configuration, TransportConfiguration transport) {
        return new KmipClient(org.kmipkit.extensions.Client.create(configuration, transport));
    }
    public FluentRequest request() { return new FluentRequest(this); }
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
