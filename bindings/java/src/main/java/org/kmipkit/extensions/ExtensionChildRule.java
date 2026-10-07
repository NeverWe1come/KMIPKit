package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.Tag;

/** Required, optional, or repeated child constraint in a Structure schema. */
public final class ExtensionChildRule {
    private final NativeHandle handle;

    private ExtensionChildRule(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CHILD_RULE);
    }

    public static ExtensionChildRule required(Tag tag, ExtensionSchema schema) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionChildRule(NativeExtensionRegistry.extensionChildRuleRequired(tag.handle(), schema.handle()));
    }

    public static ExtensionChildRule optional(Tag tag, ExtensionSchema schema) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionChildRule(NativeExtensionRegistry.extensionChildRuleOptional(tag.handle(), schema.handle()));
    }

    public static ExtensionChildRule repeated(Tag tag, ExtensionSchema schema) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionChildRule(NativeExtensionRegistry.extensionChildRuleRepeated(tag.handle(), schema.handle()));
    }

    long handle() {
        return handle.get();
    }
}
