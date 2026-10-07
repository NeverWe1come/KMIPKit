package org.kmipkit.extensions;

import java.util.Arrays;
import java.util.List;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvItemType;

/** Data-only schema used to validate a vendor extension TTLV subtree. */
public final class ExtensionSchema {
    private final NativeHandle handle;

    private ExtensionSchema(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SCHEMA);
    }

    public static ExtensionSchema scalar(TtlvItemType itemType) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionSchema(NativeExtensionRegistry.extensionSchemaScalar(itemType.code()));
    }

    public static ExtensionSchema structure(List<ExtensionChildRule> children,
            List<ExtensionOrderConstraint> orderConstraints, boolean preserveUndeclaredChildren) {
        NativeHandle[] owners = new NativeHandle[children.size() + orderConstraints.size()];
        for (int index = 0; index < children.size(); index++) {
            owners[index] = children.get(index).nativeHandleOwner();
        }
        for (int index = 0; index < orderConstraints.size(); index++) {
            owners[children.size() + index] = orderConstraints.get(index).nativeHandleOwner();
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withValues(owners, handles -> {
            int childCount = children.size();
            long[] childHandles = Arrays.copyOfRange(handles, 0, childCount);
            long[] orderHandles = Arrays.copyOfRange(handles, childCount, handles.length);
            return new ExtensionSchema(NativeExtensionRegistry.extensionSchemaStructure(
                    childHandles, orderHandles, preserveUndeclaredChildren));
        });
    }

    public static ExtensionSchema with_minimum_length(ExtensionSchema schema, long value) {
        NativeExtensionRegistry.ensureLoaded();
        long input = schema.handle.transfer();
        return new ExtensionSchema(NativeExtensionRegistry.extensionSchemaMinimumLength(input, value));
    }

    public static ExtensionSchema with_maximum_length(ExtensionSchema schema, long value) {
        NativeExtensionRegistry.ensureLoaded();
        long input = schema.handle.transfer();
        return new ExtensionSchema(NativeExtensionRegistry.extensionSchemaMaximumLength(input, value));
    }

    public static ExtensionSchema with_signed_range(ExtensionSchema schema, long minimum, long maximum) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionSchema(NativeExtensionRegistry.extensionSchemaSignedRange(
                schema.handle.transfer(), minimum, maximum));
    }

    public static ExtensionSchema with_unsigned_range(ExtensionSchema schema, long minimum, long maximum) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionSchema(NativeExtensionRegistry.extensionSchemaUnsignedRange(
                schema.handle.transfer(), minimum, maximum));
    }

    public static ExtensionSchema with_allowed_enumeration(ExtensionSchema schema, int value) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionSchema(NativeExtensionRegistry.extensionSchemaAllowedEnumeration(
                schema.handle.transfer(), value));
    }

    public static ExtensionSchema with_allowed_bit_mask(ExtensionSchema schema, int value) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionSchema(NativeExtensionRegistry.extensionSchemaAllowedBitMask(
                schema.handle.transfer(), value));
    }

    public static ExtensionSchema with_required_bit_mask(ExtensionSchema schema, int value) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionSchema(NativeExtensionRegistry.extensionSchemaRequiredBitMask(
                schema.handle.transfer(), value));
    }

    long handle() {
        return handle.get();
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }
}
