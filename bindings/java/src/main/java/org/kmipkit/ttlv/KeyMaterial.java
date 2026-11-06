package org.kmipkit.ttlv;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Owned, validated KMIP Key Material with explicit format and a TTLV view. */
public final class KeyMaterial extends ManagedTtlvValue {
    private final long formatRaw;

    private KeyMaterial(long nativeHandle, long formatRaw) {
        super(nativeHandle);
        this.formatRaw = formatRaw;
    }

    /** Copies and validates Key Material for the supplied open Key Format Type. */
    public static KeyMaterial fromItem(long formatRaw, TtlvItem item) {
        if (formatRaw < 0 || formatRaw > 0xffff_ffffL || item == null) {
            throw new InvalidInputException("Key Material input is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(value -> new KeyMaterial(
                NativeExtensionRegistry.keyMaterialFromItem((int) formatRaw, value), formatRaw));
    }

    /** Returns the source-preserving Key Format Type raw value. */
    public long formatRaw() { return formatRaw; }

}
