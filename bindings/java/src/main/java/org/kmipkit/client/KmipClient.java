package org.kmipkit.client;

import org.kmipkit.fluent.FluentRequest;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.ClientBatchResponse;
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
