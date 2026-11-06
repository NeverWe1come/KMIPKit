package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvItem;

/** Owned KMIP 2.1 Add Attribute Request model. */
public final class AddAttributeRequest extends AttributeRequest {
    private AddAttributeRequest(long handle, UniqueIdentifier identifier) {
        super(handle, NativeExtensionRegistry.ADD_ATTRIBUTE_REQUEST,
                NativeExtensionRegistry.ATTRIBUTE_OPERATION_ADD, identifier, null);
    }

    /** Copies the optional identifier and required New Attribute Item. */
    public static AddAttributeRequest create(UniqueIdentifier identifier, TtlvItem newAttribute) {
        if (newAttribute == null) {
            throw new org.kmipkit.InvalidInputException("New Attribute is required");
        }
        return new AddAttributeRequest(AttributeRequest.createNative(
                NativeExtensionRegistry.ATTRIBUTE_OPERATION_ADD, identifier,
                null, newAttribute, null, 0, null), identifier);
    }

    /** Returns an independently owned copy of New Attribute. */
    public TtlvItem newAttribute() {
        return item(NativeExtensionRegistry.ATTRIBUTE_FIELD_NEW);
    }
}
