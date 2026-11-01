package org.kmipkit.credentials;

import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Function;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/**
 * A validated Attestation Credential Value preserving its original TTLV structure.
 *
 * <p>Byte arrays copied from a TTLV view are Java-managed and cannot be deterministically
 * zeroized by KMIPKit.</p>
 */
public final class AttestationCredential implements AutoCloseable {
    private final NativeHandle handle;
    private final AtomicInteger activeViews = new AtomicInteger();

    private AttestationCredential(long nativeHandle) {
        handle = NativeHandle.owned(
                nativeHandle, NativeExtensionRegistry.ATTESTATION_CREDENTIAL);
    }

    /** Consume and validate a TTLV Structure containing the Attestation Credential Value. */
    public static AttestationCredential fromTtlv(TtlvStructure structure) {
        if (structure == null) {
            throw new InvalidInputException("attestation credential structure is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> {
            NativeHandle owner = structure.nativeHandleOwner();
            long[] outCredential = new long[1];
            int status = NativeExtensionRegistry.attestationCredentialFromTtlv(
                    owner.get(), outCredential);
            if (status != CredentialInterop.SHARED_OWNER_STATUS) {
                owner.transfer();
            }
            CredentialInterop.throwIfFailed(status, "attestation credential");
            return new AttestationCredential(outCredential[0]);
        });
    }

    /**
     * Lend a read-only view of the original ordered TTLV and close it when the callback returns.
     * The credential cannot be consumed while the callback is active.
     */
    public <T> T withTtlv(Function<TtlvStructureView, T> callback) {
        if (callback == null) {
            throw new InvalidInputException("attestation credential view callback is invalid");
        }
        return handle.withValue(value -> {
            TtlvStructureView view = TtlvStructureView.fromNative(
                    NativeExtensionRegistry.attestationCredentialTtlvView(value), this);
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
            throw new InvalidInputException(
                    "attestation credential is in use by a scoped view");
        }
        return CredentialInterop.intoTtlv(
                handle, NativeExtensionRegistry::attestationCredentialIntoTtlv);
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "AttestationCredential[redacted]";
    }
}
