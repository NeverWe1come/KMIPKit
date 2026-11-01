package org.kmipkit.extensions;

import java.util.List;

import org.kmipkit.NativeExtensionRegistry;

/** Owned KMIP 2.1 Get Attribute List request model. */
public final class GetAttributeListRequest extends AttributeRequest {
    private GetAttributeListRequest(long handle, UniqueIdentifier identifier) {
        super(handle, NativeExtensionRegistry.GET_ATTRIBUTE_LIST_REQUEST,
                OPERATION_GET_ATTRIBUTE_LIST, identifier, null);
    }

    /** Creates a request with the optional Unique Identifier from Table 226. */
    public static GetAttributeListRequest create(UniqueIdentifier identifier) {
        long handle = AttributeRequest.createReadNative(
                OPERATION_GET_ATTRIBUTE_LIST, identifier, List.of());
        return new GetAttributeListRequest(handle, identifier);
    }
}
