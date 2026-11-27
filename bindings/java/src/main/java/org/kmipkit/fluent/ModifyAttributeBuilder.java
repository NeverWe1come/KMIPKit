package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvItem;

/** Fluent builder for a typed KMIP Modify Attribute request. */
public final class ModifyAttributeBuilder extends TypedOperationBuilder<ModifyAttributeBuilder> {
    private UniqueIdentifier uniqueIdentifier;
    private TtlvItem currentAttribute;
    private TtlvItem newAttribute;

    ModifyAttributeBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected ModifyAttributeBuilder self() { return this; }

    public ModifyAttributeBuilder uniqueIdentifier(UniqueIdentifier id) {
        ensureOpen();
        uniqueIdentifier = id;
        return this;
    }

    public ModifyAttributeBuilder currentAttribute(TtlvItem item) {
        ensureOpen();
        currentAttribute = item;
        return this;
    }

    public ModifyAttributeBuilder newAttribute(TtlvItem item) {
        ensureOpen();
        newAttribute = item;
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (newAttribute == null) throw new InvalidInputException("Modify Attribute newAttribute is required");
        return validated(ClientBatchItem.modifyAttribute(uniqueIdentifier, currentAttribute, newAttribute));
    }
}
