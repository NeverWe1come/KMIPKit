package org.kmipkit.ttlv;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Owned, validated managed object with a lossless generic TTLV view. */
public final class ManagedObject extends ManagedTtlvValue {
    private final long objectType;

    private ManagedObject(long nativeHandle, long objectType) {
        super(nativeHandle);
        this.objectType = objectType;
    }

    /** Copies and validates a managed object without consuming its source Item. */
    public static ManagedObject fromItem(long objectType, TtlvItem item) {
        if (objectType < 0 || objectType > 0xffff_ffffL || item == null) {
            throw new InvalidInputException("managed object input is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(value -> new ManagedObject(
                NativeExtensionRegistry.managedObjectFromItem((int) objectType, value), objectType));
    }

    /** Returns the source-preserving KMIP Object Type raw value. */
    public long objectType() {
        return objectType;
    }

}
