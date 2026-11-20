package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent typed Recertify request. Structure setters retain independent copies. */
public final class RecertifyBuilder extends TypedOperationBuilder<RecertifyBuilder> {
    private UniqueIdentifier uniqueIdentifier;
    private Long offset;
    private TtlvStructure attributes;
    private TtlvStructure protectionStorageMasks;
    private UniqueIdentifier certificateRequestUniqueIdentifier;
    private Long certificateRequestType;
    private byte[] certificateRequestValue;

    RecertifyBuilder(KmipClient client) {
        super(client);
    }
    @Override
    protected RecertifyBuilder self() {
        return this;
    }

    public RecertifyBuilder uniqueIdentifier(UniqueIdentifier value) {
        ensureOpen();
        uniqueIdentifier = value;
        return this;
    }
    public RecertifyBuilder offset(long value) {
        ensureOpen();
        offset = unsigned32(value, "Offset");
        return this;
    }
    public RecertifyBuilder attributes(TtlvStructure value) {
        attributes = replace(attributes, value);
        return this;
    }
    public RecertifyBuilder protectionStorageMasks(TtlvStructure value) {
        protectionStorageMasks = replace(protectionStorageMasks, value);
        return this;
    }
    public RecertifyBuilder certificateRequestUniqueIdentifier(UniqueIdentifier value) {
        ensureOpen();
        certificateRequestUniqueIdentifier = value;
        return this;
    }
    public RecertifyBuilder certificateRequestType(long value) {
        ensureOpen();
        certificateRequestType = unsigned32(value, "Certificate Request Type");
        return this;
    }
    public RecertifyBuilder certificateRequestValue(byte[] value) {
        ensureOpen();
        if (certificateRequestValue != null) java.util.Arrays.fill(certificateRequestValue, (byte) 0);
        certificateRequestValue = value == null ? null : value.clone();
        return this;
    }
    @Override
    public void close() {
        if (certificateRequestValue != null) java.util.Arrays.fill(certificateRequestValue, (byte) 0);
        super.close();
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.recertify(uniqueIdentifier, certificateRequestUniqueIdentifier, certificateRequestType, certificateRequestValue, offset, attributes, protectionStorageMasks));
    }
}
