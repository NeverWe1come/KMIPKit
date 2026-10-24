package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.jspecify.annotations.Nullable;

/** Owned KMIP 2.1 Revoke request with optional identifier and compromise date. */
public final class RevokeRequest implements AutoCloseable {
    private final NativeHandle handle;

    private RevokeRequest(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.REVOKE_REQUEST);
    }

    /**
     * Creates a Revoke request. The required reason is consumed; a null identifier omits the
     * Unique Identifier field, and a null date omits Compromise Occurrence Date.
     */
    public static RevokeRequest create(
            @Nullable UniqueIdentifier uniqueIdentifier,
            RevocationReason reason,
            @Nullable Long compromiseOccurrenceDate) {
        if (reason == null) {
            throw new InvalidInputException("Revoke request reason is required");
        }
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long consumedReason = NativeHandle.transferAll(reason.nativeHandleOwner())[0];
            return new RevokeRequest(NativeExtensionRegistry.revokeRequestCreate(
                    uniqueIdentifier == null ? 0 : uniqueIdentifier.nativeKind(),
                    identifierText,
                    uniqueIdentifier == null ? 0 : uniqueIdentifier.nativeNumber(),
                    consumedReason,
                    compromiseOccurrenceDate != null,
                    compromiseOccurrenceDate == null ? 0 : compromiseOccurrenceDate));
        } finally {
            if (identifierText != null) java.util.Arrays.fill(identifierText, (byte) 0);
        }
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    @Override
    public void close() {
        handle.close();
    }
}
