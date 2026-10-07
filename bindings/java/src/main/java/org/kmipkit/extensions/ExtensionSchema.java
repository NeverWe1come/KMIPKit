package org.kmipkit.extensions;

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
        long[] childHandles = children.stream().mapToLong(ExtensionChildRule::handle).toArray();
        long[] orderHandles = orderConstraints.stream().mapToLong(ExtensionOrderConstraint::handle).toArray();
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionSchema(NativeExtensionRegistry.extensionSchemaStructure(
                childHandles, orderHandles, preserveUndeclaredChildren));
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
}
