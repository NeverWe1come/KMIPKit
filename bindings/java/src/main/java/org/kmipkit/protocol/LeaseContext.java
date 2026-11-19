package org.kmipkit.protocol;

import java.util.Objects;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.internal.NativeHandle;

/** Client lease lifecycle context tracking key usage authorization expiration. */
public final class LeaseContext implements AutoCloseable {
    private final NativeHandle handle;

    private LeaseContext(long nativeHandle) {
        this.handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.LEASE_CONTEXT);
    }

    public static LeaseContext create(UniqueIdentifier uniqueIdentifier, long obtainedAtUnixSecs, long leaseTimeSecs) {
        Objects.requireNonNull(uniqueIdentifier, "uniqueIdentifier");
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = NativeExtensionRegistry.leaseContextCreate(
                uniqueIdentifier.nativeKind(),
                uniqueIdentifier.nativeText(),
                uniqueIdentifier.nativeNumber(),
                obtainedAtUnixSecs,
                leaseTimeSecs);
        return new LeaseContext(nativeHandle);
    }

    public UniqueIdentifier uniqueIdentifier() {
        return handle.withValue(NativeExtensionRegistry::leaseContextUniqueIdentifier);
    }

    public long expiresAt() {
        return handle.withValue(NativeExtensionRegistry::leaseContextExpiresAt);
    }

    public boolean isExpired(long currentTimeUnixSecs) {
        return handle.withValue(raw -> NativeExtensionRegistry.leaseContextIsExpired(raw, currentTimeUnixSecs));
    }

    public void checkValid(long currentTimeUnixSecs) {
        handle.withValue(raw -> {
            NativeExtensionRegistry.leaseContextCheckValid(raw, currentTimeUnixSecs);
            return null;
        });
    }

    @Override
    public void close() {
        handle.close();
    }
}
