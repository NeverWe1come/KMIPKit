package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvItem;

/** Owned KMIP 2.1 Set Attribute Request model. */
public final class SetAttributeRequest extends AttributeRequest {
    private SetAttributeRequest(long handle, UniqueIdentifier identifier) {
        super(handle, NativeExtensionRegistry.SET_ATTRIBUTE_REQUEST,
                NativeExtensionRegistry.ATTRIBUTE_OPERATION_SET, identifier, null);
    }

    /** Copies the optional identifier and required New Attribute Item. */
    public static SetAttributeRequest create(UniqueIdentifier identifier, TtlvItem newAttribute) {
        if (newAttribute == null) {
            throw new InvalidInputException("New Attribute is required");
        }
        return new SetAttributeRequest(AttributeRequest.createNative(
                NativeExtensionRegistry.ATTRIBUTE_OPERATION_SET, identifier,
                null, newAttribute, null, 0, null), identifier);
    }

    /** Returns an independently owned copy of New Attribute. */
    public TtlvItem newAttribute() {
        return item(NativeExtensionRegistry.ATTRIBUTE_FIELD_NEW);
    }
}
