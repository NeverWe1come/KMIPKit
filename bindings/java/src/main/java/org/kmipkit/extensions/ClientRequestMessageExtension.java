package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** A registry-validated value and caller-selected KMIP criticality flag. */
public final class ClientRequestMessageExtension {
    private final NativeHandle handle;
    private final RegisteredExtensionValue owner;
    private final boolean criticalityIndicator;

    private ClientRequestMessageExtension(long nativeHandle, RegisteredExtensionValue owner,
            boolean criticalityIndicator) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.REQUEST_EXTENSION);
        this.owner = owner;
        this.criticalityIndicator = criticalityIndicator;
    }

    public static ClientRequestMessageExtension create(
            RegisteredExtensionValue value, boolean criticalityIndicator) {
        NativeExtensionRegistry.ensureLoaded();
        long valueHandle = value.transferHandle();
        long nativeHandle = NativeExtensionRegistry.clientRequestMessageExtensionCreate(
                valueHandle, criticalityIndicator);
        return new ClientRequestMessageExtension(nativeHandle, value, criticalityIndicator);
    }

    long handle() {
        return handle.get();
    }

    long nativeHandle() {
        return handle.get();
    }

    NativeHandle handleOwner() {
        return handle;
    }

    boolean criticalityIndicator() {
        return criticalityIndicator;
    }
}
