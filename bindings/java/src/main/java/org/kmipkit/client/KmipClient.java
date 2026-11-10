package org.kmipkit.client;

import org.kmipkit.fluent.FluentRequest;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.TransportConfiguration;

/** Idiomatic task-oriented facade over the existing typed KMIP client. */
public final class KmipClient implements AutoCloseable {
    private final org.kmipkit.extensions.Client client;
    private KmipClient(org.kmipkit.extensions.Client client) { this.client = client; }
    public static KmipClient create(ClientConfiguration configuration, TransportConfiguration transport) {
        return new KmipClient(org.kmipkit.extensions.Client.create(configuration, transport));
    }
    public FluentRequest request() { return new FluentRequest(this); }
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
