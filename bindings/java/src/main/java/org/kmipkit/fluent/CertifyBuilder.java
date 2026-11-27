package org.kmipkit.fluent;

import java.util.Arrays;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP Certify request. */
public final class CertifyBuilder extends TypedOperationBuilder<CertifyBuilder> {
    private UniqueIdentifier uniqueIdentifier;
    private Long certificateRequestType;
    private byte[] certificateRequestValue;
    private TtlvStructure attributes;
    private TtlvStructure protectionStorageMasks;

    CertifyBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected CertifyBuilder self() { return this; }

    public CertifyBuilder uniqueIdentifier(UniqueIdentifier id) {
        ensureOpen();
        uniqueIdentifier = id;
        return this;
    }

    public CertifyBuilder certificateRequestType(long type) {
        ensureOpen();
        certificateRequestType = unsigned32(type, "Certificate Request Type");
        return this;
    }

    public CertifyBuilder certificateRequestValue(byte[] value) {
        ensureOpen();
        certificateRequestValue = value == null ? null : Arrays.copyOf(value, value.length);
        return this;
    }

    public CertifyBuilder attributes(TtlvStructure value) {
        attributes = replace(attributes, value);
        return this;
    }

    public CertifyBuilder protectionStorageMasks(TtlvStructure value) {
        protectionStorageMasks = replace(protectionStorageMasks, value);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.certify(
                uniqueIdentifier, certificateRequestType, certificateRequestValue, attributes, protectionStorageMasks));
    }
}
