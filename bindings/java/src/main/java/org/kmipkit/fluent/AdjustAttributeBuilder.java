package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvValue;

/** Fluent builder for a typed KMIP Adjust Attribute request (§6.1.3). */
public final class AdjustAttributeBuilder extends TypedOperationBuilder<AdjustAttributeBuilder> {
    private UniqueIdentifier uniqueIdentifier;
    private AttributeReference attributeReference;
    private Long adjustmentType;
    private TtlvValue adjustmentValue;

    AdjustAttributeBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected AdjustAttributeBuilder self() { return this; }

    public AdjustAttributeBuilder uniqueIdentifier(UniqueIdentifier id) {
        ensureOpen();
        uniqueIdentifier = id;
        return this;
    }

    public AdjustAttributeBuilder attributeReference(AttributeReference ref) {
        ensureOpen();
        attributeReference = ref;
        return this;
    }

    public AdjustAttributeBuilder adjustmentType(long value) {
        ensureOpen();
        adjustmentType = unsigned32(value, "Adjustment Type");
        return this;
    }

    public AdjustAttributeBuilder adjustmentValue(TtlvValue value) {
        ensureOpen();
        adjustmentValue = value;
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (attributeReference == null || adjustmentType == null) {
            throw new InvalidInputException(
                    "Adjust Attribute requires attributeReference and adjustmentType");
        }
        return validated(ClientBatchItem.adjustAttribute(
                uniqueIdentifier, attributeReference, adjustmentType, adjustmentValue));
    }
}
