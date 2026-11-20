package org.kmipkit.fluent;

import java.util.Arrays;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent typed Create Split Key request with ordered standard and vendor attributes. */
public final class CreateSplitKeyBuilder extends AttributeCollectionBuilder<CreateSplitKeyBuilder> {
    private Long objectType;
    private Long method;
    private Integer parts;
    private Integer threshold;
    private UniqueIdentifier uniqueIdentifier;
    private byte[] primeFieldSize;
    private TtlvStructure protectionStorageMasks;

    CreateSplitKeyBuilder(KmipClient client) {
        super(client);
    }
    @Override
    protected CreateSplitKeyBuilder self() {
        return this;
    }

    public CreateSplitKeyBuilder objectType(long value) {
        ensureOpen();
        objectType = unsigned32(value, "Object Type");
        return this;
    }
    public CreateSplitKeyBuilder method(long value) {
        ensureOpen();
        method = unsigned32(value, "Split Key Method");
        return this;
    }
    public CreateSplitKeyBuilder parts(int value) {
        ensureOpen();
        parts = value;
        return this;
    }
    public CreateSplitKeyBuilder threshold(int value) {
        ensureOpen();
        threshold = value;
        return this;
    }
    public CreateSplitKeyBuilder uniqueIdentifier(UniqueIdentifier value) {
        ensureOpen();
        uniqueIdentifier = value;
        return this;
    }
    public CreateSplitKeyBuilder primeFieldSize(byte[] value) {
        ensureOpen();
        if (primeFieldSize != null) Arrays.fill(primeFieldSize, (byte) 0);
        primeFieldSize = value == null ? null : value.clone();
        return this;
    }
    public CreateSplitKeyBuilder protectionStorageMasks(TtlvStructure value) {
        protectionStorageMasks = replace(protectionStorageMasks, value);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (objectType == null || method == null || parts == null || threshold == null) {
            throw new InvalidInputException("Object Type, Split Key Method, parts and threshold are required");
        }
        try (TtlvStructure attributes = buildAttributes()) {
            return validated(ClientBatchItem.createSplitKey(objectType, uniqueIdentifier, parts, threshold,
                    method, primeFieldSize, attributes, protectionStorageMasks));
        }
    }

    @Override
    public void close() {
        if (primeFieldSize != null) Arrays.fill(primeFieldSize, (byte) 0);
        super.close();
    }
}
