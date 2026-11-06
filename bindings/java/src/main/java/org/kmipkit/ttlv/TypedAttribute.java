package org.kmipkit.ttlv;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Owned direct attribute checked against its assigned tag and KMIP TTLV type. */
public final class TypedAttribute implements AutoCloseable {
    private final NativeHandle handle;
    private final StandardAttribute attribute;

    private TypedAttribute(long nativeHandle, StandardAttribute attribute) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_ITEM);
        this.attribute = attribute;
    }

    /** Copies and validates one generic Item as the selected standard attribute. */
    public static TypedAttribute fromItem(StandardAttribute attribute, TtlvItem item) {
        if (attribute == null || item == null) {
            throw new InvalidInputException("typed attribute inputs are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withValues(new NativeHandle[] {item.nativeHandleOwner()}, values ->
                new TypedAttribute(
                        NativeExtensionRegistry.typedAttributeFromItem(attribute.tag(), values[0]),
                        attribute));
    }

    /** Returns the catalogued standard attribute definition. */
    public StandardAttribute attribute() {
        handle.get();
        return attribute;
    }

    /** Returns an independently owned generic Item copy. */
    public TtlvItem asItem() {
        return handle.withValue(value -> TtlvItem.fromNative(
                NativeExtensionRegistry.typedAttributeFromItem(attribute.tag(), value)));
    }

    /** Creates a read-only TTLV view that retains this typed owner. */
    public TtlvItemView view() {
        return handle.withValue(value -> TtlvItemView.fromNative(
                NativeTtlvCodec.itemViewFromItem(value), this));
    }

    @Override
    public void close() {
        handle.close();
    }
}
