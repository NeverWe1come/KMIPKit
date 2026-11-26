package org.kmipkit.fluent;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.SplitKeyReference;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP Join Split Key request. */
public final class JoinSplitKeyBuilder extends TypedOperationBuilder<JoinSplitKeyBuilder> {
    private Long objectType;
    private final List<SplitKeyReference> references = new ArrayList<>();
    private Long secretDataType;
    private TtlvStructure attributes;
    private TtlvStructure protectionStorageMasks;

    JoinSplitKeyBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected JoinSplitKeyBuilder self() { return this; }

    public JoinSplitKeyBuilder objectType(long value) {
        ensureOpen();
        objectType = unsigned32(value, "Object Type");
        return this;
    }

    public JoinSplitKeyBuilder reference(SplitKeyReference ref) {
        ensureOpen();
        if (ref == null) throw new InvalidInputException("split key reference is required");
        references.add(ref);
        return this;
    }

    public JoinSplitKeyBuilder references(List<SplitKeyReference> refs) {
        ensureOpen();
        if (refs == null) throw new InvalidInputException("split key references are required");
        references.addAll(refs);
        return this;
    }

    public JoinSplitKeyBuilder secretDataType(long value) {
        ensureOpen();
        secretDataType = unsigned32(value, "Secret Data Type");
        return this;
    }

    public JoinSplitKeyBuilder attributes(TtlvStructure value) {
        attributes = replace(attributes, value);
        return this;
    }

    public JoinSplitKeyBuilder protectionStorageMasks(TtlvStructure value) {
        protectionStorageMasks = replace(protectionStorageMasks, value);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (objectType == null || references.isEmpty()) {
            throw new InvalidInputException("Join Split Key requires objectType and references");
        }
        return validated(ClientBatchItem.joinSplitKey(
                objectType, references, secretDataType, attributes, protectionStorageMasks));
    }
}
