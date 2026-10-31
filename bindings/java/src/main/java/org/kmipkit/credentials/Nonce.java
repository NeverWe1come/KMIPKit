package org.kmipkit.credentials;

import java.util.function.Function;

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

    private Nonce(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.NONCE);
    }

    /** Consumes and validates a TTLV Structure containing Nonce ID and Nonce Value. */
    public static Nonce fromTtlv(TtlvStructure structure) {
        if (structure == null) {
            throw new org.kmipkit.InvalidInputException("nonce structure is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return new Nonce(NativeExtensionRegistry.nonceFromTtlv(
                structure.nativeHandleOwner().transfer()));
    }

    /** Lends a read-only TTLV view and closes its native view when the callback returns. */
    public <T> T withTtlv(Function<TtlvStructureView, T> callback) {
        if (callback == null) {
            throw new org.kmipkit.InvalidInputException("nonce view callback is invalid");
        }
        return handle.withValue(value -> {
            TtlvStructureView view = TtlvStructureView.fromNative(
                    NativeExtensionRegistry.nonceTtlvView(value), this);
            try (view) {
                return callback.apply(view);
            }
        });
    }

    /** Transfers the original ordered TTLV Structure without rebuilding it. */
    public TtlvStructure intoTtlv() {
        return TtlvStructure.fromNative(
                NativeExtensionRegistry.nonceIntoTtlv(handle.transfer()), this);
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
