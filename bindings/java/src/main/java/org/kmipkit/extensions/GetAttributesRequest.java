package org.kmipkit.extensions;

import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Owned KMIP 2.1 Get Attributes request model. */
public final class GetAttributesRequest extends AttributeRequest {
    private final List<AttributeReference> references;

    private GetAttributesRequest(
            long handle, UniqueIdentifier identifier, List<AttributeReference> references) {
        super(handle, NativeExtensionRegistry.GET_ATTRIBUTES_REQUEST,
                OPERATION_GET_ATTRIBUTES, identifier, null);
        this.references = references;
    }

    /** Creates a request with ordered, unique Attribute References. */
    public static GetAttributesRequest create(
            UniqueIdentifier identifier, List<AttributeReference> references) {
        if (references == null) {
            throw new InvalidInputException("Attribute References are required");
        }
        List<AttributeReference> copied = List.copyOf(references);
        long handle = AttributeRequest.createReadNative(
                OPERATION_GET_ATTRIBUTES, identifier, copied);
        return new GetAttributesRequest(handle, identifier, copied);
    }

    /** Returns the ordered, unique Attribute References selected by this request. */
    public List<AttributeReference> attributeReferences() {
        ensureOpen();
        return references;
    }
}
