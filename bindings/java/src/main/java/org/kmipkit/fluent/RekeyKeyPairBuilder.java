package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent typed Re-key Key Pair request retaining all attribute and protection-mask fields. */
public final class RekeyKeyPairBuilder extends TypedOperationBuilder<RekeyKeyPairBuilder> {
    private UniqueIdentifier privateKeyUniqueIdentifier;
    private Long offset;
    private TtlvStructure commonAttributes;
    private TtlvStructure privateKeyAttributes;
    private TtlvStructure publicKeyAttributes;
    private TtlvStructure commonProtectionStorageMasks;
    private TtlvStructure privateProtectionStorageMasks;
    private TtlvStructure publicProtectionStorageMasks;

    RekeyKeyPairBuilder(KmipClient client) {
        super(client);
    }
    @Override
    protected RekeyKeyPairBuilder self() {
        return this;
    }
    public RekeyKeyPairBuilder privateKeyUniqueIdentifier(UniqueIdentifier value) {
        ensureOpen();
        privateKeyUniqueIdentifier = value;
        return this;
    }
    public RekeyKeyPairBuilder offset(long value) {
        ensureOpen();
        offset = unsigned32(value, "Offset");
        return this;
    }
    public RekeyKeyPairBuilder commonAttributes(TtlvStructure value) {
        commonAttributes = replace(commonAttributes, value);
        return this;
    }
    public RekeyKeyPairBuilder commonAttributes(AttributeSet value) {
        try (TtlvStructure items = value.asItems()) {
            return commonAttributes(items);
        }
    }
    public RekeyKeyPairBuilder privateKeyAttributes(TtlvStructure value) {
        privateKeyAttributes = replace(privateKeyAttributes, value);
        return this;
    }
    public RekeyKeyPairBuilder privateKeyAttributes(AttributeSet value) {
        try (TtlvStructure items = value.asItems()) {
            return privateKeyAttributes(items);
        }
    }
    public RekeyKeyPairBuilder publicKeyAttributes(TtlvStructure value) {
        publicKeyAttributes = replace(publicKeyAttributes, value);
        return this;
    }
    public RekeyKeyPairBuilder publicKeyAttributes(AttributeSet value) {
        try (TtlvStructure items = value.asItems()) {
            return publicKeyAttributes(items);
        }
    }
    public RekeyKeyPairBuilder commonProtectionStorageMasks(TtlvStructure value) {
        commonProtectionStorageMasks = replace(commonProtectionStorageMasks, value);
        return this;
    }
    public RekeyKeyPairBuilder privateProtectionStorageMasks(TtlvStructure value) {
        privateProtectionStorageMasks = replace(privateProtectionStorageMasks, value);
        return this;
    }
    public RekeyKeyPairBuilder publicProtectionStorageMasks(TtlvStructure value) {
        publicProtectionStorageMasks = replace(publicProtectionStorageMasks, value);
        return this;
    }
    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.rekeyKeyPair(privateKeyUniqueIdentifier, offset,
                commonAttributes, privateKeyAttributes, publicKeyAttributes,
                commonProtectionStorageMasks, privateProtectionStorageMasks, publicProtectionStorageMasks));
    }
}
