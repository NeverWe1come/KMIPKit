package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TypedAttribute;

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

    /** Appends one typed occurrence while preserving repeats and source order. */
    public void tryPushTyped(StandardAttribute attribute, TypedAttribute typed) {
        if (attribute == null || typed == null) {
            throw new InvalidInputException("typed Attribute inputs are required");
        }
        NativeExtensionRegistry.ensureLoaded();
        try (TtlvItem item = typed.asItem()) {
            NativeHandle.withValues(new NativeHandle[] {handle, item.nativeHandleOwner()}, values -> {
                NativeExtensionRegistry.attributeSetTryPushTyped(
                        values[0], attribute.tag(), values[1]);
                return null;
            });
        }
    }

    /** Returns the number of occurrences of one catalogued attribute. */
    public long typedCount(StandardAttribute attribute) {
        if (attribute == null) {
            throw new InvalidInputException("standard Attribute is required");
        }
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value ->
                NativeExtensionRegistry.attributeSetTypedCount(value, attribute.tag()));
    }

    /** Returns one caller-owned occurrence by its zero-based position. */
    public TypedAttribute typedAt(StandardAttribute attribute, long occurrence) {
        if (attribute == null || occurrence < 0) {
            throw new InvalidInputException("typed Attribute selection is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value ->
                NativeExtensionRegistry.attributeSetTypedAt(value, attribute, occurrence));
    }

    /** Returns all typed occurrences as caller-owned copies in wire order. */
    public List<TypedAttribute> typed(StandardAttribute attribute) {
        if (attribute == null) {
            throw new InvalidInputException("standard Attribute is required");
        }
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value -> {
            long count = NativeExtensionRegistry.attributeSetTypedCount(value, attribute.tag());
            if (count > Integer.MAX_VALUE) {
                throw new InvalidInputException("typed Attribute occurrence count is too large");
            }
            List<TypedAttribute> result = new ArrayList<>((int) count);
            try {
                for (long index = 0; index < count; index++) {
                    result.add(NativeExtensionRegistry.attributeSetTypedAt(value, attribute, index));
                }
                return List.copyOf(result);
            } catch (RuntimeException error) {
                result.forEach(TypedAttribute::close);
                throw error;
            }
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
