package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP Re-Provision request (§6.1.48). */
public final class ReProvisionBuilder extends TypedOperationBuilder<ReProvisionBuilder> {
    private TtlvStructure certificateRequest;
    private TtlvStructure certificate;

    ReProvisionBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected ReProvisionBuilder self() { return this; }

    public ReProvisionBuilder certificateRequest(TtlvStructure value) {
        certificateRequest = replace(certificateRequest, value);
        return this;
    }

    public ReProvisionBuilder certificate(TtlvStructure value) {
        certificate = replace(certificate, value);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.reprovision(certificateRequest, certificate));
    }
}
