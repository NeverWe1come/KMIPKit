package org.kmipkit.credentials;

import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Function;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/**
 * A server-sourced Attestation Nonce preserving its exact TTLV structure.
 *
 * <p>Generic TTLV views expose values to application code. Any byte arrays the
 * application creates from those views are Java-managed copies and cannot be
 * deterministically zeroized by KMIPKit.</p>
 */
public final class Nonce implements AutoCloseable {
    private final NativeHandle handle;
    private final AtomicInteger activeViews = new AtomicInteger();

    private Nonce(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.NONCE);
    }

    /** Consumes and validates a TTLV Structure containing Nonce ID and Nonce Value. */
    public static Nonce fromTtlv(TtlvStructure structure) {
        if (structure == null) {
            throw new org.kmipkit.InvalidInputException("nonce structure is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> {
            NativeHandle owner = structure.nativeHandleOwner();
            long[] outNonce = new long[1];
            int status = NativeExtensionRegistry.nonceFromTtlv(owner.get(), outNonce);
            if (status != CredentialInterop.SHARED_OWNER_STATUS) {
                owner.transfer();
            }
            CredentialInterop.throwIfFailed(status, "nonce");
            return new Nonce(outNonce[0]);
        });
    }

    /**
     * Lends a read-only TTLV view and closes its native view when the callback returns.
     * The Nonce cannot be consumed until the callback has returned.
     */
    public <T> T withTtlv(Function<TtlvStructureView, T> callback) {
        if (callback == null) {
            throw new org.kmipkit.InvalidInputException("nonce view callback is invalid");
        }
        return handle.withValue(value -> {
            TtlvStructureView view = TtlvStructureView.fromNative(
                    NativeExtensionRegistry.nonceTtlvView(value), this);
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

    /** Transfers the original ordered TTLV Structure without rebuilding it. */
    public TtlvStructure intoTtlv() {
        if (activeViews.get() != 0) {
            throw new InvalidInputException("nonce is in use by a scoped view");
        }
        return CredentialInterop.intoTtlv(handle, NativeExtensionRegistry::nonceIntoTtlv);
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "Nonce[redacted]";
    }
}
