package org.kmipkit.server;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.ttlv.TtlvStructure;

/** Received Notify request. */
public final class NotifyRequest extends ServerRequest {
    NotifyRequest(long handle, ServerRequestBatch owner) { super(handle, owner, 27); }

    /** Returns deleted Attribute References in their wire order. */
    public List<AttributeReference> deletedAttributeReferences() {
        long count = ServerNative.notifyDeletedAttributeReferenceCount(nativeHandle());
        List<AttributeReference> references = new ArrayList<>((int) count);
        for (long index = 0; index < count; index++) {
            references.add(ServerNative.notifyDeletedAttributeReferenceAt(nativeHandle(), index));
        }
        return List.copyOf(references);
    }

    /** Returns an independent, validated copy of the optional changed Attributes. */
    public AttributeSet attributes() {
        long[] result = ServerNative.notifyAttributes(nativeHandle());
        if (result.length == 0) return null;
        try (TtlvStructure items = TtlvStructure.fromNative(result[0], this)) {
            return AttributeSet.tryNew(items);
        }
    }
}
