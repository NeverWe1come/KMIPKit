package org.kmipkit.credentials;

import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Function;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/** A generic KMIP Credential envelope preserving its original TTLV structure. */
public final class Credential implements AutoCloseable {
    private final NativeHandle handle;
    private final AtomicInteger activeViews = new AtomicInteger();

    private Credential(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CREDENTIAL);
    }

    /** Consumes and validates the outer Credential Type and Credential Value fields. */
    public static Credential fromTtlv(TtlvStructure structure) {
        if (structure == null) {
            throw new InvalidInputException("credential structure is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> {
            NativeHandle owner = structure.nativeHandleOwner();
            long[] outCredential = new long[1];
            int status = NativeExtensionRegistry.credentialFromTtlv(owner.get(), outCredential);
            if (status != CredentialInterop.SHARED_OWNER_STATUS) {
                owner.transfer();
            }
            CredentialInterop.throwIfFailed(status, "credential");
            return new Credential(outCredential[0]);
        });
    }

    /** Return the Credential Type while preserving unknown raw values. */
    public CredentialType credentialType() {
        long raw = handle.withValue(NativeExtensionRegistry::credentialType);
        return CredentialType.fromRaw(raw);
    }

    /** Return the original unsigned 32-bit Credential Type value. */
    public long credentialTypeRaw() {
        return handle.withValue(NativeExtensionRegistry::credentialTypeRaw);
    }

    /**
     * Lends a read-only view of the original ordered TTLV structure and closes it when the
     * callback returns. The Credential cannot be consumed while the callback is active.
     */
    public <T> T withTtlv(Function<TtlvStructureView, T> callback) {
        if (callback == null) {
            throw new InvalidInputException("credential view callback is invalid");
        }
        return handle.withValue(value -> {
            TtlvStructureView view = TtlvStructureView.fromNative(
                    NativeExtensionRegistry.credentialTtlvView(value), this);
            activeViews.incrementAndGet();
            try {
                try (view) {
                    return callback.apply(view);
                }
            } finally {
                activeViews.decrementAndGet();
            }
        });
    }

    /** Transfers the original ordered TTLV structure without rebuilding it. */
    public TtlvStructure intoTtlv() {
        if (activeViews.get() != 0) {
            throw new InvalidInputException("credential is in use by a scoped view");
        }
        return CredentialInterop.intoTtlv(handle, NativeExtensionRegistry::credentialIntoTtlv);
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "Credential[redacted]";
    }
}
