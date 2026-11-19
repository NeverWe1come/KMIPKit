package org.kmipkit.extensions;

import java.util.Optional;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvItem;

/** Owned KMIP 2.1 Modify Attribute Request model. */
public final class ModifyAttributeRequest extends AttributeRequest {
    private ModifyAttributeRequest(long handle, UniqueIdentifier identifier) {
        super(handle, NativeExtensionRegistry.MODIFY_ATTRIBUTE_REQUEST,
                NativeExtensionRegistry.ATTRIBUTE_OPERATION_MODIFY, identifier, null);
    }

    /** Copies the optional Current Attribute and required New Attribute Items. */
    public static ModifyAttributeRequest create(
            UniqueIdentifier identifier, TtlvItem currentAttribute, TtlvItem newAttribute) {
        if (newAttribute == null) {
            throw new InvalidInputException("New Attribute is required");
        }
        return new ModifyAttributeRequest(AttributeRequest.createNative(
                NativeExtensionRegistry.ATTRIBUTE_OPERATION_MODIFY, identifier,
                currentAttribute, newAttribute, null, 0, null), identifier);
    }

    /** Returns a copied optional Current Attribute Item. */
    public Optional<TtlvItem> currentAttribute() {
        return Optional.ofNullable(item(NativeExtensionRegistry.ATTRIBUTE_FIELD_CURRENT));
    }

    /** Returns an independently owned copy of New Attribute. */
    public TtlvItem newAttribute() {
        return item(NativeExtensionRegistry.ATTRIBUTE_FIELD_NEW);
    }
}
