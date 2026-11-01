package org.kmipkit.credentials;

import java.util.OptionalLong;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Function;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/** A validated Hashed Password Credential Value preserving its exact original TTLV. */
public final class HashedPasswordCredential implements AutoCloseable {
    private final NativeHandle handle;
    private final AtomicInteger activeViews = new AtomicInteger();

    private HashedPasswordCredential(long nativeHandle) {
        handle = NativeHandle.owned(
                nativeHandle, NativeExtensionRegistry.HASHED_PASSWORD_CREDENTIAL);
    }

    /** Consume and validate a Hashed Password Credential Value Structure. */
    public static HashedPasswordCredential fromTtlv(TtlvStructure structure) {
        if (structure == null) {
            throw new InvalidInputException("hashed password credential structure is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> {
            NativeHandle owner = structure.nativeHandleOwner();
            long[] outCredential = new long[1];
            int status = NativeExtensionRegistry.hashedPasswordCredentialFromTtlv(
                    owner.get(), outCredential);
            if (status != CredentialInterop.SHARED_OWNER_STATUS) {
                owner.transfer();
            }
            CredentialInterop.throwIfFailed(status, "hashed password credential");
            return new HashedPasswordCredential(outCredential[0]);
        });
    }

    /** Returns the caller-supplied Hashing Algorithm, preserving unknown unsigned values. */
    public OptionalLong hashingAlgorithmRaw() {
        return handle.withValue(value -> {
            long[] outPresentAndRaw = new long[2];
            CredentialInterop.throwIfFailed(
                    NativeExtensionRegistry.hashedPasswordCredentialHashingAlgorithmRaw(
                            value, outPresentAndRaw),
                    "hashed password credential hashing algorithm");
            if (outPresentAndRaw[0] == 0) {
                return OptionalLong.empty();
            }
            return OptionalLong.of(Integer.toUnsignedLong((int) outPresentAndRaw[1]));
        });
    }

    /** Returns the effective Hashing Algorithm without adding an omitted field to the TTLV. */
    public long effectiveHashingAlgorithmRaw() {
        return handle.withValue(value -> NativeExtensionRegistry
                .hashedPasswordCredentialEffectiveHashingAlgorithmRaw(value));
    }

    /** Lend a read-only view of the original TTLV and close it when the callback returns. */
    public <T> T withTtlv(Function<TtlvStructureView, T> callback) {
        if (callback == null) {
            throw new InvalidInputException("hashed password credential view callback is invalid");
        }
        return handle.withValue(value -> {
            TtlvStructureView view = TtlvStructureView.fromNative(
                    NativeExtensionRegistry.hashedPasswordCredentialTtlvView(value), this);
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
            throw new InvalidInputException("hashed password credential is in use by a scoped view");
        }
        return CredentialInterop.intoTtlv(
                handle, NativeExtensionRegistry::hashedPasswordCredentialIntoTtlv);
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "HashedPasswordCredential[redacted]";
    }
}
