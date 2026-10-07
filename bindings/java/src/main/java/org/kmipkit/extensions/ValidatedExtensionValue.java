package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValueView;

/** Schema-validated protocol value retaining its original generic TTLV tree. */
public final class ValidatedExtensionValue {
    private final NativeHandle handle;
    private final ExtensionIdentity identity;
    private final Object owner;

    ValidatedExtensionValue(long nativeHandle, ExtensionIdentity identity, Object owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.VALIDATED_VALUE);
        this.identity = identity;
        this.owner = owner;
    }

    public static ExtensionIdentity identity(ValidatedExtensionValue value) {
        NativeExtensionRegistry.ensureLoaded();
        long identityHandle = NativeExtensionRegistry.validatedExtensionValueIdentity(value.handle.get());
        return value.identity != null ? value.identity : ExtensionIdentity.fromNative(identityHandle);
    }

    public static TtlvStructureView genericValue(ValidatedExtensionValue value) {
        NativeExtensionRegistry.ensureLoaded();
        long viewHandle = NativeExtensionRegistry.validatedExtensionValueGenericValue(value.handle.get());
        return TtlvStructureView.fromNative(viewHandle, value);
    }

    public static TtlvValueView valueAt(ValidatedExtensionValue value, TtlvPath path) {
        NativeExtensionRegistry.ensureLoaded();
        long viewHandle = NativeExtensionRegistry.validatedExtensionValueValueAt(
                value.handle.get(), path.handle());
        return TtlvValueView.fromNative(viewHandle, value);
    }

    long handle() {
        return handle.get();
    }

    @Override
    public String toString() {
        return "ValidatedExtensionValue([REDACTED])";
    }
}
