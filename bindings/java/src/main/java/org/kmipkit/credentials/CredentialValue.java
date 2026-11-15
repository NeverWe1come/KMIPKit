package org.kmipkit.credentials;

import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Function;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/** A raw-preserving KMIP Credential Value with a scoped view of its original TTLV. */
public final class CredentialValue implements AutoCloseable {
    private final NativeHandle handle;
    private final AtomicInteger activeViews = new AtomicInteger();

    private CredentialValue(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CREDENTIAL_VALUE);
    }

    /** Consume and validate a Credential Value while retaining its raw Credential Type. */
    public static CredentialValue fromTtlv(long credentialTypeRaw, TtlvStructure structure) {
        if (credentialTypeRaw < 0 || credentialTypeRaw > 0xffff_ffffL) {
            throw new InvalidInputException(
                    "Credential Type is outside its unsigned 32-bit range");
        }
        if (structure == null) {
            throw new InvalidInputException("credential value structure is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> {
            NativeHandle owner = structure.nativeHandleOwner();
            long[] outValue = new long[1];
            int status = NativeExtensionRegistry.credentialValueFromTtlv(
                    credentialTypeRaw, owner.get(), outValue);
            if (status != CredentialInterop.SHARED_OWNER_STATUS) {
                owner.transfer();
            }
            CredentialInterop.throwIfFailed(status, "credential value");
            return new CredentialValue(outValue[0]);
        });
    }

    /** Return the original unsigned 32-bit Credential Type discriminator. */
    public long credentialTypeRaw() {
        return handle.withValue(NativeExtensionRegistry::credentialValueTypeRaw);
    }

    /** Lend a read-only view of the original ordered TTLV and close it when the callback returns. */
    public <T> T withTtlv(Function<TtlvStructureView, T> callback) {
        if (callback == null) {
            throw new InvalidInputException("credential value view callback is invalid");
        }
        return handle.withValue(value -> {
            TtlvStructureView view = TtlvStructureView.fromNative(
                    NativeExtensionRegistry.credentialValueTtlvView(value), this);
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

    /** Transfer the original ordered TTLV Structure without rebuilding it. */
    public TtlvStructure intoTtlv() {
        if (activeViews.get() != 0) {
            throw new InvalidInputException("credential value is in use by a scoped view");
        }
        return CredentialInterop.intoTtlv(
                handle, NativeExtensionRegistry::credentialValueIntoTtlv);
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "CredentialValue[redacted]";
    }
}
