package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Sealed value validated by one immutable client registry. */
public final class RegisteredExtensionValue {
    private final NativeHandle handle;
    private final ExtensionIdentity identity;
    private final ClientExtensionRegistry registryOwner;
    private final TtlvStructure payloadOwner;

    RegisteredExtensionValue(long nativeHandle, ExtensionIdentity identity,
            ClientExtensionRegistry registryOwner, TtlvStructure payloadOwner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.REGISTERED_VALUE);
        this.identity = identity;
        this.registryOwner = registryOwner;
        this.payloadOwner = payloadOwner;
    }

    long handle() {
        return handle.get();
    }

    long transferHandle() {
        return handle.transfer();
    }

    @Override
    public String toString() {
        return "RegisteredExtensionValue([REDACTED])";
    }
}
