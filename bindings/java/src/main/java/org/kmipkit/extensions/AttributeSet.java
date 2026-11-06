package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;

/** Owned, validated ordered direct Items for KMIP Attribute collections. */
public final class AttributeSet implements AutoCloseable {
    private final NativeHandle handle;

    private AttributeSet(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.ATTRIBUTE_SET);
    }

    /** Copies and validates the direct Items in a generic Structure. */
    public static AttributeSet tryNew(TtlvStructure items) {
        if (items == null) {
            throw new InvalidInputException("Attribute Items are required");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> new AttributeSet(
                NativeExtensionRegistry.attributeSetTryNew(items.nativeHandleOwner().get())));
    }

    /** Copies and validates one Item while leaving its source handle open. */
    public void tryPush(TtlvItem item) {
        if (item == null) {
            throw new InvalidInputException("Attribute Item is required");
        }
        NativeExtensionRegistry.ensureLoaded();
        NativeHandle.withValues(new NativeHandle[] {handle, item.nativeHandleOwner()}, values -> {
            NativeExtensionRegistry.attributeSetTryPush(values[0], values[1]);
            return null;
        });
    }

    /** Returns a caller-owned generic Structure copy in original order. */
    public TtlvStructure asItems() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.attributeSetAsItems(value), this));
    }

    /** Returns the number of direct Items. */
    public long len() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(NativeExtensionRegistry::attributeSetLen);
    }

    /** Returns whether this collection is empty. */
    public boolean isEmpty() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(NativeExtensionRegistry::attributeSetIsEmpty);
    }

    /** Consumes this collection and returns its direct Items in original order. */
    public TtlvStructure intoItems() {
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> {
            TtlvStructure result = TtlvStructure.fromNative(
                    NativeExtensionRegistry.attributeSetIntoItems(handle.get()), this);
            handle.transfer();
            return result;
        });
    }

    @Override
    public void close() {
        handle.close();
    }
}
