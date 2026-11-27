package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP Import request. */
public final class ImportBuilder extends TypedOperationBuilder<ImportBuilder> {
    private UniqueIdentifier uniqueIdentifier;
    private Long objectType;
    private Boolean replaceExisting;
    private Long keyWrapType;
    private TtlvStructure attributes;
    private TtlvStructure anyObject;

    ImportBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected ImportBuilder self() { return this; }

    public ImportBuilder uniqueIdentifier(UniqueIdentifier id) {
        ensureOpen();
        uniqueIdentifier = id;
        return this;
    }

    public ImportBuilder objectType(long type) {
        ensureOpen();
        objectType = unsigned32(type, "Object Type");
        return this;
    }

    public ImportBuilder replaceExisting(boolean value) {
        ensureOpen();
        replaceExisting = value;
        return this;
    }

    public ImportBuilder keyWrapType(long value) {
        ensureOpen();
        keyWrapType = unsigned32(value, "Key Wrap Type");
        return this;
    }

    public ImportBuilder attributes(TtlvStructure value) {
        attributes = replace(attributes, value);
        return this;
    }

    public ImportBuilder object(TtlvStructure value) {
        anyObject = replace(anyObject, value);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (uniqueIdentifier == null || objectType == null || attributes == null || anyObject == null) {
            throw new InvalidInputException("Import requires uniqueIdentifier, objectType, attributes, and object");
        }
        return validated(ClientBatchItem.importObject(
                uniqueIdentifier, objectType, replaceExisting, keyWrapType, attributes, anyObject));
    }
}
