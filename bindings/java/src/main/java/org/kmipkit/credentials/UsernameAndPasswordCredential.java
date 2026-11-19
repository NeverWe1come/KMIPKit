package org.kmipkit.credentials;

import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Function;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/** A validated Username and Password Credential Value preserving its exact original TTLV. */
public final class UsernameAndPasswordCredential implements AutoCloseable {
    private final NativeHandle handle;
    private final AtomicInteger activeViews = new AtomicInteger();

    private UsernameAndPasswordCredential(long nativeHandle) {
        handle = NativeHandle.owned(
                nativeHandle, NativeExtensionRegistry.USERNAME_AND_PASSWORD_CREDENTIAL);
    }

    /** Consume and validate a Username and Password Credential Value Structure. */
    public static UsernameAndPasswordCredential fromTtlv(TtlvStructure structure) {
        if (structure == null) {
            throw new InvalidInputException("username and password credential structure is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> {
            NativeHandle owner = structure.nativeHandleOwner();
            long[] outCredential = new long[1];
            int status = NativeExtensionRegistry.usernameAndPasswordCredentialFromTtlv(
                    owner.get(), outCredential);
            if (status != CredentialInterop.SHARED_OWNER_STATUS) {
                owner.transfer();
            }
            CredentialInterop.throwIfFailed(status, "username and password credential");
            return new UsernameAndPasswordCredential(outCredential[0]);
        });
    }

    /** Lend a read-only view of the original TTLV and close it when the callback returns. */
    public <T> T withTtlv(Function<TtlvStructureView, T> callback) {
        if (callback == null) {
            throw new InvalidInputException("username and password credential view callback is invalid");
        }
        return handle.withValue(value -> {
            TtlvStructureView view = TtlvStructureView.fromNative(
                    NativeExtensionRegistry.usernameAndPasswordCredentialTtlvView(value), this);
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
            throw new InvalidInputException("username and password credential is in use by a scoped view");
        }
        return CredentialInterop.intoTtlv(
                handle, NativeExtensionRegistry::usernameAndPasswordCredentialIntoTtlv);
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "UsernameAndPasswordCredential[redacted]";
    }
}
