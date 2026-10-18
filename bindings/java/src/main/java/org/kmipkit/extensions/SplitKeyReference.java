package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Caller-known local context pairing one split key identifier with its threshold. */
public final class SplitKeyReference implements AutoCloseable {
    private final NativeHandle handle;
    private final UniqueIdentifier uniqueIdentifier;
    private final long splitKeyThreshold;

    /** Creates an owned native reference; the threshold is local context and is never serialized. */
    public SplitKeyReference(UniqueIdentifier uniqueIdentifier, long splitKeyThreshold) {
        if (uniqueIdentifier == null || splitKeyThreshold < 0 || splitKeyThreshold > 0xffff_ffffL) {
            throw new InvalidInputException("split key reference is invalid");
        }
        this.uniqueIdentifier = uniqueIdentifier;
        this.splitKeyThreshold = splitKeyThreshold;
        byte[] identifierText = uniqueIdentifier.nativeText();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long nativeHandle = NativeExtensionRegistry.splitKeyReferenceCreate(
                    uniqueIdentifier.nativeKind(), identifierText,
                    uniqueIdentifier.nativeNumber(), splitKeyThreshold);
            handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SPLIT_KEY_REFERENCE);
        } finally {
            if (identifierText != null) java.util.Arrays.fill(identifierText, (byte) 0);
        }
    }

    /** Returns the exact identifier value associated with this threshold. */
    public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }

    /** Returns this identifier occurrence's caller-known local threshold. */
    public long splitKeyThreshold() { return splitKeyThreshold; }

    NativeHandle nativeHandleOwner() { return handle; }

    @Override public void close() { handle.close(); }

    @Override public String toString() {
        return "SplitKeyReference[uniqueIdentifier=redacted, splitKeyThreshold="
                + splitKeyThreshold + "]";
    }
}
