package org.kmipkit.extensions;

import java.util.Optional;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvItem;

/** Owned KMIP 2.1 Delete Attribute Request model. */
public final class DeleteAttributeRequest extends AttributeRequest {
    private DeleteAttributeRequest(
            long handle, UniqueIdentifier identifier, AttributeReference reference) {
        super(handle, NativeExtensionRegistry.DELETE_ATTRIBUTE_REQUEST,
                NativeExtensionRegistry.ATTRIBUTE_OPERATION_DELETE, identifier, reference);
    }

    /** Copies the independently optional Current Attribute and Attribute Reference. */
    public static DeleteAttributeRequest create(
            UniqueIdentifier identifier, TtlvItem currentAttribute, AttributeReference reference) {
        return new DeleteAttributeRequest(AttributeRequest.createNative(
                NativeExtensionRegistry.ATTRIBUTE_OPERATION_DELETE, identifier,
                currentAttribute, null, reference, 0, null), identifier, reference);
    }

    /** Returns the optional Attribute Reference selector. */
    public Optional<AttributeReference> attributeReference() {
        return Optional.ofNullable(attributeReferenceValue());
    }

    /** Returns a copied optional Current Attribute Item. */
    public Optional<TtlvItem> currentAttribute() {
        return Optional.ofNullable(item(NativeExtensionRegistry.ATTRIBUTE_FIELD_CURRENT));
    }
}
