package org.kmipkit.credentials;

import java.util.function.Function;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructureView;

/** Read-only owner-backed view of one Credential in an Authentication. */
public final class CredentialView implements AutoCloseable {
    private final NativeHandle handle;
    @SuppressWarnings("unused")
    private final Authentication owner;

    CredentialView(long nativeHandle, Authentication owner) {
        this.handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CREDENTIAL_VIEW);
        this.owner = owner;
    }

    /** Return the raw-preserving typed Credential Type. */
    public CredentialType credentialType() {
        return handle.withValue(value -> CredentialType.fromRaw(
                NativeExtensionRegistry.credentialViewType(value)));
    }

    /** Return the original unsigned 32-bit Credential Type value. */
    public long credentialTypeRaw() {
        return handle.withValue(NativeExtensionRegistry::credentialViewTypeRaw);
    }

    /** Lends this Credential Structure to a callback and closes the view on return. */
    public <T> T withTtlv(Function<TtlvStructureView, T> callback) {
        if (callback == null) {
            throw new org.kmipkit.InvalidInputException("credential view callback is invalid");
        }
        return handle.withValue(value -> {
            TtlvStructureView view = TtlvStructureView.fromNative(
                    NativeExtensionRegistry.credentialViewTtlvView(value), this);
            try (view) {
                return callback.apply(view);
            }
        });
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "CredentialView[redacted]";
    }
}
