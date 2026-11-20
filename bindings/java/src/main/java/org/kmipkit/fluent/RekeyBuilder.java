package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent typed Rekey request. Structure setters retain independent copies. */
public final class RekeyBuilder extends TypedOperationBuilder<RekeyBuilder> {
    private UniqueIdentifier uniqueIdentifier;
    private Long offset;
    private TtlvStructure attributes;
    private TtlvStructure protectionStorageMasks;

    RekeyBuilder(KmipClient client) {
        super(client);
    }
    @Override
    protected RekeyBuilder self() {
        return this;
    }

    public RekeyBuilder uniqueIdentifier(UniqueIdentifier value) {
        ensureOpen();
        uniqueIdentifier = value;
        return this;
    }
    public RekeyBuilder offset(long value) {
        ensureOpen();
        offset = unsigned32(value, "Offset");
        return this;
    }
    public RekeyBuilder attributes(TtlvStructure value) {
        attributes = replace(attributes, value);
        return this;
    }
    public RekeyBuilder protectionStorageMasks(TtlvStructure value) {
        protectionStorageMasks = replace(protectionStorageMasks, value);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.rekey(uniqueIdentifier, offset, attributes, protectionStorageMasks));
    }
}
