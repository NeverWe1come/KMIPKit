package org.kmipkit.fluent;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP Derive Key request. */
public final class DeriveKeyBuilder extends TypedOperationBuilder<DeriveKeyBuilder> {
    private Long objectType;
    private final List<UniqueIdentifier> uniqueIdentifiers = new ArrayList<>();
    private Long derivationMethod;
    private TtlvStructure derivationParameters;
    private TtlvStructure attributes;

    DeriveKeyBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected DeriveKeyBuilder self() { return this; }

    public DeriveKeyBuilder objectType(long value) {
        ensureOpen();
        objectType = unsigned32(value, "Object Type");
        return this;
    }

    public DeriveKeyBuilder uniqueIdentifier(UniqueIdentifier id) {
        ensureOpen();
        if (id == null) throw new InvalidInputException("Derive Key identifier is required");
        uniqueIdentifiers.add(id);
        return this;
    }

    public DeriveKeyBuilder uniqueIdentifiers(List<UniqueIdentifier> ids) {
        ensureOpen();
        if (ids == null) throw new InvalidInputException("Derive Key identifiers are required");
        uniqueIdentifiers.addAll(ids);
        return this;
    }

    public DeriveKeyBuilder derivationMethod(long value) {
        ensureOpen();
        derivationMethod = unsigned32(value, "Derivation Method");
        return this;
    }

    public DeriveKeyBuilder derivationParameters(TtlvStructure value) {
        derivationParameters = replace(derivationParameters, value);
        return this;
    }

    public DeriveKeyBuilder attributes(TtlvStructure value) {
        attributes = replace(attributes, value);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (objectType == null || derivationMethod == null || derivationParameters == null || attributes == null) {
            throw new InvalidInputException("Derive Key requires objectType, derivationMethod, derivationParameters, and attributes");
        }
        return validated(ClientBatchItem.deriveKey(
                objectType, uniqueIdentifiers, derivationMethod, derivationParameters, attributes));
    }
}
